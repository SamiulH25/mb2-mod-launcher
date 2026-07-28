export const MOD_ROW_HEIGHT = 54;
export const VISIBLE_ROW_COUNT = 14;
export const VIRTUAL_LIST_THRESHOLD = VISIBLE_ROW_COUNT;
export const VIRTUAL_OVERSCAN = 4;

export function getVisibleRange(
  scrollTop: number,
  viewportHeight: number,
  itemCount: number,
  itemHeight: number,
  overscan = VIRTUAL_OVERSCAN,
): { start: number; end: number } {
  if (itemCount === 0) {
    return { start: 0, end: 0 };
  }

  const start = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  const end = Math.min(
    itemCount,
    Math.ceil((scrollTop + viewportHeight) / itemHeight) + overscan,
  );

  return { start, end: Math.max(start, end) };
}
