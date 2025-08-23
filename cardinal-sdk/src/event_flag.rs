#![allow(non_upper_case_globals)]
use bitflags::bitflags;
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventFlag: u32 {
        const None = fsevent_sys::kFSEventStreamEventFlagNone;
        const MustScanSubDirs = fsevent_sys::kFSEventStreamEventFlagMustScanSubDirs;
        const UserDropped = fsevent_sys::kFSEventStreamEventFlagUserDropped;
        const KernelDropped = fsevent_sys::kFSEventStreamEventFlagKernelDropped;
        const EventIdsWrapped = fsevent_sys::kFSEventStreamEventFlagEventIdsWrapped;
        const HistoryDone = fsevent_sys::kFSEventStreamEventFlagHistoryDone;
        const RootChanged = fsevent_sys::kFSEventStreamEventFlagRootChanged;
        const Mount = fsevent_sys::kFSEventStreamEventFlagMount;
        const Unmount = fsevent_sys::kFSEventStreamEventFlagUnmount;
        const ItemCreated = fsevent_sys::kFSEventStreamEventFlagItemCreated;
        const ItemRemoved = fsevent_sys::kFSEventStreamEventFlagItemRemoved;
        const ItemInodeMetaMod = fsevent_sys::kFSEventStreamEventFlagItemInodeMetaMod;
        const ItemRenamed = fsevent_sys::kFSEventStreamEventFlagItemRenamed;
        const ItemModified = fsevent_sys::kFSEventStreamEventFlagItemModified;
        const ItemFinderInfoMod = fsevent_sys::kFSEventStreamEventFlagItemFinderInfoMod;
        const ItemChangeOwner = fsevent_sys::kFSEventStreamEventFlagItemChangeOwner;
        const ItemXattrMod = fsevent_sys::kFSEventStreamEventFlagItemXattrMod;
        const ItemIsFile = fsevent_sys::kFSEventStreamEventFlagItemIsFile;
        const ItemIsDir = fsevent_sys::kFSEventStreamEventFlagItemIsDir;
        const ItemIsSymlink = fsevent_sys::kFSEventStreamEventFlagItemIsSymlink;
        const OwnEvent = fsevent_sys::kFSEventStreamEventFlagOwnEvent;
        const IsHardlink = fsevent_sys::kFSEventStreamEventFlagItemIsHardlink;
        const IsLastHardlink = fsevent_sys::kFSEventStreamEventFlagItemIsLastHardlink;
        const Cloned = fsevent_sys::kFSEventStreamEventFlagItemCloned;
    }
}

pub enum EventType {
    Unknown,
    File,
    Dir,
    Symlink,
    Hardlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    /// Scan a single node
    SingleNode,
    /// Scan the whole folder, including sub-folders.
    Folder,
    /// Something wrong happened, do re-indexing.
    /// Should only happen with `kFSEventStreamCreateFlagWatchRoot` set in EventStream::new().
    ReScan,
    /// Do nothing, since event id is always updated.
    Nop,
}

impl EventFlag {
    pub fn event_type(&self) -> EventType {
        if self.contains(EventFlag::IsHardlink) | self.contains(EventFlag::IsLastHardlink) {
            EventType::Hardlink
        } else if self.contains(EventFlag::ItemIsSymlink) {
            EventType::Symlink
        } else if self.contains(EventFlag::ItemIsDir) {
            EventType::Dir
        } else if self.contains(EventFlag::ItemIsFile) {
            EventType::File
        } else {
            EventType::Unknown
        }
    }

    pub fn scan_type(&self) -> ScanType {
        let event_type = self.event_type();
        let is_dir = matches!(event_type, EventType::Dir);
        if self.contains(EventFlag::HistoryDone) {
            ScanType::Nop
        } else if self.contains(EventFlag::EventIdsWrapped) {
            ScanType::Nop
        } else if self.contains(EventFlag::RootChanged) {
            ScanType::ReScan
        } else {
            // Strange event, doesn't know when it happens, processing it using a generic way
            // e.g. new event: fs_event=FsEvent { path: "/.docid/16777229/changed/782/src=0,dst=41985052", flag: kFSEventStreamEventFlagNone, id: 471533015 }
            if is_dir {
                ScanType::Folder
            } else {
                ScanType::SingleNode
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_deduction() {
        assert!(matches!(
            (EventFlag::ItemIsFile).event_type(),
            EventType::File
        ));
        assert!(matches!(
            (EventFlag::ItemIsDir).event_type(),
            EventType::Dir
        ));
        assert!(matches!(
            (EventFlag::ItemIsSymlink).event_type(),
            EventType::Symlink
        ));
        assert!(matches!(
            (EventFlag::IsHardlink).event_type(),
            EventType::Hardlink
        ));
        // Unknown when no type bits set
        assert!(matches!((EventFlag::None).event_type(), EventType::Unknown));
    }

    #[test]
    fn test_scan_type_root_changed_and_history_done() {
        // RootChanged should always trigger ReScan regardless of type bits; test with RootChanged only.
        assert_eq!(EventFlag::RootChanged.scan_type(), ScanType::ReScan);
        assert_eq!(EventFlag::HistoryDone.scan_type(), ScanType::Nop);
    }

    #[test]
    fn test_scan_type_created_removed_modified() {
        // File create => SingleNode
        assert!(matches!(
            (EventFlag::ItemCreated | EventFlag::ItemIsFile).scan_type(),
            ScanType::SingleNode
        ));
        // Dir removal => Folder
        assert!(matches!(
            (EventFlag::ItemRemoved | EventFlag::ItemIsDir).scan_type(),
            ScanType::Folder
        ));
        // File removal => SingleNode
        assert!(matches!(
            (EventFlag::ItemRemoved | EventFlag::ItemIsFile).scan_type(),
            ScanType::SingleNode
        ));
        // File modified => SingleNode
        assert!(matches!(
            (EventFlag::ItemModified | EventFlag::ItemIsFile).scan_type(),
            ScanType::SingleNode
        ));
    }

    #[test]
    fn test_scan_type_must_scan_subdirs() {
        // MustScanSubDirs => Folder
        assert!(matches!(
            (EventFlag::MustScanSubDirs | EventFlag::ItemIsDir).scan_type(),
            ScanType::Folder
        ));
    }
}
