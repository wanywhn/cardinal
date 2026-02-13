use crate::{EventFlag, ScanType};
use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FsEvent {
    /// The path of this event.
    pub path: PathBuf,
    /// The event type.
    pub flag: EventFlag,
    /// The event id.
    pub id: u64,
}

impl FsEvent {
    pub(crate) unsafe fn from_raw(path: *const i8, flag: u32, id: u64) -> Self {
        let path = unsafe { CStr::from_ptr(path) };
        let path = OsStr::from_bytes(path.to_bytes());
        let path = PathBuf::from(path);
        let flag = EventFlag::from_bits_truncate(flag);
        FsEvent { path, flag, id }
    }

    pub fn should_rescan(&self, root: &Path) -> bool {
        match self.flag.scan_type() {
            ScanType::ReScan => true,
            ScanType::SingleNode | ScanType::Folder if self.path == root => true,
            ScanType::SingleNode | ScanType::Folder | ScanType::Nop => false,
        }
    }
}