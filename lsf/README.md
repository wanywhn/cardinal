# lsf

一个用于本地文件系统“索引 + 实时增量更新 + 交互式查询”的命令行工具（Cardinal 项目的子模块）。

- 首次运行会遍历指定目录构建搜索缓存（SearchCache）。
- 后续通过文件系统事件监听（cardinal-sdk 提供）持续增量更新索引。
- 提供交互式 REPL：输入查询字符串，返回匹配的文件条目（路径与元数据）。
- 退出时将索引持久化到本地文件以加速下次启动。

## 功能特点
- 全量扫描 + 增量更新：首次 `walk_fs`，运行中监听事件并更新，必要时自动触发重扫（rescan）。
- 交互式检索：逐行输入关键字，返回命中的文件节点（含 path 与 metadata）。
- 缓存持久化：退出时写入压缩文件，启动时优先读取缓存以减少等待。
- 日志可控：支持 `RUST_LOG` 环境变量或默认 INFO 级别。

## 构建要求
- Rust 工具链：仓库根目录固定为 `nightly-2025-05-09`（见 `rust-toolchain.toml`）。
- 平台：开发在 macOS 上（事件监听通常依赖 FSEvents）。

## 快速开始

- 构建（从仓库根目录）：
```bash
cargo build -p lsf
```

- 运行（从仓库根目录，传参给可执行需使用 `--` 分隔）：
```bash
# 建议指定一个较小的根目录，避免默认“/”耗时过长
cargo run -p lsf -- --path $HOME --refresh
```
说明：
- `--refresh` 忽略旧缓存并重新全量遍历。
- 不加 `--refresh` 时，程序会尝试读取上次的持久化缓存，失败则回退到全量遍历。

- 启用日志（可选）：
```bash
RUST_LOG=info cargo run -p lsf -- --path $HOME
```
支持的常见级别：`error,warn,info,debug,trace`。

## 命令行参数
- `--refresh`（默认 false）：忽略已有缓存，强制重新遍历文件系统。
- `--path <PATH>`（默认 "/"）：索引的根目录。

示例：
```bash
cargo run -p lsf -- --path ~/Projects --refresh
```

## 交互使用
- 启动后进入 REPL，提示符为 `>`。
- 输入任意查询字符串并回车，打印命中的条目：
  - 输出格式：`[索引] <Path> <Metadata>`
- 输入 `/bye` 退出程序。

## 缓存与持久化
- 缓存文件路径（相对运行目录）：
  - `target/cache.zstd`（临时写入 `target/cache.zstd.tmp` 后原子替换）
- 编解码：`bincode` + `zstd`（多线程压缩）。
- 缓存内容包含：版本号、最后事件 ID、根路径、索引根节点、节点 Slab、名称反向索引等。

## 运行时行为概览
- 启动时：
  1) 尝试从缓存恢复；若失败或 `--refresh`，则全量遍历 `--path`。
  2) 启动事件监听（`EventWatcher::spawn`）。
  3) 创建若干通道处理查询与结束信号。
- 运行中：
  - 监听文件变更事件并调用 `cache.handle_fs_events` 增量更新。
  - 当检测到需要重扫（`HandleFSEError::Rescan`）时，清空事件并执行 `cache.rescan()`。
  - 主线程处理 REPL 输入，将查询发到后台并打印结果。
- 退出时：
  - 通过通道拿回最终 `SearchCache`，调用 `flush_to_file()` 写回缓存。

## 注意事项
- 默认 `--path` 是根目录 “/”，首次扫描可能非常耗时；建议指定家目录或项目目录。
- 目前实现为 CLI 交互模式；未来可能提供 TUI 与更细粒度的查询缓存。
- 文件系统事件在不同平台的能力差异可能影响增量更新体验。

## 相关子模块
- `search-cache`：索引结构、查询与持久化（`target/cache.zstd`）。
- `cardinal-sdk`：文件系统事件监听（`EventWatcher`）。
- `fswalk`：文件系统遍历。
- `namepool`、`query-segmentation`：名称池与查询分段等工具。

## 开发提示
- 在根目录用 `RUST_LOG` 调整日志噪音，便于观察重扫触发、缓存读写耗时：
```bash
RUST_LOG=info,lsf=debug cargo run -p lsf -- --path $HOME
```
- 如需验证持久化，可多次运行并对比“Try reading cache…”与“Walking filesystem…”的启动路径。

## 未来计划（代码中 TODO 摘要）
- 查询分段与查询结果缓存（前缀/后缀/精确）。
- 可能的 TUI 界面。
- 懒加载元数据（空闲时补全，支持中断续作）。
- 探索“已删除文件”历史检索能力。

---
如需在更大项目中集成，可将其作为本地文件搜索后端，结合 GUI/TUI 或服务化接口使用。
