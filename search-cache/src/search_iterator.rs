// SPDX-FileCopyrightText: 2026 Cardinal-Qt Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 搜索迭代器模块
//!
//! 纯后台遍历模式：
//! - 后台线程遍历文件树，通过通道发送结果
//! - next_batch 从通道接收结果
//! - 无惰性遍历，只有一个遍历源

use crate::prefetch_thread::{
    PrefetchState, PrefetchMessage,
    start_prefetch_thread_rwlock,
};
use crate::{SearchCache, SearchOptions, SlabIndex};
use search_cancel::CancellationToken;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::info;

/// 搜索结果数量回调函数类型
type SearchResultNumCallback = Arc<dyn Fn(i64) + Send + Sync>;

/// 搜索迭代器状态
#[derive(Debug, Clone)]
pub struct IteratorState {
    /// 当前已返回的索引数量
    pub yielded_count: usize,
    /// 搜索是否已完成
    pub search_completed: bool,
    /// 搜索是否被取消
    pub cancelled: bool,
}

impl Default for IteratorState {
    fn default() -> Self {
        Self {
            yielded_count: 0,
            search_completed: false,
            cancelled: false,
        }
    }
}

/// 批次获取结果
#[derive(Debug, Clone)]
pub struct SearchBatch {
    /// 本批次索引列表
    pub indices: Vec<SlabIndex>,
    /// 是否还有更多数据
    pub has_more: bool,
    /// 搜索是否已完成
    pub search_completed: bool,
}

/// 搜索迭代器（内部实现）
///
/// 纯后台遍历模式：
/// - 后台线程遍历文件树，通过通道发送结果
/// - next_batch 从通道接收结果
/// - 无惰性遍历，只有一个遍历源
pub struct SearchIterator {
    /// 迭代器唯一 ID
    id: u64,
    /// 迭代器状态
    pub state: IteratorState,
    /// 总节点数
    total_nodes: usize,
    /// 预取状态（必须存在，用于后台遍历）
    prefetch_state: PrefetchState,
}

impl SearchIterator {
    /// 创建新的搜索迭代器（支持 Arc<RwLock<SearchCache>> 预取模式）
    ///
    /// 纯后台遍历模式：
    /// - 后台线程遍历文件树，通过通道发送结果
    /// - next_batch 从通道接收结果
    ///
    /// # 参数
    /// - `on_search_complete`: 搜索完成回调函数，当搜索完成或被取消时调用，传入搜索结果数量
    pub fn new_with_rwlock<F>(
        shared_cache: Arc<RwLock<SearchCache>>,
        query: &str,
        _options: SearchOptions,
        batch_size: usize,
        cancel_token: CancellationToken,
        on_search_complete: F,
    ) -> Result<Self, anyhow::Error>
    where
        F: Fn(i64) + Send + Sync + 'static,
    {
        // 锁定缓存获取必要信息（使用读锁，因为初始化不需要修改）
        let cache_guard = shared_cache.read().unwrap();

        let total_nodes = cache_guard.get_total_files();
        let id = generate_unique_id();

        info!(
            "Created SearchIterator (Arc<RwLock> mode): id={}, query='{}', total_nodes={}, batch_size={}",
            id, query, total_nodes, batch_size
        );

        // 释放锁后再启动后台线程
        drop(cache_guard);

        // 创建回调存储
        let on_search_complete: Arc<RwLock<Option<SearchResultNumCallback>>> = Arc::new(RwLock::new(Some(Arc::new(on_search_complete))));

        // 启动后台遍历线程
        let prefetch_state = start_prefetch_thread_rwlock(
            shared_cache,
            query,
            _options,
            batch_size,
            cancel_token,
            Arc::clone(&on_search_complete),
        );

        Ok(Self {
            id,
            state: IteratorState::default(),
            total_nodes,
            prefetch_state,
        })
    }

    /// 获取下一批结果（纯后台遍历模式）
    ///
    /// 从后台遍历线程的通道读取结果
    pub fn next_batch(
        &mut self,
        max_count: usize,
    ) -> SearchBatch {
        // 检查是否已取消
        if self.state.cancelled {
            return SearchBatch {
                indices: Vec::new(),
                has_more: false,
                search_completed: true,
            };
        }

        let mut result_indices = Vec::with_capacity(max_count);

        // 1. 首先从预取缓冲区返回已有的结果
        while result_indices.len() < max_count && self.prefetch_state.buffer_pos < self.prefetch_state.buffer.len() {
            result_indices.push(self.prefetch_state.buffer[self.prefetch_state.buffer_pos]);
            self.prefetch_state.buffer_pos += 1;
            self.state.yielded_count += 1;
        }

        // 2. 如果还需要更多数据且预取未完成，从通道接收
        while result_indices.len() < max_count && !self.prefetch_state.prefetch_done {
            if let Some(msg) = self.prefetch_state.recv_timeout(Duration::from_millis(100)) {
                match msg {
                    PrefetchMessage::Batch(indices) => {
                        self.prefetch_state.buffer = indices;
                        self.prefetch_state.buffer_pos = 0;
                        // 如果有结果，立即从缓冲区取数据
                        if !self.prefetch_state.buffer.is_empty() {
                            break;
                        }
                        // 如果为空，继续循环接收下一批
                    }
                    PrefetchMessage::Done => {
                        self.prefetch_state.prefetch_done = true;
                        self.state.search_completed = true;
                        break;
                    }
                    PrefetchMessage::Cancelled => {
                        self.prefetch_state.prefetch_done = true;
                        self.state.cancelled = true;
                        self.state.search_completed = true;
                        break;
                    }
                }
            }
        }

        // 3. 最后从预取缓冲区取数据
        while result_indices.len() < max_count && self.prefetch_state.buffer_pos < self.prefetch_state.buffer.len() {
            result_indices.push(self.prefetch_state.buffer[self.prefetch_state.buffer_pos]);
            self.prefetch_state.buffer_pos += 1;
            self.state.yielded_count += 1;
        }

        SearchBatch {
            indices: result_indices,
            has_more: !self.prefetch_state.prefetch_done || self.prefetch_state.buffer_pos < self.prefetch_state.buffer.len(),
            search_completed: self.state.search_completed,
        }
    }

    /// 获取迭代器 ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// 获取已返回数量
    pub fn yielded_count(&self) -> usize {
        self.state.yielded_count
    }

    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        self.state.search_completed
    }

    /// 检查后台遍历线程是否完成
    pub fn is_background_thread_done(&self) -> bool {
        self.prefetch_state.background_thread_done.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 检查是否有预取数据可用
    pub fn has_prefetched_data(&self) -> bool {
        // 检查预取缓冲区是否有数据（buffer_pos 之后的数据）
        self.prefetch_state.buffer.len() > self.prefetch_state.buffer_pos
    }

    /// 检查是否还有更多数据可获取
    ///
    /// 返回 true 表示：
    /// - 后台遍历未完成，或
    /// - 预取缓冲区还有未读取的数据
    pub fn has_more(&self) -> bool {
        !self.prefetch_state.prefetch_done || self.has_prefetched_data()
    }

    /// 获取总节点数
    pub fn total_nodes(&self) -> usize {
        self.total_nodes
    }
}

/// 生成唯一迭代器 ID
fn generate_unique_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SearchCache;
    use tempdir::TempDir;
    use std::fs;

    /// 创建测试用的临时目录和文件
    fn setup_test_cache() -> (TempDir, Arc<RwLock<SearchCache>>) {
        let temp_dir = TempDir::new("search_iterator_test").unwrap();

        // 创建测试文件结构
        fs::create_dir_all(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::write(temp_dir.path().join("subdir/file3.txt"), "content3").unwrap();

        // 构建搜索缓存
        let cache = SearchCache::walk_fs(temp_dir.path());
        let cache_arc = Arc::new(RwLock::new(cache));
        (temp_dir, cache_arc)
    }

    /// 测试 1: 迭代器创建
    #[test]
    fn test_iterator_creation() {
        let (_temp_dir, cache_arc) = setup_test_cache();

        let iterator = SearchIterator::new_with_rwlock(
            cache_arc,
            "*.txt",
            SearchOptions::default(),
            10,
            CancellationToken::noop(),
            |_| {}, // 空回调
        );

        assert!(iterator.is_ok());
        let iter = iterator.unwrap();
        assert_eq!(iter.state.yielded_count, 0);
        assert!(!iter.state.search_completed);
    }

    /// 测试 2: 小结果集分批获取
    #[test]
    fn test_next_batch_small_result() {
        let (_temp_dir, cache_arc) = setup_test_cache();

        let mut iterator = SearchIterator::new_with_rwlock(
            cache_arc,
            "*.txt",
            SearchOptions::default(),
            10,
            CancellationToken::noop(),
            |_| {}, // 空回调
        ).unwrap();

        // 第一批获取 - 应该能获取到至少 1 个结果
        let batch1 = iterator.next_batch(10);
        assert!(batch1.indices.len() >= 1, "should have at least 1 result");
    }

    /// 测试 3: 空结果处理
    #[test]
    fn test_empty_result() {
        let (_temp_dir, cache_arc) = setup_test_cache();

        let mut iterator = SearchIterator::new_with_rwlock(
            cache_arc,
            "*.nonexistent",
            SearchOptions::default(),
            10,
            CancellationToken::noop(),
            |_| {}, // 空回调
        ).unwrap();

        let batch = iterator.next_batch(10);
        // 预取模式下，搜索会完成但可能没有结果
        assert!(batch.search_completed);
    }
}
