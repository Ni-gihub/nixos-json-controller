import assert from "node:assert/strict";
import test from "node:test";
import {
  buildRecommendationFallbackApps,
  formatUnresolvedRecommendationMessage,
} from "./recommendation-fallback.ts";

test("builds useful fallback cards for curated package IDs", () => {
  const apps = buildRecommendationFallbackApps([
    "firefox",
    "kdePackages.okular",
    "obs-studio",
  ]);

  assert.deepEqual(
    apps.map((app) => app.id),
    ["firefox", "kdePackages.okular", "obs-studio"],
  );
  assert.equal(apps[0].name, "Firefox");
  assert.equal(apps[1].name, "Okular");
  assert.equal(apps[2].name, "OBS Studio");
  assert.ok(apps.every((app) => app.description.length > 0));
  assert.ok(apps.every((app) => app.version === ""));
});

test("does not generate cards for IDs outside the curated allowlist", () => {
  assert.deepEqual(buildRecommendationFallbackApps(["builtins.abort", "unknown-app"]), []);
});

test("describes recommendation IDs that have no fallback metadata", () => {
  assert.equal(
    formatUnresolvedRecommendationMessage(["unknown-app", "other-unknown"]),
    "表示用の情報がないため表示できないアプリ: unknown-app, other-unknown",
  );
  assert.equal(formatUnresolvedRecommendationMessage([]), undefined);
  assert.equal(
    formatUnresolvedRecommendationMessage(["unknown-app", "unknown-app"]),
    "表示用の情報がないため表示できないアプリ: unknown-app",
  );
});

test("provides fallback metadata for corrected Nixpkgs recommendation IDs", () => {
  const ids = [
    "dbeaver-bin",
    "kdePackages.falkon",
    "onlyoffice-desktopeditors",
    "krusader",
  ];
  const apps = buildRecommendationFallbackApps(ids);

  assert.deepEqual(apps.map((app) => app.id), ids);
  assert.deepEqual(apps.map((app) => app.name), [
    "DBeaver",
    "Falkon",
    "ONLYOFFICE",
    "Krusader",
  ]);
});
