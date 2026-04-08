use crate::{FsEvent, EventFlag};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use libc::dev_t;
use nix::sys::inotify::{Inotify, InitFlags};
use std::{
    path::PathBuf,
    thread,
    time::Duration,
};

type EventsCallback = Box<dyn FnMut(Vec<FsEvent>) + Send>;

/// Linux EventStream 实现
/// 
/// 注意：inotify 不支持历史事件回放，`since_event_id` 参数被忽略。
/// 应用启动时总是从"现在"开始监控，无法恢复上次监控之后的事件。
pub struct EventStream {
    inotify: Inotify,
    paths: Vec<String>,
    latency: f64,
    callback: EventsCallback,
}

impl EventStream {
    /// 创建新的事件流
    /// 
    /// 注意：`since_event_id` 参数在 Linux 下被忽略，因为 inotify 不支持历史事件回放。
    pub fn new(
        paths: &[&str],
        _since_event_id: u64,  // 在 Linux 下忽略此参数
        latency: f64,
        callback: EventsCallback,
    ) -> Self {
        let inotify = Inotify::init(InitFlags::empty()).expect("Failed to initialize inotify");

        let paths: Vec<String> = paths.iter().map(|s| s.to_string()).collect();

        EventStream {
            inotify,
            paths,
            latency,
            callback,
        }
    }

    pub fn spawn(self) -> Option<EventStreamHandle> {
        let (tx, rx) = unbounded();
        let inotify = self.inotify;
        let paths = self.paths;
        let latency = self.latency;
        let mut callback = self.callback;

        // Add watches for all paths
        for path in &paths {
            let watch_mask = nix::sys::inotify::AddWatchFlags::IN_ACCESS |
                             nix::sys::inotify::AddWatchFlags::IN_MODIFY |
                             nix::sys::inotify::AddWatchFlags::IN_ATTRIB |
                             nix::sys::inotify::AddWatchFlags::IN_CLOSE_WRITE |
                             nix::sys::inotify::AddWatchFlags::IN_MOVED_FROM |
                             nix::sys::inotify::AddWatchFlags::IN_MOVED_TO |
                             nix::sys::inotify::AddWatchFlags::IN_CREATE |
                             nix::sys::inotify::AddWatchFlags::IN_DELETE |
                             nix::sys::inotify::AddWatchFlags::IN_DELETE_SELF |
                             nix::sys::inotify::AddWatchFlags::IN_MOVE_SELF |
                             nix::sys::inotify::AddWatchFlags::IN_ONLYDIR;

            let result = inotify.add_watch(path.as_str(), watch_mask);
            if result.is_err() {
                eprintln!("Failed to add inotify watch for path: {}", path);
            }
        }

        let handle = thread::Builder::new()
            .name("cardinal-sdk-linux-event-stream".to_string())
            .spawn(move || {
                let mut pending_events = Vec::new();
                // 事件 ID 只是简单的计数器，不持久化
                // 应用重启后无法恢复历史事件，需要完整重新扫描
                let mut event_id_counter: u64 = 0;

                loop {
                    match inotify.read_events() {
                        Ok(events) => {
                            for event in events {
                                event_id_counter += 1;
                                if let Some(ref path) = event.name {
                                    let path_str = path.to_string_lossy();
                                    let fs_event = FsEvent {
                                        path: PathBuf::from(path_str.as_ref()),
                                        flag: EventFlag::from_inotify_mask(event),
                                        id: event_id_counter,
                                    };
                                    pending_events.push(fs_event);
                                }
                            }

                            // Send events after latency period
                            if !pending_events.is_empty() {
                                (callback)(pending_events.drain(..).collect());
                            }
                        }
                        Err(_) => break, // Error reading events, exit loop
                    }

                    // Small sleep to implement latency
                    thread::sleep(Duration::from_millis((latency * 1000.0) as u64));
                }
            })
            .unwrap();

        Some(EventStreamHandle {
            _handle: handle,
            _tx: tx,
            _rx: rx,
        })
    }

    /// 获取被监控的设备 ID
    /// 
    /// 注意：Linux inotify 不提供设备 ID，返回 0 作为占位符。
    pub fn dev(&self) -> dev_t {
        // On Linux, getting the device ID for an inotify instance is not straightforward
        // Return a dummy value for now, this may need more sophisticated handling
        0
    }
}

pub struct EventStreamHandle {
    _handle: thread::JoinHandle<()>,
    _tx: Sender<Vec<FsEvent>>,
    _rx: Receiver<Vec<FsEvent>>,
}

pub struct EventWatcher {
    receiver: Receiver<Vec<FsEvent>>,
    _cancellation_token: Sender<()>,
}

impl std::ops::Deref for EventWatcher {
    type Target = Receiver<Vec<FsEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl std::ops::DerefMut for EventWatcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}

impl EventWatcher {
    pub fn noop() -> Self {
        let (_, receiver) = unbounded();
        let (cancellation_token, _) = bounded::<()>(1);
        Self {
            receiver,
            _cancellation_token: cancellation_token,
        }
    }

    pub fn spawn(
        path: String,
        since_event_id: u64,
        latency: f64,
    ) -> (dev_t, EventWatcher) {
        let (cancellation_tx, cancellation_rx) = bounded::<()>(1);
        let (sender, receiver) = unbounded();
        
        let stream = EventStream::new(
            &[&path],
            since_event_id,
            latency,
            Box::new(move |events| {
                let _ = sender.send(events);
            }),
        );
        
        let dev = stream.dev();
        
        thread::Builder::new()
            .name("cardinal-sdk-event-watcher".to_string())
            .spawn(move || {
                let _stream_handle = stream.spawn().expect("failed to spawn event stream");
                
                // Wait for cancellation
                let _ = cancellation_rx.recv();
            })
            .unwrap();

        (
            dev,
            EventWatcher {
                receiver,
                _cancellation_token: cancellation_tx,
            },
        )
    }
}

// ============================================================================
// Linux 平台测试
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::RecvTimeoutError;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

    #[test]
    fn event_watcher_on_non_existent_path() {
        // Linux inotify 对不存在路径会添加 watch 失败
        // 当前实现会打印错误但仍会创建 EventWatcher
        // 测试应验证不会收到任何事件
        let (_dev, watcher) = EventWatcher::spawn("/nonexistent_path_12345".to_string(), 0, 0.05);

        // 不应该收到任何事件（因为 inotify watch 添加失败）
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut received_any = false;
        while Instant::now() < deadline {
            match watcher.recv_timeout(Duration::from_millis(200)) {
                Ok(_batch) => {
                    received_any = true;
                    break;
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        assert!(
            !received_any,
            "event watcher on non-existent path should not deliver events"
        );
    }

    #[test]
    fn drop_then_respawn_event_watcher_delivers_events() {
        let temp_dir = tempdir().expect("failed to create tempdir");
        let watched_root = temp_dir.path().to_path_buf();
        let watched_root = watched_root.canonicalize().expect("failed to canonicalize");
        let watch_path = watched_root
            .to_str()
            .expect("tempdir path should be utf8")
            .to_string();

        let (_, initial_watcher) = EventWatcher::spawn(watch_path.clone(), 0, 0.05);
        drop(initial_watcher);

        // Give the background thread a moment to observe the drop.
        std::thread::sleep(Duration::from_millis(500));

        let (_, respawned_watcher) = EventWatcher::spawn(watch_path.clone(), 0, 0.05);

        // Allow the stream to start before triggering filesystem activity.
        std::thread::sleep(Duration::from_millis(500));

        let created_file = watched_root.join("respawn_event.txt");
        std::fs::write(&created_file, "cardinal").expect("failed to write test file");

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut observed_change = false;
        let mut all_events = Vec::new();
        let expected_filename = created_file.file_name().expect("should have filename");
        
        while Instant::now() < deadline {
            match respawned_watcher.recv_timeout(Duration::from_millis(200)) {
                Ok(batch) => {
                    all_events.extend(batch);
                    // Linux inotify 返回的是相对于监控目录的相对路径（仅文件名）
                    if all_events
                        .iter()
                        .any(|event| event.path.file_name() == Some(expected_filename))
                    {
                        observed_change = true;
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        drop(respawned_watcher);
        assert!(
            observed_change,
            "respawned watcher failed to deliver file change event"
        );
    }
}