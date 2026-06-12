export interface ToolbarOverflowItem {
  id: string;
  isSeparator?: boolean;
}

export const totalWidth = (
  itemWidths: number[],
  count: number,
  gap: number
): number => {
  if (count <= 0) {
    return 0;
  }
  const itemsWidth = itemWidths
    .slice(0, count)
    .reduce((sum, width) => sum + width, 0);
  return itemsWidth + gap * (count - 1);
};

export const calculateVisibleCount = (
  containerWidth: number,
  itemWidths: number[],
  overflowButtonWidth: number,
  gap: number
): { visibleCount: number; showOverflow: boolean } => {
  const itemCount = itemWidths.length;

  if (itemCount === 0 || containerWidth <= 0) {
    return { showOverflow: false, visibleCount: 0 };
  }

  const fitsAll = totalWidth(itemWidths, itemCount, gap) <= containerWidth;
  if (fitsAll) {
    return { showOverflow: false, visibleCount: itemCount };
  }

  const availableWithOverflow = containerWidth - overflowButtonWidth - gap;
  let visibleCount = 0;

  for (let i = 0; i < itemCount; i++) {
    const nextWidth = totalWidth(itemWidths, i + 1, gap);
    if (nextWidth > availableWithOverflow) {
      break;
    }
    visibleCount = i + 1;
  }

  return { showOverflow: true, visibleCount };
};

export const cleanupSeparatorVisibility = (
  items: ToolbarOverflowItem[],
  visibleCount: number
): boolean[] =>
  items.map((item, index) => {
    if (index >= visibleCount) {
      return false;
    }

    if (!item.isSeparator) {
      return true;
    }

    const prevVisible = index > 0 && !items[index - 1]?.isSeparator;
    const nextVisible =
      index + 1 < visibleCount && !items[index + 1]?.isSeparator;

    return prevVisible && nextVisible;
  });
