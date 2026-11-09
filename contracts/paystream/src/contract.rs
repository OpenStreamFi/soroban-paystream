use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};
use crate::errors::PayStreamError;
use crate::events;
use crate::storage;
use crate::types::{Stream, StreamStatus};

#[contract]
pub struct PayStreamContract;

#[contractimpl]
impl PayStreamContract {

    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        deposit: i128,
        start_time: u64,
        end_time: u64,
    ) -> Result<u64, PayStreamError> {
        sender.require_auth();

        if deposit <= 0 {
            return Err(PayStreamError::InvalidAmount);
        }
        if end_time <= start_time {
            return Err(PayStreamError::InvalidTimeRange);
        }
        if end_time <= env.ledger().timestamp() {
            return Err(PayStreamError::StreamAlreadyEnded);
        }

        let duration = (end_time - start_time) as i128;
        let rate_per_sec = deposit / duration;

        if rate_per_sec <= 0 {
            return Err(PayStreamError::InvalidAmount);
        }

        // Pull tokens from sender into the contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &deposit);

        let stream_id = storage::get_stream_count(&env) + 1;

        let stream = Stream {
            sender:       sender.clone(),
            recipient:    recipient.clone(),
            token:        token.clone(),
            deposit,
            rate_per_sec,
            start_time,
            end_time,
            claimed:      0,
            status:       StreamStatus::Active,
            pause_time:   0,
            total_paused: 0,
            accrued:      0,
        };

        storage::save_stream(&env, stream_id, &stream);
        storage::set_stream_count(&env, stream_id);
        storage::add_user_stream(&env, &sender, stream_id);
        storage::add_user_stream(&env, &recipient, stream_id);

        events::stream_created(&env, stream_id, &sender, &recipient);

        Ok(stream_id)
    }

    pub fn get_claimable(env: Env, stream_id: u64) -> Result<i128, PayStreamError> {
        let stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        if stream.status == StreamStatus::Cancelled {
            return Ok(0);
        }

        let earned = Self::compute_earned(&env, &stream);
        let claimable = earned - stream.claimed;

        Ok(claimable.max(0))
    }

    fn compute_earned(env: &Env, stream: &Stream) -> i128 {
        let now = if stream.status == StreamStatus::Paused {
            stream.pause_time
        } else {
            env.ledger().timestamp().min(stream.end_time)
        };

        if now <= stream.start_time {
            return stream.accrued;
        }

        let duration = (stream.end_time - stream.start_time) as i128;
        let elapsed = (now - stream.start_time) as i128 - stream.total_paused as i128;

        let segment_deposit = stream.deposit - stream.accrued;
        let segment_earned = if elapsed >= duration {
            segment_deposit
        } else {
            elapsed * stream.rate_per_sec
        };

        stream.accrued + segment_earned
    }

    pub fn withdraw(env: Env, stream_id: u64) -> Result<i128, PayStreamError> {
        let mut stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        stream.recipient.require_auth();

        if stream.status == StreamStatus::Cancelled {
            return Err(PayStreamError::StreamNotActive);
        }

        let claimable = Self::get_claimable(env.clone(), stream_id)?;

        if claimable <= 0 {
            return Err(PayStreamError::NothingToClaim);
        }

        stream.claimed += claimable;

        // Mark completed if fully drained
        let now = env.ledger().timestamp();
        if now >= stream.end_time && stream.claimed >= stream.deposit {
            stream.status = StreamStatus::Completed;
        }

        storage::save_stream(&env, stream_id, &stream);

        let token_client = token::Client::new(&env, &stream.token);
        token_client.transfer(&env.current_contract_address(), &stream.recipient, &claimable);

        events::stream_withdrawn(&env, stream_id, &stream.recipient, claimable);

        Ok(claimable)
    }

    pub fn pause_stream(env: Env, stream_id: u64) -> Result<(), PayStreamError> {
        let mut stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        stream.sender.require_auth();

        if stream.status != StreamStatus::Active {
            return Err(PayStreamError::StreamAlreadyPaused);
        }

        stream.status = StreamStatus::Paused;
        stream.pause_time = env.ledger().timestamp();

        storage::save_stream(&env, stream_id, &stream);
        events::stream_paused(&env, stream_id, &stream.sender);

        Ok(())
    }

    pub fn resume_stream(env: Env, stream_id: u64) -> Result<(), PayStreamError> {
        let mut stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        stream.sender.require_auth();

        if stream.status != StreamStatus::Paused {
            return Err(PayStreamError::StreamNotPaused);
        }

        let paused_duration = env.ledger().timestamp() - stream.pause_time;
        stream.total_paused += paused_duration;
        stream.pause_time = 0;
        stream.status = StreamStatus::Active;

        storage::save_stream(&env, stream_id, &stream);
        events::stream_resumed(&env, stream_id, &stream.sender);

        Ok(())
    }

    pub fn cancel_stream(env: Env, stream_id: u64) -> Result<(), PayStreamError> {
        let mut stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        stream.sender.require_auth();

        if stream.status == StreamStatus::Cancelled
            || stream.status == StreamStatus::Completed
        {
            return Err(PayStreamError::StreamNotActive);
        }

        // Pay out whatever recipient has earned so far
        let claimable = Self::get_claimable(env.clone(), stream_id)?;
        let token_client = token::Client::new(&env, &stream.token);

        if claimable > 0 {
            stream.claimed += claimable;
            token_client.transfer(
                &env.current_contract_address(),
                &stream.recipient,
                &claimable,
            );
        }

        // Refund remaining to sender
        let refund = stream.deposit - stream.claimed;
        if refund > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &stream.sender,
                &refund,
            );
        }

        stream.status = StreamStatus::Cancelled;
        storage::save_stream(&env, stream_id, &stream);
        events::stream_cancelled(&env, stream_id, &stream.sender);

        Ok(())
    }

    pub fn top_up(env: Env, stream_id: u64, amount: i128) -> Result<(), PayStreamError> {
        let mut stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        stream.sender.require_auth();

        if stream.status != StreamStatus::Active {
            return Err(PayStreamError::StreamNotActive);
        }

        if amount <= 0 {
            return Err(PayStreamError::InvalidAmount);
        }

        let now = env.ledger().timestamp();
        if now >= stream.end_time {
            return Err(PayStreamError::StreamAlreadyEnded);
        }

        let earned_so_far = Self::compute_earned(&env, &stream);

        // Pull additional tokens from sender into contract
        let token_client = token::Client::new(&env, &stream.token);
        token_client.transfer(&stream.sender, &env.current_contract_address(), &amount);

        // Lock in already earned tokens
        stream.deposit += amount;
        stream.accrued = earned_so_far;

        // Reset segment start to now (or start_time if stream hasn't started yet)
        if now > stream.start_time {
            stream.start_time = now;
        }
        stream.total_paused = 0;

        let remaining_duration = (stream.end_time - stream.start_time) as i128;
        if remaining_duration <= 0 {
            return Err(PayStreamError::InvalidTimeRange);
        }

        let remaining_deposit = stream.deposit - stream.accrued;
        let new_rate = remaining_deposit / remaining_duration;

        if new_rate <= 0 {
            return Err(PayStreamError::InvalidAmount);
        }

        stream.rate_per_sec = new_rate;

        storage::save_stream(&env, stream_id, &stream);
        events::stream_topped_up(
            &env,
            stream_id,
            &stream.sender,
            amount,
            stream.deposit,
            stream.rate_per_sec,
        );

        Ok(())
    }

    pub fn extend(env: Env, stream_id: u64, new_end_time: u64) -> Result<(), PayStreamError> {
        let mut stream = storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)?;

        stream.sender.require_auth();

        if stream.status != StreamStatus::Active {
            return Err(PayStreamError::StreamNotActive);
        }

        if new_end_time <= stream.end_time {
            return Err(PayStreamError::InvalidTimeRange);
        }

        let now = env.ledger().timestamp();
        if now >= stream.end_time {
            return Err(PayStreamError::StreamAlreadyEnded);
        }

        let earned_so_far = Self::compute_earned(&env, &stream);

        // Lock in already earned tokens
        stream.accrued = earned_so_far;

        // Reset segment start to now (or start_time if stream hasn't started yet)
        if now > stream.start_time {
            stream.start_time = now;
        }
        stream.end_time = new_end_time;
        stream.total_paused = 0;

        let remaining_duration = (stream.end_time - stream.start_time) as i128;
        if remaining_duration <= 0 {
            return Err(PayStreamError::InvalidTimeRange);
        }

        let remaining_deposit = stream.deposit - stream.accrued;
        let new_rate = remaining_deposit / remaining_duration;

        if new_rate <= 0 {
            return Err(PayStreamError::InvalidAmount);
        }

        stream.rate_per_sec = new_rate;

        storage::save_stream(&env, stream_id, &stream);
        events::stream_extended(&env, stream_id, &stream.sender, new_end_time, stream.rate_per_sec);

        Ok(())
    }

    pub fn get_stream(env: Env, stream_id: u64) -> Result<Stream, PayStreamError> {
        storage::load_stream(&env, stream_id)
            .ok_or(PayStreamError::StreamNotFound)
    }

    pub fn get_streams_by_user(env: Env, user: Address) -> Vec<u64> {
        storage::get_user_streams(&env, &user)
    }

    pub fn get_stream_count(env: Env) -> u64 {
        storage::get_stream_count(&env)
    }
}
