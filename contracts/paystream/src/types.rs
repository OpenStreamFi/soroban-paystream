use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    Active,
    Paused,
    Cancelled,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Stream {
    pub sender:        Address,
    pub recipient:     Address,
    pub token:         Address,
    pub deposit:       i128,
    pub rate_per_sec:  i128,
    pub start_time:    u64,
    pub end_time:      u64,
    pub claimed:       i128,
    pub status:        StreamStatus,
    pub pause_time:    u64,
    pub total_paused:  u64,
}
