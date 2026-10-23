import packageMap from "../../scripts/recommendation-package-map.json" with { type: "json" };

export type RecommendationRecord = {
  id: string;
  archPackage: string;
  genre: string;
  popularity: number | null;
  count: number | null;
  samples: number | null;
};

export type RecommendationPeriod = {
  startMonth: number;
  endMonth: number;
};

export type RecommendationData = {
  source: string;
  sourceUrl: string;
  generatedAt: string | null;
  period: RecommendationPeriod | null;
  recommendations: RecommendationRecord[];
};

export type RecommendationDocument = RecommendationData;

export type CachedRecommendations = {
  data: RecommendationDocument;
  checkedAt: number;
};

type StorageLike = Pick<Storage, "getItem" | "setItem" | "removeItem">;

const BUNDLED_DATA_URL = "/data/recommendations.json";
const RECOMMENDATION_CACHE_KEY = "nxc.app-store.recommendations.v2";
export const RECOMMENDATION_CACHE_TTL_MS = 24 * 60 * 60 * 1000;

const EMPTY_DATA: RecommendationDocument = {
  source: "NXC curated recommendations",
  sourceUrl: "https://github.com/nxc-project/nixos-json-controller",
  generatedAt: null,
  period: null,
  recommendations: [],
};

/** Get localStorage when the WebView makes it available. */
function getBrowserStorage(): StorageLike | undefined {
  try {
    return typeof window === "undefined" ? undefined : window.localStorage;
  } catch {
    return undefined;
  }
}

/** Check whether an unknown JSON value is a non-array object. */
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Check whether a numeric month uses the pkgstats YYYYMM format. */
function isValidMonth(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value >= 200001 &&
    value <= 299912 &&
    value % 100 >= 1 &&
    value % 100 <= 12
  );
}

/** Validate remote or bundled JSON before it influences package lookup. */
export function parseRecommendationData(
  value: unknown,
): RecommendationDocument | undefined {
  if (!isRecord(value) || !Array.isArray(value.recommendations)) {
    return undefined;
  }

  const recommendations: RecommendationRecord[] = [];
  const seen = new Set<string>();
  for (const item of value.recommendations) {
    if (
      !isRecord(item) ||
      typeof item.id !== "string" ||
      !/^[A-Za-z_][A-Za-z0-9_'-]*(\.[A-Za-z_][A-Za-z0-9_'-]*)*$/.test(
        item.id,
      ) ||
      typeof item.archPackage !== "string" ||
      !/^[A-Za-z0-9@+_.-]+$/.test(item.archPackage) ||
      typeof item.genre !== "string" ||
      item.genre.trim().length === 0 ||
      item.genre.length > 40 ||
      (item.popularity !== null &&
        (typeof item.popularity !== "number" ||
          !Number.isFinite(item.popularity) ||
          item.popularity < 0 ||
          item.popularity > 100)) ||
      (item.count !== null &&
        (typeof item.count !== "number" ||
          !Number.isInteger(item.count) ||
          item.count < 0)) ||
      (item.samples !== null &&
        (typeof item.samples !== "number" ||
          !Number.isInteger(item.samples) ||
          item.samples < 0))
    ) {
      return undefined;
    }

    if (!seen.has(item.id)) {
      recommendations.push({
        id: item.id,
        archPackage: item.archPackage,
        genre: item.genre,
        popularity: item.popularity as number | null,
        count: item.count as number | null,
        samples: item.samples as number | null,
      });
      seen.add(item.id);
    }
  }

  if (recommendations.length > 250) return undefined;

  let period: RecommendationPeriod | null = null;
  if (value.period !== null) {
    if (
      !isRecord(value.period) ||
      !isValidMonth(value.period.startMonth) ||
      !isValidMonth(value.period.endMonth) ||
      value.period.startMonth > value.period.endMonth
    ) {
      return undefined;
    }
    period = {
      startMonth: value.period.startMonth,
      endMonth: value.period.endMonth,
    };
  }

  const generatedAt =
    typeof value.generatedAt === "string" &&
    !Number.isNaN(Date.parse(value.generatedAt))
      ? value.generatedAt
      : null;

  return {
    source: typeof value.source === "string" ? value.source : EMPTY_DATA.source,
    sourceUrl:
      typeof value.sourceUrl === "string"
        ? value.sourceUrl
        : EMPTY_DATA.sourceUrl,
    generatedAt,
    period,
    recommendations,
  };
}

/** Read the last usable recommendation result persisted by the app. */
export function readCachedRecommendations(
  now = Date.now(),
  storage: StorageLike | undefined = getBrowserStorage(),
): CachedRecommendations | undefined {
  if (!storage) return undefined;

  try {
    const raw = storage.getItem(RECOMMENDATION_CACHE_KEY);
    if (!raw) return undefined;

    const envelope: unknown = JSON.parse(raw);
    if (
      !isRecord(envelope) ||
      envelope.version !== 1 ||
      typeof envelope.checkedAt !== "number" ||
      !Number.isFinite(envelope.checkedAt) ||
      envelope.checkedAt < 0 ||
      envelope.checkedAt > now
    ) {
      storage.removeItem(RECOMMENDATION_CACHE_KEY);
      return undefined;
    }

    const data = parseRecommendationData(envelope.data);
    if (!data || data.recommendations.length === 0) {
      storage.removeItem(RECOMMENDATION_CACHE_KEY);
      return undefined;
    }

    return { data, checkedAt: envelope.checkedAt };
  } catch {
    // Remove malformed JSON when possible; storage errors must not break app startup.
    try {
      storage.removeItem(RECOMMENDATION_CACHE_KEY);
    } catch {
      // Storage can be unavailable in restricted WebView contexts.
    }
    return undefined;
  }
}

/** Persist only validated, non-empty recommendation data. */
export function writeCachedRecommendations(
  data: RecommendationDocument,
  checkedAt = Date.now(),
  storage: StorageLike | undefined = getBrowserStorage(),
): void {
  if (
    !storage ||
    data.recommendations.length === 0 ||
    !data.recommendations.some((item) => item.popularity !== null) ||
    !Number.isFinite(checkedAt) ||
    checkedAt < 0
  ) {
    return;
  }

  try {
    storage.setItem(
      RECOMMENDATION_CACHE_KEY,
      JSON.stringify({ version: 1, checkedAt, data }),
    );
  } catch {
    // The app must still work if persistent browser storage is unavailable.
  }
}

/** Cached JSON is reused for 24 hours; expired data remains usable while refreshing. */
export function isRecommendationCacheFresh(
  cached: CachedRecommendations,
  now = Date.now(),
): boolean {
  const age = now - cached.checkedAt;
  return age >= 0 && age < RECOMMENDATION_CACHE_TTL_MS;
}

/**
 * Prefer a usable candidate, but preserve existing usable data when the candidate is empty.
 * This keeps the UI populated when persistent storage and the bundled document are unavailable.
 */
export function retainUsableRecommendationData(
  candidate: RecommendationDocument,
  existing: RecommendationDocument | null | undefined,
): RecommendationDocument {
  if (candidate.recommendations.some((item) => item.popularity !== null)) {
    return candidate;
  }
  if (existing?.recommendations.length) return existing;
  return candidate;
}

/** Fetch and validate one recommendation document, treating network errors as unavailable data. */
async function readRecommendationData(
  url: string,
  fetcher: typeof fetch,
): Promise<RecommendationDocument | undefined> {
  try {
    const response = await fetcher(url, {
      cache: "no-store",
      signal: AbortSignal.timeout(5_000),
    });
    if (!response.ok) return undefined;

    return parseRecommendationData(await response.json());
  } catch {
    return undefined;
  }
}

const MAX_CURATED_ITEMS_PER_GENRE = 12;

/**
 * Build an offline-safe recommendation list from NXC's own package/genre map.
 * These records do not contain popularity statistics from a third party.
 */
export function buildCuratedRecommendationData(): RecommendationDocument {
  const recommendations: RecommendationRecord[] = [];
  const includedIds = new Set<string>();
  const genreCounts = new Map<string, number>();

  for (const item of packageMap) {
    if (includedIds.has(item.id)) continue;
    const count = genreCounts.get(item.genre) ?? 0;
    if (count >= MAX_CURATED_ITEMS_PER_GENRE) continue;

    recommendations.push({
      id: item.id,
      archPackage: item.archPackage,
      genre: item.genre,
      popularity: null,
      count: null,
      samples: null,
    });
    includedIds.add(item.id);
    genreCounts.set(item.genre, count + 1);
  }

  return {
    source: "NXC curated recommendations",
    sourceUrl:
      "https://github.com/nxc-project/nixos-json-controller/blob/main/frontend/scripts/recommendation-package-map.json",
    generatedAt: null,
    period: null,
    recommendations,
  };
}

/**
 * Prefer locally generated distributor data when available. Without it, use NXC's
 * curated map and do not invent or imply third-party popularity statistics.
 */
export async function loadBundledRecommendations(
  fetcher: typeof fetch = fetch,
): Promise<RecommendationDocument> {
  return (
    (await readRecommendationData(BUNDLED_DATA_URL, fetcher)) ??
    buildCuratedRecommendationData()
  );
}
