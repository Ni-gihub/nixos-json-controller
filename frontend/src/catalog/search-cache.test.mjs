import assert from "node:assert/strict";
import test from "node:test";
import { SearchPageCache } from "./search-cache.ts";

const page = (results, nextOffset, hasMore) => ({
  results: results.map((id) => ({ id })),
  nextOffset,
  hasMore,
});

test("loads pages on demand and reuses pages already fetched for a query", async () => {
  const cache = new SearchPageCache(4);
  let calls = 0;

  const first = await cache.load("Firefox", 0, 40, async () => {
    calls += 1;
    return page(["firefox", "firefox-esr"], 40, true);
  });
  const repeatedFirst = await cache.load("firefox", 0, 40, async () => {
    calls += 1;
    return page(["should-not-load"], 40, true);
  });
  const second = await cache.load("firefox", 40, 40, async () => {
    calls += 1;
    return page(["firefox-nightly"], 41, false);
  });
  const repeatedSecond = await cache.load("firefox", 40, 40, async () => {
    calls += 1;
    return page(["should-not-load"], 41, false);
  });

  assert.deepEqual(first.results.map((item) => item.id), [
    "firefox",
    "firefox-esr",
  ]);
  assert.strictEqual(repeatedFirst, first);
  assert.deepEqual(second.results.map((item) => item.id), ["firefox-nightly"]);
  assert.strictEqual(repeatedSecond, second);
  assert.equal(calls, 2);
  assert.deepEqual(
    ["firefox", "firefox-esr", "firefox-nightly"].map((id) =>
      cache.findCached("FIREFOX", id)?.id,
    ),
    ["firefox", "firefox-esr", "firefox-nightly"],
  );
});

test("deduplicates overlapping results across pages", async () => {
  const cache = new SearchPageCache(4);

  await cache.load("browser", 0, 40, async () =>
    page(["firefox", "chromium"], 40, true),
  );
  const next = await cache.load("browser", 40, 40, async () =>
    page(["chromium", "epiphany"], 42, false),
  );

  assert.deepEqual(next.results.map((item) => item.id), ["epiphany"]);
  assert.equal(cache.findCached("browser", "chromium")?.id, "chromium");
  assert.equal(cache.findCached("browser", "epiphany")?.id, "epiphany");
});

test("deduplicates concurrent requests for the same query and offset", async () => {
  const cache = new SearchPageCache(4);
  let calls = 0;
  let resolveLoader;
  const pendingResult = new Promise((resolve) => {
    resolveLoader = resolve;
  });
  const loader = () => {
    calls += 1;
    return pendingResult;
  };

  const firstRequest = cache.load("vscode", 0, 40, loader);
  const secondRequest = cache.load("vscode", 0, 40, loader);
  await Promise.resolve();
  assert.equal(calls, 1);

  resolveLoader(page(["vscode"], 1, false));
  const [first, second] = await Promise.all([firstRequest, secondRequest]);
  assert.strictEqual(first, second);
});

test("keeps queries separate and bounds the number of cached queries", async () => {
  const cache = new SearchPageCache(2);

  await cache.load("firefox", 0, 40, async () =>
    page(["firefox"], 1, false),
  );
  await cache.load("vscode", 0, 40, async () =>
    page(["vscode"], 1, false),
  );
  // Accessing firefox makes it most-recently used.
  assert.equal(cache.findCached("firefox", "firefox")?.id, "firefox");
  await cache.load("chromium", 0, 40, async () =>
    page(["chromium"], 1, false),
  );

  assert.equal(cache.findCached("firefox", "firefox")?.id, "firefox");
  assert.equal(cache.findCached("vscode", "vscode"), undefined);
  assert.equal(cache.findCached("chromium", "chromium")?.id, "chromium");
});
