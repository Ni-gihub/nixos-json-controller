export type VirtualRange = {
  startIndex: number;
  endIndex: number;
  topOffset: number;
  bottomOffset: number;
};

/** Build cumulative row offsets from measured heights and a safe estimate. */
export function buildRowOffsets(
  ids: string[],
  measuredHeights: ReadonlyMap<string, number>,
  estimatedHeight: number,
): number[] {
  const estimate = Math.max(1, estimatedHeight);
  const offsets = [0];

  for (const id of ids) {
    const measured = measuredHeights.get(id);
    const height =
      measured !== undefined && Number.isFinite(measured) && measured > 0
        ? measured
        : estimate;
    offsets.push(offsets[offsets.length - 1] + height);
  }

  return offsets;
}

function indexAtOffset(offsets: number[], offset: number): number {
  const itemCount = Math.max(0, offsets.length - 1);
  if (itemCount === 0) return 0;

  let low = 0;
  let high = itemCount;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    if (offsets[middle + 1] <= offset) low = middle + 1;
    else high = middle;
  }
  return Math.min(low, itemCount);
}

/** Return the rows around the viewport plus overscan, with spacer heights. */
export function getVirtualRange(
  offsets: number[],
  viewportStart: number,
  viewportEnd: number,
  overscan: number,
): VirtualRange {
  const itemCount = Math.max(0, offsets.length - 1);
  if (itemCount === 0) {
    return {
      startIndex: 0,
      endIndex: 0,
      topOffset: 0,
      bottomOffset: 0,
    };
  }

  const start = indexAtOffset(offsets, Math.max(0, viewportStart));
  const end = indexAtOffset(offsets, Math.max(0, viewportEnd));
  const padding = Math.max(0, Math.floor(overscan));
  const startIndex = Math.max(0, start - padding);
  const endIndex = Math.min(itemCount, Math.max(start + 1, end) + padding);

  return {
    startIndex,
    endIndex,
    topOffset: offsets[startIndex],
    bottomOffset: Math.max(0, offsets[itemCount] - offsets[endIndex]),
  };
}
