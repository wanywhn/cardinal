use crate::{EventFlag, ScanType};
use std::{
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

    pub fn should_rescan(&self, root: &Path) -> bool {
        match self.flag.scan_type() {
            ScanType::ReScan => true,
            ScanType::SingleNode | ScanType::Folder if self.path == root => true,
            ScanType::SingleNode | ScanType::Folder | ScanType::Nop => false,
        }
    }
}