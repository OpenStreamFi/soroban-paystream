#[cfg(test)]
mod tests {
    use crate::contract::{PayStreamContract, PayStreamContractClient};
    use crate::types::StreamStatus;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        token, Address, Env,
    };

    // ─── Helpers ─────────────────────────────────────────────────────────────────

    /// Creates a new Env with the ledger timestamp set to 1000.
    fn setup_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1000);
        env
    }

    /// Registers a mock Stellar Asset Contract, mints `amount` tokens to `to`.
    /// Returns the token contract address.
    fn setup_token(env: &Env, admin: &Address, to: &Address, amount: i128) -> Address {
        let token_admin = token::StellarAssetClient::new(
            env,
            &env.register_stellar_asset_contract_v2(admin.clone()).address(),
        );
        token_admin.mint(to, &amount);
        token_admin.address.clone()
    }

    /// Registers the PayStreamContract and returns a client.
    fn setup_contract(env: &Env) -> PayStreamContractClient<'_> {
        let contract_id = env.register(PayStreamContract, ());
        PayStreamContractClient::new(env, &contract_id)
    }

    /// Returns sensible default stream params.
    /// deposit=3600, start_time=1000, end_time=4600 → rate = 1 token/sec.
    struct StreamParams {
        sender: Address,
        recipient: Address,
        token: Address,
        deposit: i128,
        start_time: u64,
        end_time: u64,
    }

    fn default_stream_params(
        env: &Env,
        sender: &Address,
        recipient: &Address,
        token_addr: &Address,
    ) -> StreamParams {
        let _ = env; // env available if needed in future
        StreamParams {
            sender: sender.clone(),
            recipient: recipient.clone(),
            token: token_addr.clone(),
            deposit: 3600,
            start_time: 1000,
            end_time: 4600,
        }
    }

    /// Convenience: create a stream from StreamParams via the client.
    fn create_stream_from_params(client: &PayStreamContractClient, p: &StreamParams) -> u64 {
        client
            .create_stream(
                &p.sender,
                &p.recipient,
                &p.token,
                &p.deposit,
                &p.start_time,
                &p.end_time,
            )
    }

    // ─── Group 1: test_create_stream ─────────────────────────────────────────────

    #[test]
    fn test_create_stream_success() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        let stream_id = create_stream_from_params(&client, &p);

        // Stream ID should be 1
        assert_eq!(stream_id, 1);

        // Verify stream was stored correctly
        let stream = client.get_stream(&stream_id);
        assert_eq!(stream.status, StreamStatus::Active);

        // Verify stream count
        assert_eq!(client.get_stream_count(), 1);

        // Verify sender balance decreased by deposit amount
        let token_client = token::Client::new(&env, &token_addr);
        assert_eq!(token_client.balance(&sender), 10_000 - 3600);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_create_stream_invalid_amount() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);

        client.create_stream(&sender, &recipient, &token_addr, &0, &1000, &4600);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #8)")]
    fn test_create_stream_invalid_time_range() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);

        // end_time <= start_time
        client.create_stream(&sender, &recipient, &token_addr, &3600, &4600, &4600);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #9)")]
    fn test_create_stream_already_ended() {
        let env = setup_env();
        env.ledger().set_timestamp(5000);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);

        // end_time = 3000, but current time = 5000 → already ended
        client.create_stream(&sender, &recipient, &token_addr, &3600, &1000, &3000);
    }

    // ─── Group 2: test_get_claimable ─────────────────────────────────────────────

    #[test]
    fn test_claimable_before_start() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);

        // Create stream that starts in the future (start_time = 2000, current = 1000)
        client.create_stream(&sender, &recipient, &token_addr, &3600, &2000, &5600);

        let claimable = client.get_claimable(&1);
        assert_eq!(claimable, 0);
    }

    #[test]
    fn test_claimable_mid_stream() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // Advance 1000 seconds (from 1000 → 2000)
        env.ledger().set_timestamp(2000);

        let claimable = client.get_claimable(&1);
        // 1000 seconds elapsed × 1 token/sec = 1000
        assert_eq!(claimable, 1000);
    }

    #[test]
    fn test_claimable_after_end() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // Advance past end_time
        env.ledger().set_timestamp(9000);

        let claimable = client.get_claimable(&1);
        // Should be capped at full deposit
        assert_eq!(claimable, 3600);
    }

    // ─── Group 3: test_withdraw ──────────────────────────────────────────────────

    #[test]
    fn test_withdraw_success() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // Advance 1000 seconds
        env.ledger().set_timestamp(2000);

        let withdrawn = client.withdraw(&1);
        assert_eq!(withdrawn, 1000);

        // Recipient balance should have increased
        let token_client = token::Client::new(&env, &token_addr);
        assert_eq!(token_client.balance(&recipient), 1000);

        // Stream claimed should be updated
        let stream = client.get_stream(&1);
        assert_eq!(stream.claimed, 1000);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #6)")]
    fn test_withdraw_nothing_to_claim() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);

        // Create stream that starts in the future
        client.create_stream(&sender, &recipient, &token_addr, &3600, &2000, &5600);

        // Try to withdraw immediately — nothing earned yet
        client.withdraw(&1);
    }

    #[test]
    fn test_withdraw_marks_completed() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // Advance past end_time
        env.ledger().set_timestamp(9000);

        let withdrawn = client.withdraw(&1);
        assert_eq!(withdrawn, 3600);

        // Stream should now be Completed
        let stream = client.get_stream(&1);
        assert_eq!(stream.status, StreamStatus::Completed);

        // Recipient received full deposit
        let token_client = token::Client::new(&env, &token_addr);
        assert_eq!(token_client.balance(&recipient), 3600);
    }

    #[test]
    fn test_withdraw_twice_no_double_claim() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // First withdrawal after 1000 seconds
        env.ledger().set_timestamp(2000);
        let first = client.withdraw(&1);
        assert_eq!(first, 1000);

        // Second withdrawal after 500 more seconds
        env.ledger().set_timestamp(2500);
        let second = client.withdraw(&1);
        assert_eq!(second, 500);

        // Total claimed should be 1500, not 2000
        let stream = client.get_stream(&1);
        assert_eq!(stream.claimed, 1500);

        let token_client = token::Client::new(&env, &token_addr);
        assert_eq!(token_client.balance(&recipient), 1500);
    }

    // ─── Group 4: test_pause_resume ──────────────────────────────────────────────

    #[test]
    fn test_pause_stream_success() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        env.ledger().set_timestamp(2000);
        client.pause_stream(&1);

        let stream = client.get_stream(&1);
        assert_eq!(stream.status, StreamStatus::Paused);
        assert_eq!(stream.pause_time, 2000);
    }

    #[test]
    fn test_resume_stream_success() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // Pause at 2000
        env.ledger().set_timestamp(2000);
        client.pause_stream(&1);

        // Resume at 3000 → paused for 1000 seconds
        env.ledger().set_timestamp(3000);
        client.resume_stream(&1);

        let stream = client.get_stream(&1);
        assert_eq!(stream.status, StreamStatus::Active);
        assert_eq!(stream.total_paused, 1000);
        assert_eq!(stream.pause_time, 0);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn test_pause_already_paused() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        client.pause_stream(&1);
        // Pausing again should fail
        client.pause_stream(&1);
    }

    #[test]
    fn test_claimable_excludes_paused_time() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // 1000 seconds active (1000 → 2000)
        env.ledger().set_timestamp(2000);
        client.pause_stream(&1);

        // Paused for 1000 seconds (2000 → 3000)
        env.ledger().set_timestamp(3000);
        client.resume_stream(&1);

        // 500 more seconds active (3000 → 3500)
        env.ledger().set_timestamp(3500);

        let claimable = client.get_claimable(&1);
        // 1000 + 500 = 1500 active seconds, NOT 2500
        assert_eq!(claimable, 1500);
    }

    // ─── Group 5: test_cancel ────────────────────────────────────────────────────

    #[test]
    fn test_cancel_active_stream() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // 1000 seconds elapsed → 1000 earned
        env.ledger().set_timestamp(2000);
        client.cancel_stream(&1);

        let token_client = token::Client::new(&env, &token_addr);

        // Recipient received 1000 (earned)
        assert_eq!(token_client.balance(&recipient), 1000);

        // Sender received refund: 3600 - 1000 = 2600
        // Original balance was 10000 - 3600 = 6400, now 6400 + 2600 = 9000
        assert_eq!(token_client.balance(&sender), 10_000 - 3600 + 2600);

        let stream = client.get_stream(&1);
        assert_eq!(stream.status, StreamStatus::Cancelled);
    }

    #[test]
    fn test_cancel_paused_stream() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // 1000 seconds active → earn 1000
        env.ledger().set_timestamp(2000);
        client.pause_stream(&1);

        // 500 more seconds pass while paused (no earnings)
        env.ledger().set_timestamp(2500);
        client.cancel_stream(&1);

        let token_client = token::Client::new(&env, &token_addr);

        // Recipient gets 1000 (only active time counts)
        assert_eq!(token_client.balance(&recipient), 1000);

        // Sender gets 2600 refund
        assert_eq!(token_client.balance(&sender), 10_000 - 3600 + 2600);

        let stream = client.get_stream(&1);
        assert_eq!(stream.status, StreamStatus::Cancelled);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn test_cancel_already_cancelled() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        client.cancel_stream(&1);
        // Cancelling again should fail
        client.cancel_stream(&1);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn test_withdraw_after_cancel_fails() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 10_000);
        let client = setup_contract(&env);
        let p = default_stream_params(&env, &sender, &recipient, &token_addr);

        create_stream_from_params(&client, &p);

        // Advance time so there would have been something to claim
        env.ledger().set_timestamp(2000);
        client.cancel_stream(&1);

        // Attempting withdraw after cancel should fail
        client.withdraw(&1);
    }

    // ─── Group 6: test_view_functions ────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn test_get_stream_not_found() {
        let env = setup_env();
        let client = setup_contract(&env);

        // Stream 999 doesn't exist
        client.get_stream(&999);
    }

    #[test]
    fn test_get_streams_by_user_indexes_both_parties() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 100_000);
        let client = setup_contract(&env);

        // Create two streams with the same sender, different recipients
        client.create_stream(&sender, &recipient1, &token_addr, &3600, &1000, &4600);
        client.create_stream(&sender, &recipient2, &token_addr, &3600, &1000, &4600);

        // Sender should have both stream IDs
        let sender_streams = client.get_streams_by_user(&sender);
        assert_eq!(sender_streams.len(), 2);
        assert_eq!(sender_streams.get(0).unwrap(), 1);
        assert_eq!(sender_streams.get(1).unwrap(), 2);

        // Recipient1 should only have stream 1
        let r1_streams = client.get_streams_by_user(&recipient1);
        assert_eq!(r1_streams.len(), 1);
        assert_eq!(r1_streams.get(0).unwrap(), 1);

        // Recipient2 should only have stream 2
        let r2_streams = client.get_streams_by_user(&recipient2);
        assert_eq!(r2_streams.len(), 1);
        assert_eq!(r2_streams.get(0).unwrap(), 2);
    }

    #[test]
    fn test_get_stream_count_increments() {
        let env = setup_env();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_addr = setup_token(&env, &Address::generate(&env), &sender, 100_000);
        let client = setup_contract(&env);

        assert_eq!(client.get_stream_count(), 0);

        client.create_stream(&sender, &recipient, &token_addr, &3600, &1000, &4600);
        assert_eq!(client.get_stream_count(), 1);

        client.create_stream(&sender, &recipient, &token_addr, &3600, &1000, &4600);
        assert_eq!(client.get_stream_count(), 2);

        client.create_stream(&sender, &recipient, &token_addr, &3600, &1000, &4600);
        assert_eq!(client.get_stream_count(), 3);
    }
}
