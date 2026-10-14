import assert from "node:assert/strict";
import test from "node:test";
import {
  loadBundledRecommendations,
  loadLatestRecommendations,
  parseRecommendationData,
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
