use crate::utils;
use bincode::Decode;
use bincode::Encode;

/// A event id for event ordering.
#[derive(Debug, Default, Clone, Copy, Decode, Encode, PartialOrd, PartialEq, Eq, Ord)]
pub struct EventId {
    pub since: u64,
    pub timestamp: i64,
}

impl EventId {
    // Return current event id and timestamp.
    pub fn now() -> Self {
        let since = utils::current_event_id();
        let timestamp = utils::current_timestamp();
        Self { since, timestamp }
    }

    pub fn now_with_id(since: u64) -> Self {
        let timestamp = utils::current_timestamp();
        Self { since, timestamp }
    }
}
