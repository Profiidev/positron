import { describe, expect, it } from 'vitest';
import {
  type ToolbarOverflowItem,
  calculateVisibleCount,
  cleanupSeparatorVisibility,
  totalWidth
} from '$lib/components/tiptap/toolbar/toolbar-overflow';

describe('totalWidth', () => {
  it('returns 0 for a non-positive count', () => {
    expect(totalWidth([10, 20], 0, 4)).toBe(0);
    expect(totalWidth([10, 20], -1, 4)).toBe(0);
  });

  it('sums a single item with no gap applied', () => {
    expect(totalWidth([10, 20, 30], 1, 4)).toBe(10);
  });

  it('adds one gap between each pair of items', () => {
    // 10 + 20 + 30 = 60, plus 2 gaps of 4 = 68
    expect(totalWidth([10, 20, 30], 3, 4)).toBe(68);
  });

  it('only counts the first `count` widths', () => {
    expect(totalWidth([10, 20, 30], 2, 0)).toBe(30);
  });
});

describe('calculateVisibleCount', () => {
  it('shows nothing when there are no items', () => {
    expect(calculateVisibleCount(100, [], 20, 4)).toEqual({
      showOverflow: false,
      visibleCount: 0
    });
  });

  it('shows nothing when the container has no width', () => {
    expect(calculateVisibleCount(0, [10, 20], 20, 4)).toEqual({
      showOverflow: false,
      visibleCount: 0
    });
    expect(calculateVisibleCount(-5, [10, 20], 20, 4)).toEqual({
      showOverflow: false,
      visibleCount: 0
    });
  });

  it('shows all items and no overflow when everything fits', () => {
    // Total = 10 + 20 + 4 = 34 <= 100
    expect(calculateVisibleCount(100, [10, 20], 20, 4)).toEqual({
      showOverflow: false,
      visibleCount: 2
    });
  });

  it('shows all items when they fit exactly', () => {
    // Total = 10 + 20 + 4 = 34
    expect(calculateVisibleCount(34, [10, 20], 20, 4)).toEqual({
      showOverflow: false,
      visibleCount: 2
    });
  });

  it('reserves room for the overflow button and trims items that no longer fit', () => {
    // Available with overflow = 50 - 20 - 4 = 26
    // 1 item: 10 <= 26 → visible; 2 items: 10+20+4=34 > 26 → stop
    expect(calculateVisibleCount(50, [10, 20, 30], 20, 4)).toEqual({
      showOverflow: true,
      visibleCount: 1
    });
  });

  it('can show zero items but still flag overflow when nothing fits with the button', () => {
    // FitsAll false (40+...), available = 30 - 20 - 4 = 6, first item 40 > 6
    expect(calculateVisibleCount(30, [40, 50], 20, 4)).toEqual({
      showOverflow: true,
      visibleCount: 0
    });
  });
});

const sep = (id: string): ToolbarOverflowItem => ({ id, isSeparator: true });
const item = (id: string): ToolbarOverflowItem => ({ id });

describe('cleanupSeparatorVisibility', () => {
  it('hides everything past the visible count', () => {
    const items = [item('a'), item('b'), item('c')];
    expect(cleanupSeparatorVisibility(items, 2)).toEqual([true, true, false]);
  });

  it('keeps a separator visible only when flanked by visible non-separators', () => {
    const items = [item('a'), sep('s'), item('b')];
    expect(cleanupSeparatorVisibility(items, 3)).toEqual([true, true, true]);
  });

  it('hides a leading separator (no visible previous item)', () => {
    const items = [sep('s'), item('a')];
    expect(cleanupSeparatorVisibility(items, 2)).toEqual([false, true]);
  });

  it('hides a trailing separator (no visible next item within the count)', () => {
    const items = [item('a'), sep('s')];
    expect(cleanupSeparatorVisibility(items, 2)).toEqual([true, false]);
  });

  it('hides a separator whose next neighbour is cut off by the visible count', () => {
    const items = [item('a'), sep('s'), item('b')];
    // VisibleCount 2 → index 2 (item b) not visible → separator dangling
    expect(cleanupSeparatorVisibility(items, 2)).toEqual([true, false, false]);
  });

  it('hides a separator sitting next to another separator', () => {
    const items = [item('a'), sep('s1'), sep('s2'), item('b')];
    expect(cleanupSeparatorVisibility(items, 4)).toEqual([
      true,
      false,
      false,
      true
    ]);
  });
});
