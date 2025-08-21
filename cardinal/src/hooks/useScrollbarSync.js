import { useEffect } from 'react';

// Custom hook to sync vertical and horizontal scrollbars
export function useScrollbarSync({ listRef, scrollAreaRef, results, colWidths, setVerticalBar, setHorizontalBar }) {
  useEffect(() => {
    function updateVerticalBar() {
      if (!listRef.current || !scrollAreaRef.current) return;
      const grid = listRef.current.Grid || listRef.current;
      const scroller = grid && grid._scrollingContainer ? grid._scrollingContainer : null;
      const totalRows = results.length;
      const rowHeight = 24;
      const visibleHeight = grid.props.height;
      const totalHeight = totalRows * rowHeight;
      // Visual track height from DOM: scroll area clientHeight - header height
      const scrollAreaEl = scrollAreaRef.current;
      const headerEl = scrollAreaEl.firstElementChild; // ColumnHeader is first child
      const headerHeight = headerEl ? headerEl.offsetHeight : 0;
      const trackHeight = Math.max(0, (scrollAreaEl ? scrollAreaEl.clientHeight : visibleHeight) - headerHeight);
      // Prefer reading the live scrollTop from the actual scrolling element to avoid lag
      const scrollTop = scroller ? scroller.scrollTop : (grid.state ? grid.state.scrollTop : 0);
      // Use actual DOM scroll metrics when available
      const domClientH = scroller ? scroller.clientHeight : visibleHeight;
      const domScrollH = scroller ? scroller.scrollHeight : totalHeight;
      if (domScrollH <= domClientH) {
        console.debug('[scrollbar][V] hidden (content fits)', {
          totalRows,
          rowHeight,
          visibleHeight,
          headerHeight,
          trackHeight,
          totalHeight,
          domClientH,
          domScrollH,
        });
        setVerticalBar({ top: 0, height: 0, visible: false });
        return;
      }
      // VS Code-like mapping using real track height and DOM scroll metrics
      // Thumb size T ≈ max(32, trackHeight * (domClientH / domScrollH))
      const barHeight = Math.max(32, trackHeight * (domClientH / Math.max(1, domScrollH)));
      const maxContentScroll = Math.max(1, domScrollH - domClientH);
      const maxTrack = Math.max(0, trackHeight - barHeight);
      const ratio = scrollTop / maxContentScroll;
      const barTop = Math.max(0, Math.min(maxTrack, ratio * maxTrack));
      console.debug('[scrollbar][V] update', {
        totalRows,
        rowHeight,
        visibleHeight,
        headerHeight,
        trackHeight,
        totalHeight,
        domClientH,
        domScrollH,
        scrollTopSource: scroller ? 'scroller' : 'grid.state',
        scrollTop,
        barHeight,
        maxContentScroll,
        maxTrack,
        ratio,
        barTop,
      });
      setVerticalBar({ top: barTop, height: barHeight, visible: true });
    }
    function updateHorizontalBar() {
      if (!scrollAreaRef.current) return;
      const el = scrollAreaRef.current;
      const scrollWidth = el.scrollWidth;
      const clientWidth = el.clientWidth;
      const scrollLeft = el.scrollLeft;
      if (scrollWidth <= clientWidth) {
        setHorizontalBar({ left: 0, width: 0, visible: false });
        return;
      }
      // VS Code-like mapping: content [0, scrollWidth - clientWidth] -> track [0, clientWidth - barWidth]
      const barWidth = Math.max(32, (clientWidth * clientWidth) / scrollWidth);
      const maxContentScrollX = Math.max(1, scrollWidth - clientWidth);
      const maxTrackX = Math.max(0, clientWidth - barWidth);
      const ratioX = scrollLeft / maxContentScrollX;
      const barLeft = Math.max(0, Math.min(maxTrackX, ratioX * maxTrackX));
      setHorizontalBar({ left: barLeft, width: barWidth, visible: true });
    }
    updateVerticalBar();
    updateHorizontalBar();
    if (!listRef.current) return;
    const grid = listRef.current.Grid || listRef.current;
    const onVScroll = () => updateVerticalBar();
    grid && grid._scrollingContainer && grid._scrollingContainer.addEventListener('scroll', onVScroll);
    const el = scrollAreaRef.current;
    const onHScroll = () => updateHorizontalBar();
    el && el.addEventListener('scroll', onHScroll);
    const onResize = () => {
      updateVerticalBar();
      updateHorizontalBar();
    };
    window.addEventListener('resize', onResize);
    return () => {
      grid && grid._scrollingContainer && grid._scrollingContainer.removeEventListener('scroll', onVScroll);
      el && el.removeEventListener('scroll', onHScroll);
      window.removeEventListener('resize', onResize);
    };
  }, [results, colWidths, listRef, scrollAreaRef, setVerticalBar, setHorizontalBar]);
}
