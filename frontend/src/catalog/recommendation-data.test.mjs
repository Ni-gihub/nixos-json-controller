import assert from "node:assert/strict";
import test from "node:test";
import {
  isRecommendationCacheFresh,
  loadBundledRecommendations,
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
      genre: "ブラウザ",
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

test("rejects unsafe IDs, malformed popularity, and missing genres", () => {
  const unsafe = structuredClone(validData);
  unsafe.recommendations[0].id = "firefox; builtins.abort";
  assert.equal(parseRecommendationData(unsafe), undefined);

  const invalidPopularity = structuredClone(validData);
  invalidPopularity.recommendations[0].popularity = 101;
  assert.equal(parseRecommendationData(invalidPopularity), undefined);

  const missingGenre = structuredClone(validData);
  delete missingGenre.recommendations[0].genre;
  assert.equal(parseRecommendationData(missingGenre), undefined);
});

test("deduplicates recommendation IDs", () => {
  const duplicated = structuredClone(validData);
  duplicated.recommendations.push({ ...duplicated.recommendations[0] });
  assert.equal(parseRecommendationData(duplicated)?.recommendations.length, 1);
});


test("accepts expanded recommendation lists and rejects oversized payloads", () => {
  const makeList = (length) => ({
    ...validData,
    recommendations: Array.from({ length }, (_, index) => ({
      id: `test-app-${index}`,
      archPackage: `test-app-${index}`,
      genre: "テスト",
      popularity: 50,
      count: 10,
      samples: 20,
    })),
  });

  assert.equal(parseRecommendationData(makeList(59))?.recommendations.length, 59);
  assert.equal(parseRecommendationData(makeList(150))?.recommendations.length, 150);
  assert.equal(parseRecommendationData(makeList(251)), undefined);
});

test("uses bundled data when it can be read", async () => {
  const data = await loadBundledRecommendations(async () =>
    mockResponse(validData),
  );
  assert.equal(data.recommendations.length, 1);
});

test("uses NXC-curated recommendations when distributor data is unavailable", async () => {
  const data = await loadBundledRecommendations(async () => {
    throw new Error("offline");
  });
  assert.ok(data.recommendations.length > 0);
  assert.equal(data.source, "NXC curated recommendations");
  assert.equal(data.recommendations[0]?.popularity, null);
  assert.equal(data.recommendations[0]?.count, null);
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

test("keeps cached popularity data when the bundled candidate has only curated entries", () => {
  const curated = {
    ...validData,
    source: "NXC curated recommendations",
    generatedAt: null,
    period: null,
    recommendations: [{ ...validData.recommendations[0], popularity: null, count: null, samples: null }],
  };

  assert.equal(retainUsableRecommendationData(curated, validData), validData);
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
  storage.setItem("nxc.app-store.recommendations.v2", "{invalid json");

  assert.equal(readCachedRecommendations(1_800_000_000_000, storage), undefined);
  assert.equal(storage.getItem("nxc.app-store.recommendations.v2"), null);
});

test("does not persist empty recommendation data", () => {
  const storage = createMemoryStorage();
  writeCachedRecommendations(
    { ...validData, recommendations: [] },
    1_800_000_000_000,
    storage,
  );

  assert.equal(storage.getItem("nxc.app-store.recommendations.v2"), null);
});

test("does not persist curated data that has no external popularity statistics", () => {
  const storage = createMemoryStorage();
  const curated = {
    ...validData,
    source: "NXC curated recommendations",
    generatedAt: null,
    period: null,
    recommendations: [{ ...validData.recommendations[0], popularity: null, count: null, samples: null }],
  };

  writeCachedRecommendations(curated, 1_800_000_000_000, storage);
  assert.equal(storage.getItem("nxc.app-store.recommendations.v2"), null);
});
