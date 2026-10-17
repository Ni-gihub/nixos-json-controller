import assert from "node:assert/strict";
import test from "node:test";
import {
  classifySearchUrlTransition,
  isCurrentSearchRequest,
} from "./discover-search-state.ts";

test("ignores a stale URL query while a clear operation is pending", () => {
  assert.deepEqual(
    classifySearchUrlTransition("firefox", "", "PUSH"),
    { kind: "stale" },
  );
});

test("acknowledges the internal URL update once the expected query arrives", () => {
  assert.deepEqual(
    classifySearchUrlTransition("", "", "PUSH"),
    { kind: "internal" },
  );
});

test("treats browser back/forward as external navigation", () => {
  assert.deepEqual(
    classifySearchUrlTransition("firefox", "", "POP"),
    { kind: "external", query: "firefox" },
  );
});

test("syncs an external URL change when no internal update is pending", () => {
  assert.deepEqual(
    classifySearchUrlTransition("vscode", null, "PUSH"),
    { kind: "external", query: "vscode" },
  );
});

test("does not accept results from an older search request", () => {
  assert.equal(isCurrentSearchRequest(4, 5), false);
  assert.equal(isCurrentSearchRequest(5, 5), true);
});
