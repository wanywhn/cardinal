use fsevent_sys::FSEventsGetCurrentEventId;

pub fn current_timestamp() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub fn current_event_id() -> u64 {
    unsafe { FSEventsGetCurrentEventId() }
}
