import assert from "node:assert/strict";
import test from "node:test";
import { buildRowOffsets, getVirtualRange } from "./virtual-list.ts";

test("uses measured row heights and estimates unmeasured rows", () => {
  const offsets = buildRowOffsets(
    ["a", "b", "c"],
    new Map([
      ["a", 100],
      ["c", 150],
    ]),
    120,
  );

  assert.deepEqual(offsets, [0, 100, 220, 370]);
});

test("renders only visible rows and overscan while preserving spacer heights", () => {
  const offsets = buildRowOffsets(
    Array.from({ length: 100 }, (_, index) => String(index)),
    new Map(),
    100,
  );
  const range = getVirtualRange(offsets, 2_000, 2_500, 3);

  assert.equal(range.startIndex, 17);
  assert.equal(range.endIndex, 28);
  assert.equal(range.topOffset, 1_700);
  assert.equal(range.bottomOffset, 7_200);
  assert.ok(range.endIndex - range.startIndex < 20);
});

test("handles empty lists and offsets outside the measured list", () => {
  assert.deepEqual(getVirtualRange([0], 0, 500, 5), {
    startIndex: 0,
    endIndex: 0,
    topOffset: 0,
    bottomOffset: 0,
  });

  const offsets = buildRowOffsets(["a", "b"], new Map(), 100);
  const range = getVirtualRange(offsets, 1_000, 2_000, 2);
  assert.equal(range.startIndex, 0);
  assert.equal(range.endIndex, 2);
  assert.equal(range.topOffset, 0);
  assert.equal(range.bottomOffset, 0);
});
