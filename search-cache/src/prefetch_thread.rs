// SPDX-FileCopyrightText: 2026 Cardinal-Qt Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 后台预取线程模块
//!
//! 纯后台遍历模式：
//! - 后台线程遍历文件树，通过通道发送结果
//! - next_batch 从通道接收结果
//! - 无惰性遍历，只有一个遍历源

use crate::{
    SearchCache, SearchOptions, SlabIndex,
    build_segment_matchers, segment::SegmentMatcher,
    query_preprocessor::{expand_query_home_dirs, strip_query_quotes},
};
use cardinal_syntax::{Expr, parse_query, optimize_query};
use query_segmentation::query_segmentation;
use search_cancel::CancellationToken;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use tracing::{debug};

/// 搜索结果数量回调函数类型（私有，仅在模块内部使用）
type SearchResultNumCallback = Arc<dyn Fn(i64) + Send + Sync>;

/// 预取状态（用于后台预取线程管理）
pub struct PrefetchState {
    /// 预取结果接收端
    pub receiver: Receiver<PrefetchMessage>,
    /// 后台线程句柄
    pub _handle: Option<std::thread::JoinHandle<()>>,
    /// 预取是否完成
    pub prefetch_done: bool,
    /// 预取缓冲区（已接收但未返回的结果）
    pub buffer: Vec<SlabIndex>,
    /// 缓冲区读取位置
    pub buffer_pos: usize,
    /// 预取是否被取消
    #[allow(dead_code)]
    pub cancelled: Arc<AtomicBool>,
    /// 后台遍历线程是否完成
    pub background_thread_done: Arc<AtomicBool>,
    /// 搜索完成回调（通过 RwLock 允许后期设置）
    pub on_search_complete: Arc<RwLock<Option<SearchResultNumCallback>>>,
}

/// 预取消息类型
pub enum PrefetchMessage {
    /// 一批搜索结果
    Batch(Vec<SlabIndex>),
    /// 搜索完成
    Done,
    /// 搜索被取消
    Cancelled,
}

impl PrefetchState {
    pub fn new(
        receiver: Receiver<PrefetchMessage>,
        _sender: Sender<PrefetchMessage>,
        handle: std::thread::JoinHandle<()>,
        cancelled: Arc<AtomicBool>,
        background_thread_done: Arc<AtomicBool>,
        on_search_complete: Arc<RwLock<Option<SearchResultNumCallback>>>,
    ) -> Self {
        Self {
            receiver,
            _handle: Some(handle),
            prefetch_done: false,
            buffer: Vec::new(),
            buffer_pos: 0,
            cancelled,
            background_thread_done,
            on_search_complete,
        }
    }

    /// 阻塞接收一批数据（带超时）
    pub fn recv_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Option<PrefetchMessage> {
        match self.receiver.recv_timeout(timeout) {
            Ok(msg) => Some(msg),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.prefetch_done = true;
                Some(PrefetchMessage::Done)
            }
        }
    }
}

/// 启动后台预取线程（使用 Arc<RwLock<SearchCache>>）
pub fn start_prefetch_thread_rwlock(
    shared_cache: Arc<RwLock<SearchCache>>,
    query: &str,
    options: SearchOptions,
    batch_size: usize,
    cancel_token: CancellationToken,
    on_search_complete: Arc<RwLock<Option<SearchResultNumCallback>>>,
) -> PrefetchState {
    // 锁定缓存获取必要信息
    let cache_guard = shared_cache.read().unwrap();

    // 解析查询
    let parsed = parse_query(query)
        .expect("Failed to parse query");
    let expanded = expand_query_home_dirs(parsed);
    let unquoted = strip_query_quotes(expanded);
    let optimized = optimize_query(unquoted);

    // 只有当查询包含路径分隔符时，才构建路径段匹配器
    let path_matchers = if query.contains('/') {
        build_segment_matchers(&query_segmentation(query), options).ok()
    } else {
        None
    };

    let total_nodes = cache_guard.get_total_files();
    let root_index = cache_guard.file_nodes.root();

    debug!(
        "Prefetch thread started for iterator: query='{}', total_nodes={}, batch_size={}",
        query, total_nodes, batch_size
    );

    // 释放锁后再启动后台线程
    drop(cache_guard);

    let (tx, rx) = mpsc::channel();
    let cancelled_flag = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled_flag.clone();
    let background_thread_done = Arc::new(AtomicBool::new(false));
    let background_thread_done_clone = background_thread_done.clone();

    // Sender 可以克隆，克隆的发送者都发送到同一个通道
    let tx_for_thread = tx.clone();

    // 克隆回调存储，以便在后台线程和 PrefetchState 中共享
    let on_search_complete_clone = Arc::clone(&on_search_complete);

    let handle = std::thread::spawn(move || {
        debug!("Prefetch thread started for iterator");
        let visit_time = Instant::now();
        // 后台线程执行完整的搜索遍历
        let mut traversal_stack = vec![(root_index, Vec::<String>::new())];  // (节点索引，路径段)
        let mut current_pos = 0;
        let mut batch_buffer = Vec::with_capacity(batch_size);
        let mut last_log_pos = 0;
        let mut matched_count = 0;

        loop {
            // 检查取消
            if cancel_token.is_cancelled_sparse(current_pos).is_none()
                || cancelled_flag.load(Ordering::Relaxed)
            {
                debug!("Prefetch thread cancelled at pos={}", current_pos);
                let _ = tx_for_thread.send(PrefetchMessage::Cancelled);
                // 调用回调通知搜索结果数量
                let callback_guard = on_search_complete_clone.read().unwrap();
                if let Some(ref callback) = *callback_guard {
                    callback(matched_count as i64);
                }
                break;
            }

            // 获取下一个节点
            let Some((current_index, current_path)) = traversal_stack.pop() else {
                // 遍历完成
                if !batch_buffer.is_empty() {
                    let _ = tx_for_thread.send(PrefetchMessage::Batch(batch_buffer));
                }
                let _ = tx_for_thread.send(PrefetchMessage::Done);
                debug!("Prefetch thread completed, total_pos={}, matched={}", current_pos, matched_count);
                // 调用回调通知搜索结果数量
                let callback_guard = on_search_complete_clone.read().unwrap();
                if let Some(ref callback) = *callback_guard {
                    callback(matched_count as i64);
                }
                // 标记后台线程完成
                background_thread_done_clone.store(true, Ordering::Relaxed);
                break;
            };

            // 使用读锁访问缓存（允许多个读取者并发）
            let cache_guard = shared_cache.read().unwrap();
            let Some(node) = cache_guard.file_nodes.get(current_index) else {
                drop(cache_guard); // 尽早释放锁
                continue;
            };

            // 跳过根节点，但遍历其子节点
            if current_index != root_index {
                // 匹配检查
                let node_name: &str = node.name();
                let matches = match_node_with_path(&optimized.expr, node_name, &current_path, path_matchers.as_deref(), options.case_insensitive);
                if matches {
                    batch_buffer.push(current_index);
                    matched_count += 1;
                    debug!("Matched node: path={:?}, name={}", current_path, node_name);
                }
                current_pos += 1;

                // 每遍历 10 万个节点打印一次进度
                if current_pos - last_log_pos >= 100000 {
                    debug!("Prefetch thread progress: pos={}/{}, matched: {}", current_pos, total_nodes, matched_count);
                    last_log_pos = current_pos;
                }
            }

            // 将子节点压入栈（需要克隆路径）
            let children: Vec<_> = node.children.iter().copied().collect();
            let node_name_owned = node.name().to_string();
            drop(cache_guard); // 释放锁

            // 为每个子节点构建新的路径
            for &child_index in children.iter().rev() {
                let mut child_path = current_path.clone();
                child_path.push(node_name_owned.clone());
                traversal_stack.push((child_index, child_path));
            }

            // 达到批处理大小时发送
            if batch_buffer.len() >= batch_size {
                let buffer_to_send = std::mem::replace(&mut batch_buffer, Vec::with_capacity(batch_size));
                if tx_for_thread.send(PrefetchMessage::Batch(buffer_to_send)).is_err() {
                    // 接收端已断开，停止
                    debug!("Prefetch thread: channel disconnected, stopping");
                    break;
                }
            }

            // 检查是否遍历完所有节点
            if current_pos >= total_nodes - 1 && traversal_stack.is_empty() {
                if !batch_buffer.is_empty() {
                    let _ = tx_for_thread.send(PrefetchMessage::Batch(batch_buffer));
                }
                let _ = tx_for_thread.send(PrefetchMessage::Done);
                debug!("Prefetch thread completed, total_pos={}, matched={}", current_pos, matched_count);
                // 调用回调通知搜索结果数量
                let callback_guard = on_search_complete_clone.read().unwrap();
                if let Some(ref callback) = *callback_guard {
                    callback(matched_count as i64);
                }
                // 标记后台线程完成
                background_thread_done_clone.store(true, Ordering::Relaxed);
                break;
            }
        }
        debug!("Prefetch thread search time: {:?}", visit_time.elapsed());
    });

    PrefetchState::new(rx, tx, handle, cancelled_clone, background_thread_done, on_search_complete)
}

/// 辅助函数：匹配节点（支持路径段匹配）
///
/// # 参数
/// - `expr`: 查询表达式
/// - `node_name`: 当前文件名
/// - `path_segments`: 从根到当前节点的路径段（不包含当前文件名）
/// - `path_matchers`: 路径段匹配器（如果查询包含路径段）
/// - `case_insensitive`: 是否大小写不敏感
fn match_node_with_path(
    expr: &Expr,
    node_name: &str,
    path_segments: &[String],
    path_matchers: Option<&[SegmentMatcher]>,
    case_insensitive: bool,
) -> bool {
    // 如果有路径段匹配器，使用路径匹配
    if let Some(matchers) = path_matchers {
        match_path_segments(matchers, path_segments, node_name)
    } else {
        // 否则使用基础文件名匹配
        match_node_basic(expr, node_name, case_insensitive)
    }
}

/// 匹配路径段
///
/// 支持相对路径匹配：查询 `*oo/bar` 应该匹配任何路径中包含连续段 `*oo` -> `bar` 的节点
/// 复用 query.rs 中的路径段匹配逻辑
fn match_path_segments(
    matchers: &[SegmentMatcher],
    path_segments: &[String],
    node_name: &str,
) -> bool {
    // 构建完整的路径段列表（包含文件名）
    let mut full_path: Vec<&str> = path_segments.iter().map(|s| s.as_str()).collect();
    full_path.push(node_name);
    
    // 相对路径匹配：查找是否有连续的段匹配所有匹配器
    // 使用滑动窗口方式检查所有可能的起始位置
    for start_idx in 0..full_path.len() {
        if match_from_position(matchers, &full_path, start_idx) {
            return true;
        }
    }
    
    false
}

/// 从指定位置开始匹配路径段
fn match_from_position(
    matchers: &[SegmentMatcher],
    full_path: &[&str],
    start_idx: usize,
) -> bool {
    let mut path_idx = start_idx;
    let mut matcher_idx = 0;
    let mut pending_globstar = false;
    
    while matcher_idx < matchers.len() {
        if path_idx >= full_path.len() {
            // 路径已用完，检查是否还有 pending_globstar
            return pending_globstar;
        }
        
        match &matchers[matcher_idx] {
            SegmentMatcher::GlobStar => {
                pending_globstar = true;
                matcher_idx += 1;
            }
            SegmentMatcher::Star => {
                // * 匹配单个路径段
                path_idx += 1;
                matcher_idx += 1;
                pending_globstar = false;
            }
            SegmentMatcher::Concrete(concrete) => {
                if pending_globstar {
                    // ** 后可以匹配任意数量的路径段
                    // 查找是否有任意后续路径段匹配
                    let mut found = false;
                    for i in path_idx..full_path.len() {
                        if concrete.matches(full_path[i]) {
                            path_idx = i + 1;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return false;
                    }
                    pending_globstar = false;
                } else {
                    // 按顺序匹配
                    if !concrete.matches(full_path[path_idx]) {
                        return false;
                    }
                    path_idx += 1;
                }
                matcher_idx += 1;
            }
        }
    }
    
    // 所有匹配器都已匹配
    // 如果还有 pending_globstar，匹配剩余所有路径段（总是成功）
    // 否则，必须正好匹配到路径末尾
    pending_globstar || path_idx == full_path.len()
}

/// 基础文件名匹配（不支持路径段）
fn match_node_basic(expr: &Expr, node_name: &str, case_insensitive: bool) -> bool {
    match expr {
        Expr::Empty => true,
        Expr::Term(term) => {
            let search_text = if case_insensitive {
                node_name.to_lowercase()
            } else {
                node_name.to_string()
            };

            match term {
                cardinal_syntax::Term::Word(text) => {
                    let pattern = if case_insensitive {
                        text.to_lowercase()
                    } else {
                        text.clone()
                    };
                    if pattern.contains('*') || pattern.contains('?') {
                        match_wildcard_static(&search_text, &pattern)
                    } else {
                        search_text.contains(&pattern)
                    }
                }
                cardinal_syntax::Term::Filter(filter) => {
                    match &filter.kind {
                        cardinal_syntax::FilterKind::Ext => {
                            if let Some(arg) = &filter.argument {
                                let ext = if case_insensitive {
                                    arg.raw.to_lowercase()
                                } else {
                                    arg.raw.clone()
                                };
                                node_name.ends_with(&format!(".{}", ext))
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                }
                cardinal_syntax::Term::Regex(_) => false,
            }
        }
        Expr::Not(inner) => !match_node_basic(inner, node_name, case_insensitive),
        Expr::And(parts) => parts.iter().all(|p| match_node_basic(p, node_name, case_insensitive)),
        Expr::Or(parts) => parts.iter().any(|p| match_node_basic(p, node_name, case_insensitive)),
    }
}

/// 通配符匹配（静态版本，用于后台线程）
fn match_wildcard_static(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let regex_pattern = pattern
        .replace('.', "\\.")
        .replace('*', ".*")
        .replace('?', ".");
    if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
        return regex.is_match(text);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_node_basic_simple() {
        use cardinal_syntax::{parse_query, optimize_query};

        let parsed = parse_query("test").unwrap();
        let optimized = optimize_query(parsed);

        assert!(match_node_basic(&optimized.expr, "test_file", false));
        assert!(match_node_basic(&optimized.expr, "my_test", false));
        assert!(!match_node_basic(&optimized.expr, "other", false));
    }

    #[test]
    fn test_match_node_basic_case_insensitive() {
        use cardinal_syntax::{parse_query, optimize_query};

        let parsed = parse_query("TEST").unwrap();
        let optimized = optimize_query(parsed);

        assert!(match_node_basic(&optimized.expr, "test_file", true));
        assert!(match_node_basic(&optimized.expr, "TEST_FILE", true));
    }

    #[test]
    fn test_match_path_segments_simple() {
        // 测试简单路径匹配：foo/bar
        let matchers = build_segment_matchers(
            &query_segmentation("foo/bar"),
            SearchOptions::default(),
        ).unwrap();
        
        // 匹配：路径 ["foo"] + 文件名 "bar"
        assert!(match_path_segments(&matchers, &["foo".to_string()], "bar"));
        
        // 不匹配：路径 ["baz"] + 文件名 "bar"
        assert!(!match_path_segments(&matchers, &["baz".to_string()], "bar"));
        
        // 不匹配：路径 ["foo"] + 文件名 "baz"
        assert!(!match_path_segments(&matchers, &["foo".to_string()], "baz"));
    }

    #[test]
    fn test_match_path_segments_globstar() {
        // 测试 globstar: foo/**/bar
        let matchers = build_segment_matchers(
            &query_segmentation("foo/**/bar"),
            SearchOptions::default(),
        ).unwrap();
        
        // 匹配：foo/bar (globstar 匹配 0 个段)
        assert!(match_path_segments(&matchers, &["foo".to_string()], "bar"));
        
        // 匹配：foo/x/bar (globstar 匹配 1 个段)
        assert!(match_path_segments(&matchers, &["foo".to_string(), "x".to_string()], "bar"));
        
        // 匹配：foo/x/y/bar (globstar 匹配 2 个段)
        assert!(match_path_segments(&matchers, &["foo".to_string(), "x".to_string(), "y".to_string()], "bar"));
        
        // 不匹配：baz/x/bar (第一段不匹配)
        assert!(!match_path_segments(&matchers, &["baz".to_string(), "x".to_string()], "bar"));
    }

    #[test]
    fn test_match_path_segments_wildcard() {
        // 测试通配符：*.rs
        let matchers = build_segment_matchers(
            &query_segmentation("*.rs"),
            SearchOptions::default(),
        ).unwrap();
        
        // 匹配：任何 .rs 文件
        assert!(match_path_segments(&matchers, &[], "test.rs"));
        assert!(match_path_segments(&matchers, &[], "lib.rs"));
        
        // 不匹配：非 .rs 文件
        assert!(!match_path_segments(&matchers, &[], "test.txt"));
    }

    #[test]
    fn test_match_node_with_path() {
        use cardinal_syntax::{parse_query, optimize_query};

        let parsed = parse_query("test").unwrap();
        let optimized = optimize_query(parsed);

        // 无路径段匹配器，仅文件名匹配
        assert!(match_node_with_path(&optimized.expr, "test_file", &[], None, false));
        assert!(match_node_with_path(&optimized.expr, "my_test", &[], None, false));
        assert!(!match_node_with_path(&optimized.expr, "other", &[], None, false));
    }

    #[test]
    fn test_match_path_with_directory() {
        use crate::SearchCache;
        use tempdir::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new("test_match_path_with_directory").unwrap();
        let dir = temp_dir.path();
        fs::create_dir_all(dir.join("foo/bar")).unwrap();
        
        let cache = SearchCache::walk_fs(dir);
        
        // 验证缓存包含预期的节点
        assert!(cache.get_total_files() >= 2);
    }
}
