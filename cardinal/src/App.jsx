import { useRef, useCallback, useEffect, useReducer } from 'react';
import './App.css';
import { ContextMenu } from './components/ContextMenu';
import { ColumnHeader } from './components/ColumnHeader';
import { FileRow } from './components/FileRow';
import StatusBar from './components/StatusBar';
import { useColumnResize } from './hooks/useColumnResize';
import { useContextMenu } from './hooks/useContextMenu';
import { ROW_HEIGHT, OVERSCAN_ROW_COUNT, SEARCH_DEBOUNCE_MS } from './constants';
import { VirtualList } from './components/VirtualList';
import { StateDisplay } from './components/StateDisplay';
import { usePreventRefresh } from './hooks/usePreventRefresh';
import { invoke } from '@tauri-apps/api/core';
import { listen, once } from '@tauri-apps/api/event';

const initialState = {
  results: [],
  isInitialized: false,
  scannedFiles: 0,
  processedEvents: 0,
  currentQuery: '',
  showLoadingUI: false,
  initialFetchCompleted: false,
  durationMs: null,
  resultCount: 0,
  searchError: null,
};

function reducer(state, action) {
  switch (action.type) {
    case 'STATUS_UPDATE':
      return {
        ...state,
        scannedFiles: action.payload.scannedFiles,
        processedEvents: action.payload.processedEvents,
      };
    case 'INIT_COMPLETED':
      return { ...state, isInitialized: true };
    case 'SEARCH_REQUEST':
      return {
        ...state,
        searchError: null,
        showLoadingUI: action.payload.immediate ? true : state.showLoadingUI,
      };
    case 'SEARCH_LOADING_DELAY':
      return {
        ...state,
        showLoadingUI: true,
      };
    case 'SEARCH_SUCCESS':
      return {
        ...state,
        results: action.payload.results,
        currentQuery: action.payload.query,
        showLoadingUI: false,
        initialFetchCompleted: true,
        durationMs: action.payload.duration,
        resultCount: action.payload.count,
        searchError: null,
      };
    case 'SEARCH_FAILURE':
      return {
        ...state,
        showLoadingUI: false,
        searchError: action.payload.error,
        initialFetchCompleted: true,
        durationMs: action.payload.duration,
        resultCount: 0,
      };
    default:
      return state;
  }
}

function App() {
  usePreventRefresh();
  const [state, dispatch] = useReducer(reducer, initialState);
  const {
    results,
    isInitialized,
    scannedFiles,
    processedEvents,
    currentQuery,
    showLoadingUI,
    initialFetchCompleted,
    durationMs,
    resultCount,
    searchError
  } = state;
  const { colWidths, onResizeStart, autoFitColumns } = useColumnResize();
  const {
    menu, showContextMenu, showHeaderContextMenu, closeMenu, getMenuItems
  } = useContextMenu(autoFitColumns);

  const headerRef = useRef(null);
  const virtualListRef = useRef(null);
  const prevSearchSignatureRef = useRef({ query: '', useRegex: false, caseSensitive: false });
  const prevResultsLenRef = useRef(0);
  const debounceTimerRef = useRef(null);
  const loadingDelayTimerRef = useRef(null);
  const hasInitialSearchRunRef = useRef(false);
  const latestSearchRef = useRef({ query: '', useRegex: false, caseSensitive: false });
  const [searchParams, updateSearchParams] = useReducer((prev, patch) => {
    const next = { ...prev, ...patch };
    latestSearchRef.current = next;
    return next;
  }, latestSearchRef.current);
  const { useRegex, caseSensitive } = searchParams;

  useEffect(() => {
    let isMounted = true;
    let unlistenStatus;
    let unlistenInit;

    const setupListeners = async () => {
      unlistenStatus = await listen('status_bar_update', (event) => {
        if (!isMounted) return;
        const { scanned_files, processed_events } = event.payload;
        dispatch({
          type: 'STATUS_UPDATE',
          payload: {
            scannedFiles: scanned_files,
            processedEvents: processed_events
          }
        });
      });

      unlistenInit = await once('init_completed', () => {
        if (!isMounted) return;
        dispatch({ type: 'INIT_COMPLETED' });
      });
    };

    setupListeners();

    return () => {
      isMounted = false;
      if (typeof unlistenStatus === 'function') {
        unlistenStatus();
      }
      if (typeof unlistenInit === 'function') {
        unlistenInit();
      }
    };
  }, []);

  const handleSearch = useCallback(async (overrides = {}) => {
    const nextSearch = { ...latestSearchRef.current, ...overrides };
    latestSearchRef.current = nextSearch;

    const { query, useRegex: nextUseRegex, caseSensitive: nextCaseSensitive } = nextSearch;
    const startTs = performance.now();
    const isInitial = !hasInitialSearchRunRef.current;
    const trimmedQuery = query.trim();

    dispatch({ type: 'SEARCH_REQUEST', payload: { immediate: isInitial } });

    if (!isInitial) {
      if (loadingDelayTimerRef.current) {
        clearTimeout(loadingDelayTimerRef.current);
      }
      loadingDelayTimerRef.current = setTimeout(() => {
        dispatch({ type: 'SEARCH_LOADING_DELAY' });
        loadingDelayTimerRef.current = null;
      }, 150);
    }

    try {
      const searchResults = await invoke('search', {
        query,
        options: {
          useRegex: nextUseRegex,
          caseInsensitive: !nextCaseSensitive,
        }
      });

      if (loadingDelayTimerRef.current) {
        clearTimeout(loadingDelayTimerRef.current);
        loadingDelayTimerRef.current = null;
      }

      const endTs = performance.now();
      const duration = endTs - startTs;

      dispatch({
        type: 'SEARCH_SUCCESS',
        payload: {
          results: searchResults,
          query: trimmedQuery,
          duration,
          count: Array.isArray(searchResults) ? searchResults.length : 0
        }
      });
    } catch (error) {
      console.error('Search failed:', error);

      if (loadingDelayTimerRef.current) {
        clearTimeout(loadingDelayTimerRef.current);
        loadingDelayTimerRef.current = null;
      }

      const endTs = performance.now();
      const duration = endTs - startTs;

      dispatch({
        type: 'SEARCH_FAILURE',
        payload: {
          error: error || 'An unknown error occurred.',
          duration
        }
      });
    } finally {
      hasInitialSearchRunRef.current = true;
    }
  }, [dispatch]);

  const onQueryChange = useCallback((e) => {
    const inputValue = e.target.value;
    updateSearchParams({ query: inputValue });
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }
    debounceTimerRef.current = setTimeout(() => {
      handleSearch({ query: inputValue });
    }, SEARCH_DEBOUNCE_MS);
  }, [handleSearch, updateSearchParams]);

  const onToggleRegex = useCallback((event) => {
    const nextValue = event.target.checked;
    updateSearchParams({ useRegex: nextValue });
  }, [updateSearchParams]);

  const onToggleCaseSensitive = useCallback((event) => {
    const nextValue = event.target.checked;
    updateSearchParams({ caseSensitive: nextValue });
  }, [updateSearchParams]);

  useEffect(() => () => {
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }
    if (loadingDelayTimerRef.current) {
      clearTimeout(loadingDelayTimerRef.current);
    }
  }, []);

  useEffect(() => {
    if (!hasInitialSearchRunRef.current) {
      handleSearch({ query: '' });
    }
  }, [handleSearch]);

  useEffect(() => {
    if (!hasInitialSearchRunRef.current) {
      return;
    }
    handleSearch();
  }, [caseSensitive, handleSearch, useRegex]);

  // 优化的搜索结果处理逻辑（保持使用 useRef，但简化其他逻辑）
  useEffect(() => {
    if (results.length === 0) return;

    const prevSignature = prevSearchSignatureRef.current;
    const currentSignature = { query: currentQuery, useRegex, caseSensitive };
    const isNewSignature =
      prevSignature.query !== currentSignature.query ||
      prevSignature.useRegex !== currentSignature.useRegex ||
      prevSignature.caseSensitive !== currentSignature.caseSensitive;
    const wasEmpty = prevResultsLenRef.current === 0;

    if (isNewSignature && virtualListRef.current) {
      virtualListRef.current.scrollToTop();
    }

    if ((isNewSignature || wasEmpty) && virtualListRef.current?.ensureRangeLoaded) {
      const preloadCount = Math.min(30, results.length);
      virtualListRef.current.ensureRangeLoaded(0, preloadCount - 1);
    }

    prevSearchSignatureRef.current = currentSignature;
    prevResultsLenRef.current = results.length;
  }, [results, currentQuery, useRegex, caseSensitive]);

  // 滚动同步处理 - 单向同步版本（Grid -> Header）
  const handleHorizontalSync = useCallback((scrollLeft) => {
    if (headerRef.current) headerRef.current.scrollLeft = scrollLeft;
  }, []);

  // 单元格渲染
  const renderRow = (rowIndex, item, rowStyle) => (
    <FileRow
      key={rowIndex}
      item={item}
      rowIndex={rowIndex}
      style={{ ...rowStyle, width: 'var(--columns-total)' }}
      onContextMenu={showContextMenu}
      searchQuery={currentQuery}
      caseInsensitive={!caseSensitive}
    />
  );

  const getDisplayState = () => {
    if (!initialFetchCompleted) return 'loading';
    if (showLoadingUI) return 'loading';
    if (searchError) return 'error';
    if (results.length === 0) return 'empty';
    return 'results';
  };

  const displayState = getDisplayState();

  return (
    <main className="container">
      <div className="search-container">
        <div className="search-bar">
          <input
            id="search-input"
            onChange={onQueryChange}
            placeholder="Search for files and folders..."
            spellCheck={false}
            autoCorrect="off"
            autoComplete="off"
            autoCapitalize="off"
          />
          <div className="search-options">
            <label
              className="search-option"
              title="Toggle case-sensitive matching"
            >
              <input
                type="checkbox"
                checked={caseSensitive}
                onChange={onToggleCaseSensitive}
                aria-label="Toggle case-sensitive matching"
              />
              <span
                className="search-option__display"
                aria-hidden="true"
              >
                Aa
              </span>
              <span className="sr-only">Toggle case-sensitive matching</span>
            </label>
            <label
              className="search-option"
              title="Enable regular expression search"
            >
              <input
                type="checkbox"
                checked={useRegex}
                onChange={onToggleRegex}
                aria-label="Enable regular expression search"
              />
              <span
                className="search-option__display"
                aria-hidden="true"
              >
                .*
              </span>
              <span className="sr-only">Enable regular expression search</span>
            </label>
          </div>
        </div>
      </div>
      <div
        className="results-container"
        style={{
          ['--w-filename']: `${colWidths.filename}px`,
          ['--w-path']: `${colWidths.path}px`,
          ['--w-size']: `${colWidths.size}px`,
          ['--w-modified']: `${colWidths.modified}px`,
          ['--w-created']: `${colWidths.created}px`,
        }}
      >
        <div className="scroll-area">
          <ColumnHeader
            ref={headerRef}
            onResizeStart={onResizeStart}
            onContextMenu={showHeaderContextMenu}
          />
          <div className="flex-fill">
            {displayState !== 'results' ? (
              <StateDisplay state={displayState} message={searchError} query={currentQuery} />
            ) : (
              <VirtualList
                ref={virtualListRef}
                results={results}
                rowHeight={ROW_HEIGHT}
                overscan={OVERSCAN_ROW_COUNT}
                renderRow={renderRow}
                onScrollSync={handleHorizontalSync}
                className="virtual-list"
              />
            )}
          </div>
        </div>
      </div>
      {menu.visible && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          items={getMenuItems()}
          onClose={closeMenu}
        />
      )}
      <StatusBar
        scannedFiles={scannedFiles}
        processedEvents={processedEvents}
        isReady={isInitialized}
        searchDurationMs={durationMs}
        resultCount={resultCount}
      />
    </main>
  );
}

export default App;
