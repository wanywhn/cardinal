# 优化待办

## 高优先级
- **为 QuickLook 图标复用现有节点缓存**  
  `cardinal/src-tauri/src/lib.rs` 中的 `cache.expand_file_nodes(&viewport)` 会重新抓取元数据。可以新增轻量接口仅返回路径，复用 `get_nodes_info` 的结果，减少磁盘 IO。
- **节流 icon viewport IPC**  
  `cardinal/src/components/VirtualList.jsx` 在每次滚动时都会 `invoke('update_icon_viewport')`，频繁跨进程。可用 `requestAnimationFrame` 或最小时间窗口批量发送，减少 Tauri IPC 开销并成批触发 QuickLook 生成。

## 中优先级
- **批量调度 icon 生成**  
  当前对每个视窗节点单独 `rayon::spawn`，Viewport 大时会产生大量任务，可换成 `icon_jobs.into_par_iter()` 或固定线程池批量处理，降低调度开销。
- **FsEvent 扫描路径去重算法优化**  
  `search-cache/src/cache.rs::scan_paths` 在最坏情况下 O(n²)。可按路径深度排序后线性合并，并用栈结构取代频繁的 `retain`。
- **NamePool 检索结构升级**  
  `namepool/src/lib.rs` 里的 `BTreeSet` + `Mutex` 每次查询都会全表扫描并复制集合。考虑换成 `RwLock` + `fst`/前缀树，或为常见前缀/后缀维护辅助索引，可显著加速模糊搜索并减少锁竞争。
