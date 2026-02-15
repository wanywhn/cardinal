# FS Events SDK (cardinal-sdk)

This chapter documents the `cardinal-sdk/` crate, which provides cross-platform filesystem event monitoring for Cardinal, supporting both macOS FSEvents and Linux inotify.

---

## Public surface

`cardinal-sdk/src/lib.rs` re-exports:
- `FsEvent` — a single filesystem event (path, flag, id).
- `EventFlag`, `EventType`, `ScanType` — bitflags and enums describing event semantics.
- `EventStream`, `EventWatcher` — types that own the filesystem event stream and dispatch queue.
- `FSEventStreamEventId` — underlying event ID type (u64 on Linux).
- Helpers from `utils`:
  - `current_event_id()` — current filesystem event ID for the system.
  - `event_id_to_timestamp()` — convert event IDs into wall-clock timestamps.

`SearchCache` and the Tauri backend use these to track incremental changes and rescan boundaries.

On macOS, the crate uses FSEvents, while on Linux it uses inotify for filesystem monitoring.

---

## EventStream and dispatch queue

`EventStream` provides cross-platform filesystem event monitoring:

- `EventStream::new(paths, since_event_id, latency, callback)`:
  - **macOS**: Creates a `CFArray` of watch paths and configures `FSEventStreamContext` with a boxed Rust callback (`EventsCallback`), then calls `FSEventStreamCreate` with flags:
    - `kFSEventStreamCreateFlagNoDefer`
    - `kFSEventStreamCreateFlagFileEvents`
    - `kFSEventStreamCreateFlagWatchRoot`
  - **Linux**: Initializes an inotify instance and adds watches for the specified paths using the nix crate's inotify bindings.
  - The callback converts raw event data into a `Vec<FsEvent>` and invokes the Rust closure.
- `spawn`:
  - **macOS**: Attaches the stream to a serial `DispatchQueue`, starts the stream (`FSEventStreamStart`) and returns `EventStreamWithQueue`.
  - **Linux**: Spawns a thread that reads inotify events and batches them according to the specified latency.
  - On failure, handles cleanup appropriately for the platform.
- `dev`:
  - **macOS**: Returns the `dev_t` for the device being watched via `FSEventStreamGetDeviceBeingWatched`.
  - **Linux**: Returns a dummy device ID (placeholder implementation).

Platform-specific creation flow:
```text
macOS:
paths (&[&str]) + since_event_id + latency
        │
        ▼
 EventStream::new(...)
        │   (wrap Rust closure as EventsCallback,
        │    configure FSEventStreamContext)
        ▼
 FSEventStreamCreate(...)
        │
        ▼
 EventStream::spawn()
        │   (attach to DispatchQueue, start stream)
        ▼
 EventStreamWithQueue
        │
        └─ incoming C callbacks → Vec<FsEvent> → Rust closure

Linux:
paths (&[&str]) + since_event_id + latency
        │
        ▼
 EventStream::new(...)
        │   (initialize inotify, add watches for paths)
        ▼
 EventStream::spawn()
        │   (spawn thread to read inotify events,
        │    batch events according to latency)
        ▼
 EventStreamHandle
        │
        └─ incoming inotify events → Vec<FsEvent> → Rust closure
```

---

## EventWatcher

`EventWatcher` is the high-level type used by the rest of the codebase:

- Holds:
  - `receiver: Receiver<Vec<FsEvent>>` — batches of events.
  - `_cancellation_token: Sender<()>` — used to end the watcher thread.
- Implements `Deref<Target = Receiver<Vec<FsEvent>>>` so callers can use `recv` or `select!` directly.

### Spawning a watcher

```rust
let (dev, watcher) = EventWatcher::spawn(path, since_event_id, latency);
```

- Creates a bounded cancellation channel and an unbounded events channel.
- Builds an `EventStream` with a callback that sends event batches into the `sender`.
- Spawns a thread that:
  - **macOS**: Calls `stream.spawn()` to start the FSEvent stream attached to a dispatch queue.
  - **Linux**: Calls `stream.spawn()` to start the inotify event monitoring thread.
  - Blocks on the cancellation receiver, keeping the stream active until dropped.

`EventWatcher::noop()` returns a watcher with a disconnected receiver and a dummy cancellation token, used when rescans are cancelled or disabled.

---

## Tests and guarantees

The `drop_then_respawn_event_watcher_delivers_events` test ensures:

- Dropping an `EventWatcher` properly tears down the underlying FSEvent stream.
- A new watcher on the same path receives fresh events as expected.

This behavior is critical for rescan flows and for cases where the watcher must be restarted after errors.

---

## Integration notes

- The Tauri backend uses `EventWatcher::spawn` to:
  - Feed `SearchCache::handle_fs_events` with live changes.
  - Trigger `perform_rescan` when filesystem event flags or paths suggest the index may be out of sync.
  - Compute a minimal set of paths to rescan (`scan_paths`) based on `ScanType` and ancestry rather than trusting individual "create/delete/modify" flags, which can arrive in unexpected combinations.
- `current_event_id()` is used to:
  - Capture the event ID at the time of a full walk, so subsequent watchers can start from a known boundary.
  - Advance `last_event_id` in the cache as events are processed.
- Platform-specific behavior:
  - **macOS**: Uses FSEvents for comprehensive filesystem monitoring with rich metadata.
  - **Linux**: Uses inotify for filesystem monitoring, mapping inotify events to equivalent FSEvent flags.
