import assert from "node:assert/strict";
import test from "node:test";
import {
  isRecommendationCacheFresh,
  loadBundledRecommendations,
  loadLatestRecommendations,
  parseRecommendationData,
  readCachedRecommendations,
  retainUsableRecommendationData,
  RECOMMENDATION_CACHE_TTL_MS,
  writeCachedRecommendations,
} from "./recommendation-data.ts";

const validData = {
  source: "Arch Linux pkgstats",
  sourceUrl: "https://pkgstats.archlinux.de/api/doc",
  generatedAt: "2026-10-01T00:00:00.000Z",
  period: { startMonth: 202510, endMonth: 202609 },
  recommendations: [
    {
      id: "firefox",
      archPackage: "firefox",
      popularity: 63.53,
      count: 123,
      samples: 456,
    },
  ],
};

function mockResponse(payload, ok = true) {
  return {
    ok,
    json: async () => payload,
  };
}

test("parses valid recommendation JSON", () => {
  const parsed = parseRecommendationData(validData);
  assert.equal(parsed?.recommendations[0]?.id, "firefox");
  assert.equal(parsed?.period?.startMonth, 202510);
});

test("rejects unsafe Nix attribute IDs and malformed popularity values", () => {
  const unsafe = structuredClone(validData);
  unsafe.recommendations[0].id = "firefox; builtins.abort";
  assert.equal(parseRecommendationData(unsafe), undefined);

  const invalidPopularity = structuredClone(validData);
  invalidPopularity.recommendations[0].popularity = 101;
  assert.equal(parseRecommendationData(invalidPopularity), undefined);
});

test("deduplicates recommendation IDs", () => {
  const duplicated = structuredClone(validData);
  duplicated.recommendations.push({ ...duplicated.recommendations[0] });
  assert.equal(parseRecommendationData(duplicated)?.recommendations.length, 1);
});

test("uses bundled data when it can be read", async () => {
  const data = await loadBundledRecommendations(async () =>
    mockResponse(validData),
  );
  assert.equal(data.recommendations.length, 1);
});

test("returns an empty safe value when the bundled resource is unavailable", async () => {
  const data = await loadBundledRecommendations(async () => {
    throw new Error("offline");
  });
  assert.equal(data.recommendations.length, 0);
});

test("does not replace existing data with an empty remote list", async () => {
  const emptyData = { ...validData, recommendations: [] };
  const data = await loadLatestRecommendations(async () =>
    mockResponse(emptyData),
  );
  assert.equal(data, undefined);
});


test("prefers non-empty bundled recommendations over retained data", () => {
  const bundled = { ...validData, recommendations: [{ ...validData.recommendations[0], id: "chromium" }] };
  const retained = validData;

  assert.equal(retainUsableRecommendationData(bundled, retained), bundled);
});

test("retains existing recommendations when bundled data is empty", () => {
  const emptyBundled = { ...validData, recommendations: [] };

  assert.equal(retainUsableRecommendationData(emptyBundled, validData), validData);
});

test("uses empty bundled data when no usable existing data is available", () => {
  const emptyBundled = { ...validData, recommendations: [] };

  assert.equal(retainUsableRecommendationData(emptyBundled, null), emptyBundled);
});

function createMemoryStorage() {
  const values = new Map();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
    removeItem: (key) => values.delete(key),
  };
}

test("persists and reads valid recommendation cache data", () => {
  const storage = createMemoryStorage();
  writeCachedRecommendations(validData, 1_800_000_000_000, storage);

  const cached = readCachedRecommendations(1_800_000_000_001, storage);
  assert.equal(cached?.data.recommendations[0]?.id, "firefox");
  assert.equal(cached?.checkedAt, 1_800_000_000_000);
});

test("reuses recommendation cache for 24 hours and expires it afterwards", () => {
  const storage = createMemoryStorage();
  const checkedAt = 1_800_000_000_000;
  writeCachedRecommendations(validData, checkedAt, storage);
  const cached = readCachedRecommendations(checkedAt + 1, storage);

  assert.ok(cached);
  assert.equal(
    isRecommendationCacheFresh(cached, checkedAt + RECOMMENDATION_CACHE_TTL_MS - 1),
    true,
  );
  assert.equal(
    isRecommendationCacheFresh(cached, checkedAt + RECOMMENDATION_CACHE_TTL_MS),
    false,
  );
});

test("ignores and removes malformed cache entries", () => {
  const storage = createMemoryStorage();
  storage.setItem("nxc.app-store.recommendations.v1", "{invalid json");

  assert.equal(readCachedRecommendations(1_800_000_000_000, storage), undefined);
  assert.equal(storage.getItem("nxc.app-store.recommendations.v1"), null);
});

test("does not persist empty recommendation data", () => {
  const storage = createMemoryStorage();
  writeCachedRecommendations(
    { ...validData, recommendations: [] },
    1_800_000_000_000,
    storage,
  );

  assert.equal(storage.getItem("nxc.app-store.recommendations.v1"), null);
});
