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

pub struct EventStream {
    inotify: Inotify,
    paths: Vec<String>,
    since_event_id: u64,
    latency: f64,
    callback: EventsCallback,
}

impl EventStream {
    pub fn new(
        paths: &[&str],
        since_event_id: u64,
        latency: f64,
        callback: EventsCallback,
    ) -> Self {
        let inotify = Inotify::init(InitFlags::empty()).expect("Failed to initialize inotify");

        let paths: Vec<String> = paths.iter().map(|s| s.to_string()).collect();

        EventStream {
            inotify,
            paths,
            since_event_id,
            latency,
            callback,
        }
    }

    pub fn spawn(self) -> Option<EventStreamHandle> {
        let (tx, rx) = unbounded();
        let mut inotify = self.inotify;
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
                let mut event_id_counter = self.since_event_id;

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