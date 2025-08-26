import { useRef, useEffect, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * 简化的行数据管理
 */
export function useRowData(results) {
  const [cache, setCache] = useState(new Map());
  const loadingRef = useRef(new Set());

  // 当结果集变更时清理缓存
  useEffect(() => {
    setCache(new Map());
    loadingRef.current.clear();
  }, [results]);

  const ensureRangeLoaded = useCallback(async (start, end) => {
    if (start < 0 || end < start || results.length === 0) return;
    
    // 找出需要加载的索引
    const needLoading = [];
    for (let i = start; i <= end; i++) {
      if (!cache.has(i) && !loadingRef.current.has(i)) {
        needLoading.push(i);
        loadingRef.current.add(i);
      }
    }
    
    if (needLoading.length === 0) return;
    
    try {
      const slice = needLoading.map(i => results[i]);
      const fetched = await invoke('get_nodes_info', { results: slice });
      
      // 更新缓存
      setCache(prev => {
        const newCache = new Map(prev);
        needLoading.forEach((originalIndex, i) => {
          newCache.set(originalIndex, fetched[i]);
          loadingRef.current.delete(originalIndex);
        });
        return newCache;
      });
    } catch (err) {
      // 清理加载状态
      needLoading.forEach(i => loadingRef.current.delete(i));
      console.error('Failed loading rows', err);
    }
  }, [results, cache]);

  const getItem = useCallback((index) => {
    const cached = cache.get(index);
    if (cached) return cached;
    
    // 如果没有缓存且不在加载中，触发加载
    if (!loadingRef.current.has(index)) {
      const batchStart = Math.max(0, index - 5);
      const batchEnd = Math.min(results.length - 1, index + 15);
      ensureRangeLoaded(batchStart, batchEnd);
    }
    
    return null; // 返回 null 让 FileRow 显示加载状态
  }, [cache, results, ensureRangeLoaded]);

  return { getItem, ensureRangeLoaded };
}
