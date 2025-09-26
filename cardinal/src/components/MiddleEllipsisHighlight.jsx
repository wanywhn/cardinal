import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';

const CHAR_WIDTH = 8; // approximate character width in pixels

export function splitTextWithHighlight(text, searchTerm, options = {}) {
  const { caseInsensitive = false } = options;
  if (!searchTerm) return [{ text, isHighlight: false }];

  const haystack = caseInsensitive ? text.toLocaleLowerCase() : text;
  const needle = caseInsensitive ? searchTerm.toLocaleLowerCase() : searchTerm;
  if (!needle.length) return [{ text, isHighlight: false }];

  const parts = [];
  let startIndex = 0;
  let matchIndex;

  while ((matchIndex = haystack.indexOf(needle, startIndex)) !== -1) {
    if (matchIndex > startIndex) {
      parts.push({ text: text.slice(startIndex, matchIndex), isHighlight: false });
    }

    const matchEnd = matchIndex + needle.length;
    parts.push({ text: text.slice(matchIndex, matchEnd), isHighlight: true });
    startIndex = matchEnd;
  }

  if (startIndex < text.length) {
    parts.push({ text: text.slice(startIndex), isHighlight: false });
  }

  return parts;
}

function applyMiddleEllipsis(parts, maxChars) {
  if (maxChars <= 2) {
    return [{ text: '…', isHighlight: false }];
  }

  const totalLength = parts.reduce((sum, part) => sum + part.text.length, 0);
  if (totalLength <= maxChars) {
    return parts;
  }

  const leftChars = Math.floor((maxChars - 1) / 2); // -1 for ellipsis
  const rightChars = maxChars - leftChars - 1;

  // 收集左侧部分
  const leftParts = [];
  let leftCount = 0;
  for (const part of parts) {
    const remainingSpace = leftChars - leftCount;
    if (remainingSpace <= 0) break;
    
    if (part.text.length <= remainingSpace) {
      leftParts.push(part);
      leftCount += part.text.length;
    } else {
      leftParts.push({
        text: part.text.slice(0, remainingSpace),
        isHighlight: part.isHighlight
      });
      break;
    }
  }

  // 收集右侧部分
  const rightParts = [];
  let rightCount = 0;
  for (let i = parts.length - 1; i >= 0; i--) {
    const part = parts[i];
    const remainingSpace = rightChars - rightCount;
    if (remainingSpace <= 0) break;
    
    if (part.text.length <= remainingSpace) {
      rightParts.unshift(part);
      rightCount += part.text.length;
    } else {
      rightParts.unshift({
        text: part.text.slice(-remainingSpace),
        isHighlight: part.isHighlight
      });
      break;
    }
  }

  return [...leftParts, { text: '…', isHighlight: false }, ...rightParts];
}

export function MiddleEllipsisHighlight({ text, className, highlightTerm, caseInsensitive }) {
  const containerRef = useRef(null);
  const [containerWidth, setContainerWidth] = useState(0);

  // 计算高亮部分（只在 text 或 searchQuery 变化时重新计算）
  const highlightedParts = useMemo(() => {
    return text ? splitTextWithHighlight(text, highlightTerm, { caseInsensitive }) : [];
  }, [text, highlightTerm, caseInsensitive]);

  // 计算显示部分（只在 highlightedParts 或 containerWidth 变化时重新计算）
  const displayParts = useMemo(() => {
    if (!containerWidth || !highlightedParts.length) return highlightedParts;
    
    const maxChars = Math.floor(containerWidth / CHAR_WIDTH) - 1;
    return applyMiddleEllipsis(highlightedParts, maxChars);
  }, [highlightedParts, containerWidth]);

  // 优化的 resize 处理
  const updateWidth = useCallback(() => {
    const el = containerRef.current;
    if (el) {
      const newWidth = el.getBoundingClientRect().width;
      setContainerWidth(newWidth);
    }
  }, []);

  useEffect(() => {
    updateWidth();
    
    const resizeObserver = new ResizeObserver(updateWidth);
    const el = containerRef.current;
    if (el) resizeObserver.observe(el);
    
    return () => resizeObserver.disconnect();
  }, [updateWidth]);

  return (
    <span ref={containerRef} className={className} title={text} style={{ display: 'block', width: '100%' }}>
      {displayParts.map((part, index) => (part.isHighlight ? <strong key={index}>{part.text}</strong> : <span key={index}>{part.text}</span>))}
    </span>
  );
}
