#[cfg(target_os = "macos")]
mod event;
#[cfg(target_os = "macos")]
mod event_flag;
#[cfg(target_os = "macos")]
mod event_stream;
#[cfg(target_os = "macos")]
mod utils;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as event;
#[cfg(target_os = "linux")]
use linux as event_flag;
#[cfg(target_os = "linux")]
use linux as event_stream;
#[cfg(target_os = "linux")]
use linux as utils;

pub use event::FsEvent;
pub use event_flag::{EventFlag, EventType, ScanType};
#[cfg(target_os = "macos")]
pub use objc2_core_services::FSEventStreamEventId;
#[cfg(target_os = "linux")]
pub type FSEventStreamEventId = u64; // Use u64 as equivalent type for Linux
pub use event_stream::{EventStream, EventWatcher};
pub use utils::{current_event_id, event_id_to_timestamp};
