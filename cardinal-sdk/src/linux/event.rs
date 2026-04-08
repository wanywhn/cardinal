use crate::{EventFlag, ScanType};
use std::{
    path::{Path, PathBuf},
};

/// Linux 文件系统事件
/// 
/// 注意：Linux 使用 inotify 实现，不支持历史事件回放。
/// 事件 ID 仅为简单计数器，应用重启后从 0 开始。
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
    /// 判断是否需要触发完整重新扫描
    /// 
    /// 在 Linux 下，由于没有历史回放，此方法主要用于判断
    /// 根目录变化或其他需要重新扫描的情况。
    pub fn should_rescan(&self, root: &Path) -> bool {
        match self.flag.scan_type() {
            ScanType::ReScan => true,
            ScanType::SingleNode | ScanType::Folder if self.path == root => true,
            ScanType::SingleNode | ScanType::Folder | ScanType::Nop => false,
        }
    }
}