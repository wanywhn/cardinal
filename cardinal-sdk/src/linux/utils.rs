use libc::dev_t;
use std::{collections::HashMap, time::SystemTime};

/// 获取当前时间戳（秒）
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// 获取当前事件 ID
/// 
/// 注意：Linux inotify 不提供全局事件 ID（如 macOS FSEvents）。
/// 这里使用原子计数器模拟，每次应用启动从 0 开始。
/// 这意味着无法恢复上次监控之后的历史事件。
pub fn current_event_id() -> u64 {
    // On Linux, we don't have a global event ID like macOS FSEvents
    // We'll use a timestamp-based ID as a placeholder
    use std::sync::atomic::{AtomicU64, Ordering};
    static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
    EVENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 将事件 ID 转换为时间戳
/// 
/// 注意：Linux inotify 不支持此操作。返回当前时间戳作为占位符。
pub fn event_id_to_timestamp(_dev: dev_t, _event_id: u64, _cache: &mut HashMap<i64, u64>) -> i64 {
    // On Linux with inotify, we don't have the same concept as macOS FSEvents
    // Return current timestamp as a placeholder
    current_timestamp()
}