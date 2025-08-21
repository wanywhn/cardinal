
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { once, listen } from '@tauri-apps/api/event';
import { InfiniteLoader, List, AutoSizer } from 'react-virtualized';
import 'react-virtualized/styles.css';
import "./App.css";
import { LRUCache } from "./utils/LRUCache";
import { formatKB } from "./utils/format";
import { useScrollbarSync } from "./hooks/useScrollbarSync";
import { VirtualizedRow } from "./components/VirtualizedRow";
import { ColumnHeader } from "./components/ColumnHeader";


function App() {
  const [results, setResults] = useState([]);
  const [colWidths, setColWidths] = useState({ filename: 240, path: 600, modified: 180, created: 180, size: 120 });
  const resizingRef = useRef(null);
  const lruCache = useRef(new LRUCache(1000));
  const infiniteLoaderRef = useRef(null);
  const debounceTimerRef = useRef(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [isStatusBarVisible, setIsStatusBarVisible] = useState(true);
  const [statusText, setStatusText] = useState("Walking filesystem...");
  const scrollAreaRef = useRef(null);
  const listRef = useRef(null);
  const [verticalBar, setVerticalBar] = useState({ top: 0, height: 0, visible: false });
  const [horizontalBar, setHorizontalBar] = useState({ left: 0, width: 0, visible: false });

  // Status event listeners
  useEffect(() => {
    listen('status_update', (event) => setStatusText(event.payload));
    once('init_completed', () => setIsInitialized(true));
  }, []);

  // Status bar fade out
  useEffect(() => {
    if (isInitialized) {
      const timer = setTimeout(() => setIsStatusBarVisible(false), 2000);
      return () => clearTimeout(timer);
    }
  }, [isInitialized]);

  // Reset InfiniteLoader cache on results change
  useEffect(() => {
    if (infiniteLoaderRef.current) {
      infiniteLoaderRef.current.resetLoadMoreRowsCache(true);
    }
  }, [results]);

  // 滚动条同步逻辑抽离为hook
  useScrollbarSync({ listRef, scrollAreaRef, results, colWidths, setVerticalBar, setHorizontalBar });

  // 横向滚动条拖动
  const onHorizontalBarMouseDown = (e) => {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const el = scrollAreaRef.current;
    const clientWidth = el?.clientWidth || 1;
    const scrollWidth = el?.scrollWidth || 1;
    const barWidth = horizontalBar.width || 0;
    // Track geometry from the visual track (thumb's parent)
    const trackRect = e.currentTarget.parentElement.getBoundingClientRect();
    const trackLeft = trackRect.left;
    const maxTrackX = Math.max(0, trackRect.width - barWidth);
    // Preserve pointer offset inside the thumb to keep alignment during drag
    const grabOffsetX = startX - (trackLeft + (horizontalBar.left || 0));
    // Avoid text selection while dragging
    const prevUserSelect = document.body.style.userSelect;
    document.body.style.userSelect = 'none';
    function onMove(ev) {
      // Map mouse to thumb-left using track rect and initial grab offset
      let newLeft = Math.max(0, Math.min(maxTrackX, (ev.clientX - trackLeft - grabOffsetX)));
      // Map thumb position to content scroll range (scrollWidth - clientWidth)
      const maxContentScrollX = Math.max(0, scrollWidth - clientWidth);
      const ratioX = maxTrackX > 0 ? (newLeft / maxTrackX) : 0;
      const scrollLeft = Math.max(0, Math.min(maxContentScrollX, ratioX * maxContentScrollX));
      if (el) el.scrollLeft = scrollLeft;
    }
    function onUp() {
      document.body.style.userSelect = prevUserSelect;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp, { once: true });
  };

  // 竖直滚动条拖动
  const onVerticalBarMouseDown = (e) => {
    e.preventDefault();
    e.stopPropagation();
    const startY = e.clientY;
    const grid = listRef.current?.Grid || listRef.current;
    const visibleHeight = grid?.props.height || 1;
    // Track geometry from the visual track (thumb's parent)
    const trackRect = e.currentTarget.parentElement.getBoundingClientRect();
    const trackTop = trackRect.top;
    const trackHeight = trackRect.height || visibleHeight;
    // Preserve pointer offset inside the thumb to keep alignment during drag
    const grabOffsetY = startY - (trackTop + (verticalBar.top || 0));
    const prevUserSelect = document.body.style.userSelect;
    document.body.style.userSelect = 'none';
    const scroller = grid && grid._scrollingContainer ? grid._scrollingContainer : null;
    function onMove(ev) {
      // Clamp thumb within track (track length = trackHeight - barHeight)
      const barHeight = verticalBar.height || 0;
      const maxTrackY = Math.max(0, trackHeight - barHeight);
      // Map mouse to thumb-top using track rect and initial grab offset
      let newTop = Math.max(0, Math.min(maxTrackY, (ev.clientY - trackTop - grabOffsetY)));
      // Map thumb to actual DOM content scroll range (scrollHeight - clientHeight)
      const domClientH = scroller ? scroller.clientHeight : visibleHeight;
      const domScrollH = scroller ? scroller.scrollHeight : (results.length * 24);
      const maxContentScrollY = Math.max(0, domScrollH - domClientH);
      const ratioY = maxTrackY > 0 ? (newTop / maxTrackY) : 0;
      const scrollTop = Math.max(0, Math.min(maxContentScrollY, ratioY * maxContentScrollY));
      if (grid && grid._scrollingContainer) {
        grid._scrollingContainer.scrollTop = scrollTop;
      }
    }
    function onUp() {
      document.body.style.userSelect = prevUserSelect;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp, { once: true });
  };

  // 搜索逻辑
  const handleSearch = async (query) => {
    let searchResults = [];
    if (query.trim() !== '') {
      searchResults = await invoke("search", { query });
    }
    lruCache.current.clear();
    setResults(searchResults);
  };

  // 搜索输入防抖
  const onQueryChange = (e) => {
    const currentQuery = e.target.value;
    clearTimeout(debounceTimerRef.current);
    debounceTimerRef.current = setTimeout(() => {
      handleSearch(currentQuery);
    }, 300);
  };

  // 列宽拖拽
  const onResizeStart = (key) => (e) => {
    e.preventDefault();
    e.stopPropagation();
    resizingRef.current = { key, startX: e.clientX, startW: colWidths[key] };
    window.addEventListener('mousemove', onResizing);
    window.addEventListener('mouseup', onResizeEnd, { once: true });
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';
  };
  const onResizing = (e) => {
    const ctx = resizingRef.current;
    if (!ctx) return;
    const delta = e.clientX - ctx.startX;
    const nextW = Math.max(80, Math.min(1200, ctx.startW + delta));
    setColWidths((w) => ({ ...w, [ctx.key]: nextW }));
  };
  const onResizeEnd = () => {
    resizingRef.current = null;
    window.removeEventListener('mousemove', onResizing);
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  };

  // 虚拟列表加载
  const isRowLoaded = ({ index }) => lruCache.current.has(index);
  const loadMoreRows = async ({ startIndex, stopIndex }) => {
    let rows = results.slice(startIndex, stopIndex + 1);
    const searchResults = await invoke("get_nodes_info", { results: rows });
    for (let i = startIndex; i <= stopIndex; i++) {
      lruCache.current.put(i, searchResults[i - startIndex]);
    }
  };

  // 行渲染抽离为组件
  const rowRenderer = ({ key, index, style }) => (
    <VirtualizedRow key={key} index={index} style={style} item={lruCache.current.get(index)} />
  );

  return (
    <main className="container">
      <div className="search-container">
        <input
          id="search-input"
          onChange={onQueryChange}
          placeholder="Search for files and folders..."
          spellCheck={false}
          autoCorrect="off"
          autoComplete="off"
          autoCapitalize="off"
        />
      </div>
      <div
        className="results-container"
        style={{
          ['--w-filename']: `${colWidths.filename}px`,
          ['--w-path']: `${colWidths.path}px`,
          ['--w-modified']: `${colWidths.modified}px`,
          ['--w-created']: `${colWidths.created}px`,
          ['--w-size']: `${colWidths.size}px`,
        }}
      >
        {/* 横向滚动区域 */}
        <div className="scroll-area" ref={scrollAreaRef}>
          <ColumnHeader colWidths={colWidths} onResizeStart={onResizeStart} />
          <div style={{ flex: 1, minHeight: 0 }}>
            <InfiniteLoader
              ref={infiniteLoaderRef}
              isRowLoaded={isRowLoaded}
              loadMoreRows={loadMoreRows}
              rowCount={results.length}
            >
              {({ onRowsRendered, registerChild }) => (
                <AutoSizer>
                  {({ height, width }) => {
                    const colGap = 12;
                    const columnsTotal =
                      colWidths.filename + colWidths.path + colWidths.modified + colWidths.created + colWidths.size + (4 * colGap) + 20;
                    return (
                      <List
                        ref={el => {
                          registerChild(el);
                          listRef.current = el;
                        }}
                        onRowsRendered={onRowsRendered}
                        width={Math.max(width, columnsTotal)}
                        height={height}
                        rowCount={results.length}
                        rowHeight={24}
                        rowRenderer={rowRenderer}
                      />
                    );
                  }}
                </AutoSizer>
              )}
            </InfiniteLoader>
          </div>
        </div>
        {/* 悬浮竖直滚动条 */}
        {verticalBar.visible && (
          <div className="vertical-scrollbar">
            <div
              className="vertical-scrollbar-inner"
              style={{
                height: verticalBar.height,
                top: verticalBar.top,
                position: 'absolute',
                right: 0,
              }}
              onMouseDown={onVerticalBarMouseDown}
            />
          </div>
        )}
        {/* 悬浮横向滚动条 */}
        {horizontalBar.visible && (
          <div className="horizontal-scrollbar">
            <div
              className="horizontal-scrollbar-inner"
              style={{
                width: horizontalBar.width,
                left: horizontalBar.left,
                position: 'absolute',
                top: 0,
              }}
              onMouseDown={onHorizontalBarMouseDown}
            />
          </div>
        )}
      </div>
      {isStatusBarVisible && (
        <div className={`status-bar ${isInitialized ? 'fade-out' : ''}`}>
          {isInitialized ? 'Initialized' :
            <div className="initializing-container">
              <div className="spinner"></div>
              <span>{statusText}</span>
            </div>
          }
        </div>
      )}
    </main>
  );
}

export default App;
