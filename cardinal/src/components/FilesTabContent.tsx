import React from 'react';
import type { CSSProperties, ReactNode } from 'react';
import type { MouseEvent as ReactMouseEvent } from 'react';
import { ColumnHeader } from './ColumnHeader';
import { StateDisplay, type DisplayState } from './StateDisplay';
import { VirtualList } from './VirtualList';
import type { ColumnKey } from '../constants';
import type { VirtualListHandle } from './VirtualList';
import type { SearchResultItem } from '../types/search';
import type { SlabIndex } from '../types/slab';

type FilesTabContentProps = {
  headerRef: React.RefObject<HTMLDivElement | null>;
  onResizeStart: (columnKey: ColumnKey) => (event: ReactMouseEvent<HTMLSpanElement>) => void;
  onHeaderContextMenu?: (event: ReactMouseEvent<HTMLDivElement>) => void;
  displayState: DisplayState;
  searchErrorMessage: string | null;
  currentQuery: string;
  virtualListRef: React.RefObject<VirtualListHandle | null>;
  results: SlabIndex[];
  rowHeight: number;
  overscan: number;
  renderRow: (
    rowIndex: number,
    item: SearchResultItem | undefined,
    rowStyle: CSSProperties,
  ) => ReactNode;
  onScrollSync: (scrollLeft: number) => void;
};

export function FilesTabContent({
  headerRef,
  onResizeStart,
  onHeaderContextMenu,
  displayState,
  searchErrorMessage,
  currentQuery,
  virtualListRef,
  results,
  rowHeight,
  overscan,
  renderRow,
  onScrollSync,
}: FilesTabContentProps): React.JSX.Element {
  return (
    <div className="scroll-area">
      <ColumnHeader
        ref={headerRef}
        onResizeStart={onResizeStart}
        onContextMenu={onHeaderContextMenu}
      />
      <div className="flex-fill">
        {displayState !== 'results' ? (
          <StateDisplay state={displayState} message={searchErrorMessage} query={currentQuery} />
        ) : (
          <VirtualList
            ref={virtualListRef}
            results={results}
            rowHeight={rowHeight}
            overscan={overscan}
            renderRow={renderRow}
            onScrollSync={onScrollSync}
            className="virtual-list"
          />
        )}
      </div>
    </div>
  );
}
