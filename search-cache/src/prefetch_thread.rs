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
    query_preprocessor::{expand_query_home_dirs, strip_query_quotes},
};
use cardinal_syntax::{Expr, parse_query, optimize_query};
use search_cancel::CancellationToken;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use tracing::{debug};

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
    ) -> Self {
        Self {
            receiver,
            _handle: Some(handle),
            prefetch_done: false,
            buffer: Vec::new(),
            buffer_pos: 0,
            cancelled,
            background_thread_done,
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
) -> PrefetchState {
    // 锁定缓存获取必要信息
    let cache_guard = shared_cache.read().unwrap();

    // 解析查询
    let parsed = parse_query(query)
        .expect("Failed to parse query");
    let expanded = expand_query_home_dirs(parsed);
    let unquoted = strip_query_quotes(expanded);
    let optimized = optimize_query(unquoted);

    let total_nodes = cache_guard.get_total_files();
    let root_index = cache_guard.file_nodes.root();

    debug!(
        "Prefetch thread: query='{}', total_nodes={}, batch_size={}",
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

    let handle = std::thread::spawn(move || {
        debug!("Prefetch thread started for iterator");
        let visit_time = Instant::now();
        // 后台线程执行完整的搜索遍历
        let mut traversal_stack = vec![root_index];
        let mut current_pos = 0;
        let mut batch_buffer = Vec::with_capacity(batch_size);
        let mut last_log_pos = 0;

        loop {
            // 检查取消
            if cancel_token.is_cancelled_sparse(current_pos).is_none()
                || cancelled_flag.load(Ordering::Relaxed)
            {
                debug!("Prefetch thread cancelled at pos={}", current_pos);
                let _ = tx_for_thread.send(PrefetchMessage::Cancelled);
                break;
            }

            // 获取下一个节点
            let Some(current_index) = traversal_stack.pop() else {
                // 遍历完成
                if !batch_buffer.is_empty() {
                    let _ = tx_for_thread.send(PrefetchMessage::Batch(batch_buffer));
                }
                let _ = tx_for_thread.send(PrefetchMessage::Done);
                debug!("Prefetch thread completed, total_pos={}", current_pos);
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
                if match_node(&optimized.expr, node_name, options.case_insensitive) {
                    batch_buffer.push(current_index);
                }
                current_pos += 1;

                // 每遍历 10 万个节点打印一次进度
                if current_pos - last_log_pos >= 100000 {
                    debug!("Prefetch thread progress: pos={}/{}, matched: {}", current_pos, total_nodes, batch_buffer.len());
                    last_log_pos = current_pos;
                }
            }

            // 将子节点压入栈（需要克隆，因为 node 是借用）
            let children: Vec<_> = node.children.iter().copied().collect();
            drop(cache_guard); // 释放锁

            for &child_index in children.iter().rev() {
                traversal_stack.push(child_index);
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
                debug!("Prefetch thread completed, total_pos={}", current_pos);
                // 标记后台线程完成
                background_thread_done_clone.store(true, Ordering::Relaxed);
                break;
            }
        }
        debug!("Prefetch thread search time: {:?}", visit_time.elapsed());
    });

    PrefetchState::new(rx, tx, handle, cancelled_clone, background_thread_done)
}

/// 辅助函数：匹配节点（用于后台线程）
fn match_node(expr: &Expr, node_name: &str, case_insensitive: bool) -> bool {
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
        Expr::Not(inner) => !match_node(inner, node_name, case_insensitive),
        Expr::And(parts) => parts.iter().all(|p| match_node(p, node_name, case_insensitive)),
        Expr::Or(parts) => parts.iter().any(|p| match_node(p, node_name, case_insensitive)),
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
    fn test_match_node_simple() {
        use cardinal_syntax::{parse_query, optimize_query};
        
        let parsed = parse_query("test").unwrap();
        let optimized = optimize_query(parsed);
        
        assert!(match_node(&optimized.expr, "test_file", false));
        assert!(match_node(&optimized.expr, "my_test", false));
        assert!(!match_node(&optimized.expr, "other", false));
    }

    #[test]
    fn test_match_node_case_insensitive() {
        use cardinal_syntax::{parse_query, optimize_query};
        
        let parsed = parse_query("TEST").unwrap();
        let optimized = optimize_query(parsed);
        
        assert!(match_node(&optimized.expr, "test_file", true));
        assert!(match_node(&optimized.expr, "TEST_FILE", true));
    }
}
