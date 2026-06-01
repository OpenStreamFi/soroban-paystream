use soroban_sdk::{contracttype, Address, Env, Vec};
use crate::types::Stream;

#[contracttype]
pub enum DataKey {
    StreamCount,
    Stream(u64),
    UserStreams(Address),
}

pub fn get_stream_count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::StreamCount).unwrap_or(0)
}

pub fn set_stream_count(env: &Env, count: u64) {
    env.storage().instance().set(&DataKey::StreamCount, &count);
}

pub fn save_stream(env: &Env, id: u64, stream: &Stream) {
    env.storage().persistent().set(&DataKey::Stream(id), stream);
}

pub fn load_stream(env: &Env, id: u64) -> Option<Stream> {
    env.storage().persistent().get(&DataKey::Stream(id))
}

pub fn get_user_streams(env: &Env, user: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::UserStreams(user.clone()))
        .unwrap_or(Vec::new(env))
}

pub fn add_user_stream(env: &Env, user: &Address, stream_id: u64) {
    let mut streams = get_user_streams(env, user);
    streams.push_back(stream_id);
    env.storage()
        .persistent()
        .set(&DataKey::UserStreams(user.clone()), &streams);
}
