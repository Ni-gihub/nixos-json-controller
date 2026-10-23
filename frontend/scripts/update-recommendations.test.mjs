import { readFile } from "node:fs/promises";
import assert from "node:assert/strict";
import test from "node:test";
import {
  buildRecommendations,
  hasSameRecommendationContent,
} from "./update-recommendations.mjs";

test("ranks only explicitly mapped GUI candidates by pkgstats popularity", () => {
  const stats = [
    { name: "chromium", popularity: 48.8, count: 488, samples: 1000 },
    { name: "firefox", popularity: 63.53, count: 635, samples: 1000 },
    { name: "systemd", popularity: 99.9, count: 999, samples: 1000 },
    { name: "gimp", popularity: 8, count: 80, samples: 1000 },
  ];
  const map = [
    { archPackage: "firefox", id: "firefox", genre: "ブラウザ" },
    { archPackage: "chromium", id: "chromium", genre: "ブラウザ" },
    { archPackage: "gimp", id: "gimp", genre: "クリエイティブ" },
    { archPackage: "not-installed", id: "not-installed", genre: "ユーティリティ" },
  ];

  const recommendations = buildRecommendations(stats, map);
  assert.deepEqual(
    recommendations.map((item) => item.id),
    ["firefox", "chromium", "gimp"],
  );
  assert.equal(recommendations[0].popularity, 63.53);
  assert.equal(recommendations.some((item) => item.id === "systemd"), false);
});

test("limits each genre to twelve recommendations while keeping other genres", () => {
  const stats = Array.from({ length: 40 }, (_, index) => ({
    name: `app-${index}`,
    popularity: 20 - index,
    count: 100 - index,
    samples: 100,
  }));
  const map = stats.map((item, index) => ({
    archPackage: item.name,
    id: `app-${item.name.slice(4)}`,
    genre: index % 2 === 0 ? "ブラウザ" : "メディア",
  }));

  const recommendations = buildRecommendations(stats, map);
  assert.equal(recommendations.length, 24);
  assert.equal(
    recommendations.filter((item) => item.genre === "ブラウザ").length,
    12,
  );
  assert.equal(
    recommendations.filter((item) => item.genre === "メディア").length,
    12,
  );
});

test("does not emit duplicate IDs when Arch packages map to one Nix attribute", () => {
  const stats = [
    { name: "editor", popularity: 90, count: 90, samples: 100 },
    { name: "editor-nightly", popularity: 80, count: 80, samples: 100 },
  ];
  const map = [
    { archPackage: "editor", id: "editor", genre: "開発ツール" },
    { archPackage: "editor-nightly", id: "editor", genre: "開発ツール" },
  ];

  assert.deepEqual(
    buildRecommendations(stats, map).map((item) => item.id),
    ["editor"],
  );
});

test("rejects invalid package mappings", () => {
  assert.throws(
    () => buildRecommendations([], [{ archPackage: "firefox", id: "firefox; abort", genre: "ブラウザ" }]),
    /invalid entry/,
  );
});

test("keeps unchanged recommendation data even when generatedAt differs", () => {
  const existing = {
    source: "Arch Linux pkgstats",
    sourceUrl: "https://pkgstats.archlinux.de/api/doc",
    generatedAt: "2026-10-12T08:17:00.000Z",
    period: { startMonth: 202510, endMonth: 202609 },
    recommendations: [
      {
        id: "firefox",
        archPackage: "firefox",
        genre: "ブラウザ",
        popularity: 66.79,
        count: 203545,
        samples: 304746,
      },
    ],
  };
  const next = {
    ...existing,
    generatedAt: "2026-10-19T08:17:00.000Z",
  };

  assert.equal(hasSameRecommendationContent(existing, next), true);
});

test("detects recommendation, period, and source changes", () => {
  const existing = {
    source: "Arch Linux pkgstats",
    sourceUrl: "https://pkgstats.archlinux.de/api/doc",
    generatedAt: "2026-10-12T08:17:00.000Z",
    period: { startMonth: 202510, endMonth: 202609 },
    recommendations: [
      {
        id: "firefox",
        archPackage: "firefox",
        genre: "ブラウザ",
        popularity: 66.79,
        count: 203545,
        samples: 304746,
      },
    ],
  };

  assert.equal(
    hasSameRecommendationContent(existing, {
      ...existing,
      recommendations: [
        { ...existing.recommendations[0], popularity: 66.8 },
      ],
    }),
    false,
  );
  assert.equal(
    hasSameRecommendationContent(existing, {
      ...existing,
      recommendations: [
        { ...existing.recommendations[0], genre: "メディア" },
      ],
    }),
    false,
  );
  assert.equal(
    hasSameRecommendationContent(existing, {
      ...existing,
      period: { startMonth: 202511, endMonth: 202609 },
    }),
    false,
  );
  assert.equal(
    hasSameRecommendationContent(existing, {
      ...existing,
      sourceUrl: "https://example.com/api",
    }),
    false,
  );
});

test("treats malformed existing recommendation data as changed", () => {
  const next = {
    source: "Arch Linux pkgstats",
    sourceUrl: "https://pkgstats.archlinux.de/api/doc",
    generatedAt: "2026-10-19T08:17:00.000Z",
    period: { startMonth: 202510, endMonth: 202609 },
    recommendations: [
      {
        id: "firefox",
        archPackage: "firefox",
        genre: "ブラウザ",
        popularity: 66.79,
        count: 203545,
        samples: 304746,
      },
    ],
  };

  assert.equal(hasSameRecommendationContent(null, next), false);
  assert.equal(
    hasSameRecommendationContent(
      { source: "Arch Linux pkgstats", recommendations: [] },
      next,
    ),
    false,
  );
});


test("keeps corrected Nixpkgs IDs synchronized between mapping and bundled recommendations", async () => {
  const packageMap = JSON.parse(
    await readFile(new URL("./recommendation-package-map.json", import.meta.url), "utf8"),
  );
  const bundled = JSON.parse(
    await readFile(new URL("../public/data/recommendations.json", import.meta.url), "utf8"),
  );
  const expected = [
    ["dbeaver", "dbeaver-bin"],
    ["falkon", "kdePackages.falkon"],
    ["onlyoffice-bin", "onlyoffice-desktopeditors"],
    ["krusader", "krusader"],
  ];

  for (const [archPackage, id] of expected) {
    assert.equal(
      packageMap.find((entry) => entry.archPackage === archPackage)?.id,
      id,
      `mapping for ${archPackage}`,
    );
    assert.equal(
      bundled.recommendations.find((entry) => entry.archPackage === archPackage)?.id,
      id,
      `bundled recommendation for ${archPackage}`,
    );
  }
});
