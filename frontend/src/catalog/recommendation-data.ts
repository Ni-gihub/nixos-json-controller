export type RecommendationRecord = {
  id: string;
  archPackage: string;
  popularity: number;
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

const BUNDLED_DATA_URL = "/data/recommendations.json";
const LATEST_DATA_URL =
  "https://raw.githubusercontent.com/Ni-gihub/nixos-json-controller/main/frontend/public/data/recommendations.json";
const EMPTY_DATA: RecommendationData = {
  source: "Arch Linux pkgstats",
  sourceUrl: "https://pkgstats.archlinux.de/api/doc",
  generatedAt: null,
  period: null,
  recommendations: [],
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

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
): RecommendationData | undefined {
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
      typeof item.popularity !== "number" ||
      !Number.isFinite(item.popularity) ||
      item.popularity < 0 ||
      item.popularity > 100 ||
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
        popularity: item.popularity,
        count: item.count as number | null,
        samples: item.samples as number | null,
      });
      seen.add(item.id);
    }
  }

  if (recommendations.length > 50) return undefined;

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

async function readRecommendationData(
  url: string,
  fetcher: typeof fetch,
): Promise<RecommendationData | undefined> {
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

/** Read the packaged copy first so initial UI does not depend on the network. */
export async function loadBundledRecommendations(
  fetcher: typeof fetch = fetch,
): Promise<RecommendationData> {
  return (
    (await readRecommendationData(BUNDLED_DATA_URL, fetcher)) ?? EMPTY_DATA
  );
}

/** Refresh from the repository's generated JSON, retaining bundled data on failure. */
export async function loadLatestRecommendations(
  fetcher: typeof fetch = fetch,
): Promise<RecommendationData | undefined> {
  const separator = LATEST_DATA_URL.includes("?") ? "&" : "?";
  const latest = await readRecommendationData(
    `${LATEST_DATA_URL}${separator}t=${Date.now()}`,
    fetcher,
  );

  // An empty remote file should not replace the last known usable recommendation list.
  return latest?.recommendations.length ? latest : undefined;
}
