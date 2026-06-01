use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PayStreamError {
    StreamNotFound       = 1,
    NotAuthorized        = 2,
    StreamNotActive      = 3,
    StreamAlreadyPaused  = 4,
    StreamNotPaused      = 5,
    NothingToClaim       = 6,
    InvalidAmount        = 7,
    InvalidTimeRange     = 8,
    StreamAlreadyEnded   = 9,
}
