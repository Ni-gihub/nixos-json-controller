import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createServer } from "vite";

test("Discover pre-search UI shows, hides, and restores recommendations as the query changes", async () => {
  const vite = await createServer({
    configFile: "./vite.config.ts",
    logLevel: "silent",
    server: { middlewareMode: true },
    appType: "custom",
  });

  try {
    const { default: DiscoverPreSearch } = await vite.ssrLoadModule(
      "/src/pages/DiscoverPreSearch.tsx",
    );

    /** Render the pre-search component with one recommendation card fixture. */
    const render = (query, props = {}) =>
      renderToStaticMarkup(
        React.createElement(
          DiscoverPreSearch,
          {
            query,
            hasRecommendations: props.hasRecommendations ?? true,
            generatedAt: "2026-10-15T00:00:00.000Z",
            loading: false,
            recommendationStatus: props.recommendationStatus ?? "ready",
            recommendationMessage: props.recommendationMessage ?? null,
            onRetry: props.onRetry ?? (() => {}),
          },
          React.createElement(
            "article",
            { "data-recommendation-card": "true" },
            React.createElement("h3", null, "Firefox"),
          ),
        ),
      );

    const emptyQuery = render("");
    assert.match(emptyQuery, /人気のアプリ/);
    assert.match(emptyQuery, /Firefox/);
    assert.match(emptyQuery, /data-recommendation-card/);

    const typingQuery = render("firefox");
    assert.doesNotMatch(typingQuery, /人気のアプリ/);
    assert.doesNotMatch(typingQuery, /data-recommendation-card/);
    assert.match(typingQuery, /検索ボタンを押してください/);

    const clearedQuery = render("");
    assert.match(clearedQuery, /人気のアプリ/);
    assert.match(clearedQuery, /Firefox/);
    assert.match(clearedQuery, /data-recommendation-card/);

    const loadingState = render("", {
      hasRecommendations: false,
      recommendationStatus: "loading",
    });
    assert.match(loadingState, /人気のアプリを読み込み中/);
    assert.match(loadingState, /初回のパッケージ評価/);

    const unavailableState = render("", {
      hasRecommendations: false,
      recommendationStatus: "unavailable",
      recommendationMessage: "nix search failed: flakes are disabled",
    });
    assert.match(unavailableState, /おすすめアプリを表示できませんでした/);
    assert.match(unavailableState, /flakes are disabled/);
    assert.match(unavailableState, /再試行/);

    const fallbackState = render("", {
      recommendationStatus: "fallback",
      recommendationMessage: "catalog lookup timed out",
    });
    assert.match(fallbackState, /同梱の基本情報でおすすめを表示/);
    assert.match(fallbackState, /catalog lookup timed out/);
    assert.match(fallbackState, /Firefox/);
  } finally {
    await vite.close();
  }
});
