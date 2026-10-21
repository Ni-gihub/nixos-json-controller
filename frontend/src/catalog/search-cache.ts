export type CatalogSearchPage<T> = {
  results: T[];
  nextOffset: number;
  hasMore: boolean;
  resultsLimited: boolean;
};

type SearchCacheEntry<T> = {
  results: T[];
  pages: Map<string, CatalogSearchPage<T>>;
  pending: Map<string, Promise<CatalogSearchPage<T>>>;
};

/** Small LRU cache that also deduplicates in-flight page requests per query. */
export class SearchPageCache<T extends { id: string }> {
  private readonly entries = new Map<string, SearchCacheEntry<T>>();
  private readonly maxQueries: number;

  constructor(maxQueries = 4) {
    this.maxQueries = maxQueries;
  }

  /** Return a cached page or fetch and deduplicate a single in-flight request. */
  async load(
    query: string,
    offset: number,
    limit: number,
    loader: () => Promise<CatalogSearchPage<T>>,
  ): Promise<CatalogSearchPage<T>> {
    const key = query.trim().toLowerCase();
    if (!key)
      return {
        results: [],
        nextOffset: 0,
        hasMore: false,
        resultsLimited: false,
      };

    const entry = this.getOrCreateEntry(key);
    const pageKey = `${offset}:${limit}`;
    const cachedPage = entry.pages.get(pageKey);
    if (cachedPage) return cachedPage;

    const pendingPage = entry.pending.get(pageKey);
    if (pendingPage) return pendingPage;

    const request = Promise.resolve()
      .then(loader)
      .then((page) => {
        // Protect the accumulated view even if a backend page unexpectedly overlaps.
        const knownIds = new Set(
          offset === 0 ? [] : entry.results.map((item) => item.id),
        );
        const pageIds = new Set<string>();
        const uniqueResults = page.results.filter((item) => {
          if (knownIds.has(item.id) || pageIds.has(item.id)) return false;
          pageIds.add(item.id);
          return true;
        });
        const normalizedPage = { ...page, results: uniqueResults };

        if (offset === 0) {
          entry.results = uniqueResults.slice();
        } else {
          entry.results = [...entry.results, ...uniqueResults];
        }
        entry.pages.set(pageKey, normalizedPage);
        return normalizedPage;
      })
      .finally(() => {
        entry.pending.delete(pageKey);
      });

    entry.pending.set(pageKey, request);
    return request;
  }

  /** Find a package across the pages already loaded for a query. */
  findCached(query: string, id: string): T | undefined {
    const key = query.trim().toLowerCase();
    const entry = this.entries.get(key);
    if (!entry) return undefined;

    this.touch(key, entry);
    return entry.results.find((item) => item.id === id);
  }

  /** Access or create a query entry, evicting older entries when necessary. */
  private getOrCreateEntry(key: string): SearchCacheEntry<T> {
    const existing = this.entries.get(key);
    if (existing) {
      this.touch(key, existing);
      return existing;
    }

    const entry: SearchCacheEntry<T> = {
      results: [],
      pages: new Map(),
      pending: new Map(),
    };
    this.entries.set(key, entry);

    while (this.entries.size > Math.max(1, this.maxQueries)) {
      const oldest = this.entries.keys().next().value as string | undefined;
      if (oldest === undefined) break;
      this.entries.delete(oldest);
    }

    return entry;
  }

  /** Move an existing query entry to the most-recently-used position. */
  private touch(key: string, entry: SearchCacheEntry<T>): void {
    this.entries.delete(key);
    this.entries.set(key, entry);
  }
}
