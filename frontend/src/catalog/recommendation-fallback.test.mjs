import assert from "node:assert/strict";
import test from "node:test";
import { buildRecommendationFallbackApps } from "./recommendation-fallback.ts";

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
