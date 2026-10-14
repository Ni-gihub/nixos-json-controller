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
    const render = (query) =>
      renderToStaticMarkup(
        React.createElement(
          DiscoverPreSearch,
          {
            query,
            hasRecommendations: true,
            generatedAt: "2026-10-15T00:00:00.000Z",
            loading: false,
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
  } finally {
    await vite.close();
  }
});
