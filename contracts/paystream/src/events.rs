use soroban_sdk::{contractevent, Address, Env};

// ── Event types (SDK v25 contractevent pattern) ──────────────────────────────

#[contractevent]
pub struct StreamCreated {
    pub stream_id: u64,
    pub sender: Address,
    pub recipient: Address,
}

#[contractevent]
pub struct StreamWithdrawn {
    pub stream_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

#[contractevent]
pub struct StreamPaused {
    pub stream_id: u64,
    pub sender: Address,
}

#[contractevent]
pub struct StreamResumed {
    pub stream_id: u64,
    pub sender: Address,
}

#[contractevent]
pub struct StreamCancelled {
    pub stream_id: u64,
    pub sender: Address,
}

#[contractevent]
pub struct StreamToppedUp {
    pub stream_id: u64,
    pub sender: Address,
    pub amount: i128,
    pub new_deposit: i128,
    pub new_rate_per_sec: i128,
}

#[contractevent]
pub struct StreamExtended {
    pub stream_id: u64,
    pub sender: Address,
    pub new_end_time: u64,
    pub new_rate_per_sec: i128,
}

// ── Publish helpers ───────────────────────────────────────────────────────────

pub fn stream_created(env: &Env, stream_id: u64, sender: &Address, recipient: &Address) {
    StreamCreated {
        stream_id,
        sender: sender.clone(),
        recipient: recipient.clone(),
    }
    .publish(env);
}

pub fn stream_withdrawn(env: &Env, stream_id: u64, recipient: &Address, amount: i128) {
    StreamWithdrawn {
        stream_id,
        recipient: recipient.clone(),
        amount,
    }
    .publish(env);
}

pub fn stream_paused(env: &Env, stream_id: u64, sender: &Address) {
    StreamPaused {
        stream_id,
        sender: sender.clone(),
    }
    .publish(env);
}

pub fn stream_resumed(env: &Env, stream_id: u64, sender: &Address) {
    StreamResumed {
        stream_id,
        sender: sender.clone(),
    }
    .publish(env);
}

pub fn stream_cancelled(env: &Env, stream_id: u64, sender: &Address) {
    StreamCancelled {
        stream_id,
        sender: sender.clone(),
    }
    .publish(env);
}

pub fn stream_topped_up(
    env: &Env,
    stream_id: u64,
    sender: &Address,
    amount: i128,
    new_deposit: i128,
    new_rate_per_sec: i128,
) {
    StreamToppedUp {
        stream_id,
        sender: sender.clone(),
        amount,
        new_deposit,
        new_rate_per_sec,
    }
    .publish(env);
}

pub fn stream_extended(
    env: &Env,
    stream_id: u64,
    sender: &Address,
    new_end_time: u64,
    new_rate_per_sec: i128,
) {
    StreamExtended {
        stream_id,
        sender: sender.clone(),
        new_end_time,
        new_rate_per_sec,
    }
    .publish(env);
}
