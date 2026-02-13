use libc::dev_t;
use std::{collections::HashMap, time::SystemTime};

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

pub fn current_event_id() -> u64 {
    // On Linux, we don't have a global event ID like macOS FSEvents
    // We'll use a timestamp-based ID as a placeholder
    use std::sync::atomic::{AtomicU64, Ordering};
    static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
    EVENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn last_event_id_before_time(_dev: dev_t, _timestamp: i64) -> u64 {
    // On Linux with inotify, we don't have the same concept as macOS FSEvents
    // Return a default value as a placeholder
    0
}

pub fn event_id_to_timestamp(_dev: dev_t, _event_id: u64, _cache: &mut HashMap<i64, u64>) -> i64 {
    // On Linux with inotify, we don't have the same concept as macOS FSEvents
    // Return current timestamp as a placeholder
    current_timestamp()
}