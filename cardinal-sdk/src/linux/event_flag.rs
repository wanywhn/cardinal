use bitflags::bitflags;

// Linux 事件标志定义
// 注意：这些标志是为了与 macOS FSEvents 兼容而定义的。
// Linux 使用 inotify 实现，某些标志（如 Hardlink、Cloned）在 inotify 中没有对应概念。
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventFlag: u32 {
        const None = 0;
        const MustScanSubDirs = 1 << 0;
        const UserDropped = 1 << 1;
        const KernelDropped = 1 << 2;
        const EventIdsWrapped = 1 << 3;
        const HistoryDone = 1 << 4;  // 在 Linux 下不会产生此事件
        const RootChanged = 1 << 5;
        const Mount = 1 << 6;
        const Unmount = 1 << 7;
        const ItemCreated = 1 << 8;
        const ItemRemoved = 1 << 9;
        const ItemInodeMetaMod = 1 << 10;
        const ItemRenamed = 1 << 11;
        const ItemModified = 1 << 12;
        const ItemFinderInfoMod = 1 << 13;  // 在 Linux 下不会产生此事件
        const ItemChangeOwner = 1 << 14;
        const ItemXattrMod = 1 << 15;
        const ItemIsFile = 1 << 16;
        const ItemIsDir = 1 << 17;
        const ItemIsSymlink = 1 << 18;
        const OwnEvent = 1 << 19;  // 在 Linux 下不会产生此事件
        const IsHardlink = 1 << 20;  // 在 Linux 下不会产生此事件
        const IsLastHardlink = 1 << 21;  // 在 Linux 下不会产生此事件
        const Cloned = 1 << 22;  // 在 Linux 下不会产生此事件
    }
}

impl EventFlag {
    pub fn from_inotify_mask(event: nix::sys::inotify::InotifyEvent) -> Self {
        let mut flags = EventFlag::empty();

        // 移除 IN_ACCESS 映射，避免索引扫描时读取元数据触发事件循环
        // 原代码: if event.mask.contains(IN_ACCESS) { flags.insert(ItemModified); }

        // 检查是否有任何有意义的事件
        let has_meaningful_event =
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MODIFY) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_ATTRIB) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_CLOSE_WRITE) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MOVED_FROM) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MOVED_TO) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_CREATE) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_DELETE) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_DELETE_SELF) ||
            event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MOVE_SELF);

        // 只有有意义的事件才设置类型标志
        if has_meaningful_event {
            if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_ISDIR) {
                flags.insert(EventFlag::ItemIsDir);
            } else {
                flags.insert(EventFlag::ItemIsFile);
            }
        }
        
        // 设置具体的事件类型
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MODIFY) {
            flags.insert(EventFlag::ItemModified);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_ATTRIB) {
            flags.insert(EventFlag::ItemInodeMetaMod);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_CLOSE_WRITE) {
            flags.insert(EventFlag::ItemModified);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MOVED_FROM) {
            flags.insert(EventFlag::ItemRenamed);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MOVED_TO) {
            flags.insert(EventFlag::ItemRenamed);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_CREATE) {
            flags.insert(EventFlag::ItemCreated);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_DELETE) {
            flags.insert(EventFlag::ItemRemoved);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_DELETE_SELF) {
            flags.insert(EventFlag::ItemRemoved);
        }
        if event.mask.contains(nix::sys::inotify::AddWatchFlags::IN_MOVE_SELF) {
            flags.insert(EventFlag::ItemRenamed);
        }

        flags
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        // 空标志或无意义事件返回 Nop
        if self.is_empty() ||
           self.contains(EventFlag::HistoryDone) ||
           self.contains(EventFlag::EventIdsWrapped) {
            ScanType::Nop
        } else if self.contains(EventFlag::RootChanged) {
            ScanType::ReScan
        } else {
            let event_type = self.event_type();
            let is_dir = matches!(event_type, EventType::Dir);
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