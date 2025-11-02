import React, { useMemo, memo } from 'react';
import type { CSSProperties, MouseEvent as ReactMouseEvent } from 'react';
import { MiddleEllipsisHighlight } from './MiddleEllipsisHighlight';
import { formatKB, formatTimestamp } from '../utils/format';
import type { SearchResultItem } from '../types/search';

const SEGMENT_SEPARATOR = /[\\/]+/;

type FileRowProps = {
  item?: SearchResultItem;
  rowIndex: number;
  style?: CSSProperties;
  onContextMenu?: (event: ReactMouseEvent<HTMLDivElement>, path: string) => void;
  searchQuery?: string;
  caseInsensitive?: boolean;
};

function deriveHighlightTerm(query?: string): string {
  if (!query) return '';
  const segments = query.split(SEGMENT_SEPARATOR).filter(Boolean);
  if (segments.length === 0) {
    return query.trim();
  }
  return segments[segments.length - 1].trim();
}

export const FileRow = memo(function FileRow({
  item,
  rowIndex,
  style,
  onContextMenu,
  searchQuery,
  caseInsensitive,
}: FileRowProps): React.JSX.Element | null {
  const highlightTerm = useMemo(() => deriveHighlightTerm(searchQuery), [searchQuery]);

  if (!item) {
    return null;
  }

  const path = item.path;
  let filename = '';
  let directoryPath = '';

  if (path) {
    if (path === '/') {
      directoryPath = '/';
    } else {
      // Split on either slash to support Windows and POSIX paths.
      const parts = path.split(/[\\/]/);
      filename = parts.pop() || '';
      directoryPath = parts.join('/');
    }
  }

  const metadata = item.metadata;
  const mtimeSec = metadata?.mtime ?? item.mtime;
  const ctimeSec = metadata?.ctime ?? item.ctime;
  const sizeBytes = metadata?.size ?? item.size;
  const sizeText = metadata?.type !== 1 ? formatKB(sizeBytes) : null;
  const mtimeText = formatTimestamp(mtimeSec);
  const ctimeText = formatTimestamp(ctimeSec);

  const handleContextMenu = (e: ReactMouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    if (path && onContextMenu) {
      onContextMenu(e, path);
    }
  };

  return (
    <div
      style={style}
      className={`row columns ${rowIndex % 2 === 0 ? 'row-even' : 'row-odd'}`}
      onContextMenu={handleContextMenu}
      title={path}
    >
      <div className="filename-column">
        {item.icon ? (
          <img src={item.icon} alt="icon" className="file-icon" />
        ) : (
          <span className="file-icon file-icon-placeholder" aria-hidden="true" />
        )}
        <MiddleEllipsisHighlight
          className="filename-text"
          text={filename}
          highlightTerm={highlightTerm}
          caseInsensitive={caseInsensitive}
        />
      </div>
      {/* Directory column renders the parent path (the filename column already shows the leaf). */}
      <span className="path-text" title={directoryPath}>
        {directoryPath}
      </span>
      <span className={`size-text ${!sizeText ? 'muted' : ''}`}>{sizeText || '—'}</span>
      <span className={`mtime-text ${!mtimeText ? 'muted' : ''}`}>{mtimeText || '—'}</span>
      <span className={`ctime-text ${!ctimeText ? 'muted' : ''}`}>{ctimeText || '—'}</span>
    </div>
  );
});

FileRow.displayName = 'FileRow';
