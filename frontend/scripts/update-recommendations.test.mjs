import assert from "node:assert/strict";
import test from "node:test";
import { buildRecommendations, hasSameRecommendationContent } from "./update-recommendations.mjs";

test("ranks only explicitly mapped GUI candidates by pkgstats popularity", () => {
  const stats = [
    { name: "chromium", popularity: 48.8, count: 488, samples: 1000 },
    { name: "firefox", popularity: 63.53, count: 635, samples: 1000 },
    { name: "systemd", popularity: 99.9, count: 999, samples: 1000 },
    { name: "gimp", popularity: 8, count: 80, samples: 1000 },
  ];
  const map = [
    { archPackage: "firefox", id: "firefox" },
    { archPackage: "chromium", id: "chromium" },
    { archPackage: "gimp", id: "gimp" },
    { archPackage: "not-installed", id: "not-installed" },
  ];

  const recommendations = buildRecommendations(stats, map);
  assert.deepEqual(
    recommendations.map((item) => item.id),
    ["firefox", "chromium", "gimp"],
  );
  assert.equal(recommendations[0].popularity, 63.53);
  assert.equal(recommendations.some((item) => item.id === "systemd"), false);
});

test("limits the output to the supported number of recommendations", () => {
  const stats = Array.from({ length: 20 }, (_, index) => ({
    name: `app-${index}`,
    popularity: 20 - index,
    count: 100 - index,
    samples: 100,
  }));
  const map = stats.map((item) => ({
    archPackage: item.name,
    id: `app-${item.name.slice(4)}`,
  }));

  assert.equal(buildRecommendations(stats, map).length, 12);
});

test("rejects invalid package mappings", () => {
  assert.throws(
    () => buildRecommendations([], [{ archPackage: "firefox", id: "firefox; abort" }]),
    /invalid entry/,
  );
});
