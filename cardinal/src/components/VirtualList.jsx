import React, { useRef, useState, useCallback, useLayoutEffect, useEffect, forwardRef, useImperativeHandle } from 'react';

/**
 * 等高行虚拟列表 (支持任意滚动位置, 不会在底部/中间跳动)
 * Props:
 *  - rowCount: 总行数
 *  - rowHeight: 行高 (px)
 *  - overscan: 额外预渲染的行数 (上/下 各 overscan)
 *  - renderRow(rowIndex, style): 行渲染函数
 *  - onRangeChange(start, end): 当可见+overscan区间变化时回调 (用于数据预加载)
 *  - onScrollSync(scrollLeft): 水平滚动同步 (用于列头同步)
 *  - className: 自定义 class
 * Exposed imperative API via ref:
 *  - scrollToTop()
 *  - scrollToIndex(index, align = 'start')  // 可选: 'start' | 'center' | 'end'
 */
export const VirtualList = forwardRef(function VirtualList({
	rowCount = 0,
	rowHeight = 24,
	overscan = 5,
	renderRow,
	onRangeChange,
	onScrollSync,
	className = '',
	showEmptyState = true
}, ref) {
	const scrollRef = useRef(null);
	const lastScrollLeftRef = useRef(0);
	const rafRef = useRef(0);
	const [range, setRange] = useState({ start: 0, end: -1 });

	const computeRange = useCallback((el) => {
		if (!el) return { start: 0, end: -1 };
		if (rowCount === 0) return { start: 0, end: -1 };
		const viewportHeight = el.clientHeight || 0;
		const rawStart = Math.floor(el.scrollTop / rowHeight);
		const visible = viewportHeight > 0 ? Math.ceil(viewportHeight / rowHeight) : 0;
		const rawEnd = rawStart + visible - 1;
		return {
			start: Math.max(0, rawStart - overscan),
			end: Math.min(rowCount - 1, rawEnd + overscan)
		};
	}, [rowCount, rowHeight, overscan]);

	const updateRange = useCallback(() => {
		const el = scrollRef.current;
		const next = computeRange(el);
		setRange(prev => {
			if (prev.start === next.start && prev.end === next.end) return prev;
			if (onRangeChange && next.end >= next.start && rowCount > 0) {
				onRangeChange(next.start, next.end);
			}
			return next;
		});
	}, [computeRange, onRangeChange, rowCount]);

	const handleScroll = useCallback(() => {
		if (rafRef.current) cancelAnimationFrame(rafRef.current);
		rafRef.current = requestAnimationFrame(() => {
			updateRange();
			const el = scrollRef.current;
			if (!el) return;
			const sl = el.scrollLeft;
			if (onScrollSync && sl !== lastScrollLeftRef.current) {
				lastScrollLeftRef.current = sl;
				onScrollSync(sl);
			}
		});
	}, [updateRange, onScrollSync]);

	// Resize observer (height changes)
	useLayoutEffect(() => {
		const el = scrollRef.current;
		if (!el) return;
		const ro = new ResizeObserver(() => updateRange());
		ro.observe(el);
		updateRange();
		return () => ro.disconnect();
	}, [rowCount, rowHeight, updateRange]);

	// Recalc when dependencies change explicitly
	useEffect(() => { updateRange(); }, [rowCount, rowHeight, overscan, updateRange]);

	useEffect(() => () => rafRef.current && cancelAnimationFrame(rafRef.current), []);

	useImperativeHandle(ref, () => ({
		scrollToTop: () => {
			const el = scrollRef.current; if (!el) return;
			el.scrollTo({ top: 0, behavior: 'instant' });
			updateRange();
		},
		scrollToIndex: (index, align = 'start') => {
			const el = scrollRef.current; if (!el) return;
			if (index < 0 || index >= rowCount) return;
			const viewportHeight = el.clientHeight || 0;
			const targetTop = index * rowHeight;
			let scrollTop = targetTop;
			if (align === 'center') scrollTop = targetTop - (viewportHeight - rowHeight) / 2;
			else if (align === 'end') scrollTop = targetTop - (viewportHeight - rowHeight);
			scrollTop = Math.max(0, Math.min(scrollTop, rowCount * rowHeight - viewportHeight));
			el.scrollTo({ top: scrollTop });
			updateRange();
		}
	}), [rowCount, rowHeight, updateRange]);

	const { start, end } = range;
	const totalHeight = rowCount * rowHeight;
	const count = end >= start && rowCount > 0 ? end - start + 1 : 0;
	const items = count > 0 ? Array.from({ length: count }, (_, i) => {
		const rowIndex = start + i;
		return renderRow(rowIndex, {
			position: 'absolute',
			top: i * rowHeight,
			height: rowHeight,
			left: 0,
			right: 0
		});
	}) : null;

	return (
		<div
			ref={scrollRef}
			className={className}
			style={{ position: 'relative', overflow: 'auto', width: '100%', height: '100%' }}
			onScroll={handleScroll}
			role="list"
			aria-rowcount={rowCount}
		>
			<div style={{ height: totalHeight, position: 'relative' }}>
				<div style={{ position: 'absolute', top: start * rowHeight, left: 0, right: 0 }}>
					{items}
				</div>
			</div>
			{showEmptyState && rowCount === 0 && (
				<div className="empty-state">
					<div className="empty-icon" aria-hidden="true">
						<svg width="72" height="72" viewBox="0 0 72 72" fill="none" stroke="currentColor" strokeWidth="1.5">
							<circle cx="32" cy="32" r="18" strokeOpacity="0.5" />
							<path d="M45 45 L60 60" strokeLinecap="round" />
							<circle cx="24" cy="30" r="2" fill="currentColor" />
							<circle cx="32" cy="30" r="2" fill="currentColor" />
							<circle cx="40" cy="30" r="2" fill="currentColor" />
							<path d="M25 38 Q32 44 39 38" strokeLinecap="round" strokeLinejoin="round" />
						</svg>
					</div>
					<div className="empty-title">No Results</div>
					<div className="empty-desc">Try adjusting your keywords or filters.</div>
					<ul className="empty-tips">
						<li>Use more specific terms (e.g. src/components)</li>
						<li>Search partial names: part of filename/path</li>
						<li>Case-insensitive by default</li>
					</ul>
				</div>
			)}
		</div>
	);
});

VirtualList.displayName = 'VirtualList';

export default VirtualList;