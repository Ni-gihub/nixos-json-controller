import { invoke } from "@tauri-apps/api/core";
import type { App } from "./apps";
import {
  SearchPageCache,
  type CatalogSearchPage,
} from "./search-cache";

const PAGE_SIZE = 40;
const searchCache = new SearchPageCache<App>(4);

/** Search the catalog in stable pages and reuse already loaded query pages. */
export async function searchApps(
  query: string,
  offset = 0,
  limit = PAGE_SIZE,
): Promise<CatalogSearchPage<App>> {
  const normalized = query.trim();
  if (!normalized)
    return {
      results: [],
      nextOffset: 0,
      hasMore: false,
      resultsLimited: false,
    };

  return searchCache.load(normalized, offset, limit, () =>
    invoke<CatalogSearchPage<App>>("search_catalog", {
      query: normalized,
      offset,
      limit,
    }),
  );
}

/** Find a package by exact ID, preferring metadata already loaded by discovery. */
export async function findApp(
  id: string,
  query?: string,
): Promise<App | undefined> {
  // Search results already contain the detail metadata and resolved icon candidates.
  // Reuse all pages loaded for the current query instead of searching Nixpkgs again.
  const cachedApp = query ? searchCache.findCached(query, id) : undefined;
  if (cachedApp) return cachedApp;

  // Keep direct detail links and hard refreshes working when no search result is cached.
  const app = await invoke<App | null>("get_catalog_app", { id });
  return app ?? undefined;
}
