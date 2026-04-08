//! Linux 文件系统事件监控实现
//!
//! 本模块使用 inotify 实现文件系统事件监控，作为 macOS FSEvents 的跨平台替代。
//!
//! # 重要限制
//!
//! 与 macOS FSEvents 相比，inotify 有以下限制：
//!
//! 1. **不支持历史事件回放**：inotify 只监控未来的事件，无法恢复上次监控之后的事件。
//!    应用启动时必须完整重新扫描文件系统。
//!
//! 2. **无全局事件 ID**：使用原子计数器模拟事件 ID，每次应用启动从 0 开始。
//!
//! 3. **不递归监控子目录**：需要为每个子目录单独添加 watch（当前实现未处理此问题）。
//!
//! 4. **设备 ID 不可用**：`dev()` 返回 0 作为占位符。
//!
//! # 可用功能
//!
//! - ✅ 运行期间的实时文件事件监控
//! - ✅ 增量更新搜索缓存（创建、修改、删除、重命名）
//! - ✅ 与 macOS 兼容的事件类型和扫描类型判断

mod event;
mod event_flag;
mod event_stream;
mod utils;

pub use event::FsEvent;
pub use event_flag::{EventFlag, EventType, ScanType};
pub use event_stream::{EventStream, EventWatcher};
pub use utils::{current_event_id, event_id_to_timestamp};