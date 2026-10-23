#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const frontendDirectory = path.resolve(scriptDirectory, "..");
const mapPath = path.join(scriptDirectory, "recommendation-package-map.json");
const outputPath = path.join(frontendDirectory, "public", "data", "recommendations.json");
const apiUrl = "https://pkgstats.archlinux.de/api/packages";
const sourceUrl = "https://pkgstats.archlinux.de/api/doc";
const pageSize = 10_000;
const maxOffset = 100_000;
const recommendationLimitPerGenre = 12;

/** Convert a UTC Date into the API's numeric YYYYMM representation. */
function formatMonth(date) {
  return date.getUTCFullYear() * 100 + date.getUTCMonth() + 1;
}

/** Select the latest twelve complete months for a consistent statistics window. */
function defaultPeriod(now = new Date()) {
  // The API's defaults are the most recent 12 complete months. Pin the range
  // across paginated requests so the result is consistent during the run.
  const end = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
  end.setUTCMonth(end.getUTCMonth() - 1);
  const start = new Date(Date.UTC(end.getUTCFullYear(), end.getUTCMonth(), 1));
  start.setUTCMonth(start.getUTCMonth() - 11);
  return { startMonth: formatMonth(start), endMonth: formatMonth(end) };
}

/** Fetch and validate one page of popularity statistics for the requested period. */
async function fetchPage(offset, period) {
  const url = new URL(apiUrl);
  url.searchParams.set("limit", String(pageSize));
  url.searchParams.set("offset", String(offset));
  url.searchParams.set("startMonth", String(period.startMonth));
  url.searchParams.set("endMonth", String(period.endMonth));

  const response = await fetch(url, {
    headers: { Accept: "application/json", "User-Agent": "nixos-json-controller-recommendations" },
    signal: AbortSignal.timeout(30_000),
  });

  if (!response.ok) {
    throw new Error(`pkgstats API returned HTTP ${response.status} at offset ${offset}`);
  }

  const payload = await response.json();
  if (
    !payload ||
    !Array.isArray(payload.packagePopularities) ||
    !Number.isInteger(payload.total) ||
    !Number.isInteger(payload.count)
  ) {
    throw new Error(`pkgstats API response has an unexpected shape at offset ${offset}`);
  }

  for (const item of payload.packagePopularities) {
    if (
      typeof item.name !== "string" ||
      typeof item.popularity !== "number" ||
      !Number.isFinite(item.popularity) ||
      item.popularity < 0 ||
      item.popularity > 100 ||
      !Number.isInteger(item.count) ||
      !Number.isInteger(item.samples) ||
      !Number.isInteger(item.startMonth) ||
      !Number.isInteger(item.endMonth) ||
      item.startMonth !== period.startMonth ||
      item.endMonth !== period.endMonth
    ) {
      throw new Error("pkgstats API returned an invalid package popularity record or period");
    }
  }

  return payload;
}

/** Fetch all available result pages while detecting inconsistent or incomplete responses. */
async function fetchAllPopularities(period) {
  const firstPage = await fetchPage(0, period);
  if (firstPage.total <= 0 || firstPage.packagePopularities.length === 0) {
    throw new Error("pkgstats API returned no package statistics; keeping existing data");
  }

  const results = [...firstPage.packagePopularities];
  const reportedTotal = firstPage.total;

  for (let offset = pageSize; offset < reportedTotal && offset <= maxOffset; offset += pageSize) {
    // Keep request frequency low while respecting the API's request-rate guidance.
    await new Promise((resolve) => setTimeout(resolve, 350));
    const page = await fetchPage(offset, period);
    results.push(...page.packagePopularities);

    if (page.packagePopularities.length === 0) break;
    if (page.total !== reportedTotal) {
      throw new Error("pkgstats API total changed during pagination; retry on the next run");
    }
  }

  if (results.length < Math.min(reportedTotal, maxOffset + pageSize)) {
    throw new Error("pkgstats API pagination returned incomplete data; keeping existing data");
  }

  return results;
}

/** Rank only explicitly mapped GUI candidates by their reported pkgstats popularity. */
export function buildRecommendations(popularities, packageMap) {
  const popularityByName = new Map(
    popularities.map((record) => [record.name, record]),
  );
  const uniqueMappings = new Map();
  for (const mapping of packageMap) {
    if (
      !mapping ||
      typeof mapping.archPackage !== "string" ||
      typeof mapping.id !== "string" ||
      typeof mapping.genre !== "string" ||
      mapping.genre.trim().length === 0 ||
      !/^[A-Za-z_][A-Za-z0-9_'-]*(\.[A-Za-z_][A-Za-z0-9_'-]*)*$/.test(mapping.id)
    ) {
      throw new Error("recommendation package map contains an invalid entry");
    }
    if (!uniqueMappings.has(mapping.archPackage)) {
      uniqueMappings.set(mapping.archPackage, {
        id: mapping.id,
        genre: mapping.genre,
      });
    }
  }

  const recommendations = [];
  for (const [archPackage, mapping] of uniqueMappings) {
    const statistic = popularityByName.get(archPackage);
    if (!statistic) continue;
    recommendations.push({
      id: mapping.id,
      archPackage,
      genre: mapping.genre,
      popularity: statistic.popularity,
      count: statistic.count,
      samples: statistic.samples,
    });
  }

  recommendations.sort(
    (a, b) =>
      b.popularity - a.popularity ||
      b.count - a.count ||
      a.archPackage.localeCompare(b.archPackage),
  );

  const genreCounts = new Map();
  const includedIds = new Set();
  return recommendations.filter((recommendation) => {
    if (includedIds.has(recommendation.id)) return false;
    const count = genreCounts.get(recommendation.genre) ?? 0;
    if (count >= recommendationLimitPerGenre) return false;
    includedIds.add(recommendation.id);
    genreCounts.set(recommendation.genre, count + 1);
    return true;
  });
}

/**
 * Compare recommendation payloads while ignoring generatedAt.
 * generatedAt represents when the recommendation content last changed, not every poll.
 */
function recommendationContentKey(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;

  const record = value;
  if (
    typeof record.source !== "string" ||
    typeof record.sourceUrl !== "string" ||
    typeof record.generatedAt !== "string" ||
    Number.isNaN(Date.parse(record.generatedAt)) ||
    !record.period ||
    typeof record.period !== "object" ||
    !Number.isInteger(record.period.startMonth) ||
    !Number.isInteger(record.period.endMonth) ||
    !Array.isArray(record.recommendations) ||
    record.recommendations.length === 0
  ) {
    return null;
  }

  const recommendations = [];
  for (const item of record.recommendations) {
    if (
      !item ||
      typeof item !== "object" ||
      typeof item.id !== "string" ||
      typeof item.archPackage !== "string" ||
      typeof item.genre !== "string" ||
      typeof item.popularity !== "number" ||
      !Number.isFinite(item.popularity) ||
      !Number.isInteger(item.count) ||
      !Number.isInteger(item.samples)
    ) {
      return null;
    }

    recommendations.push({
      id: item.id,
      archPackage: item.archPackage,
      genre: item.genre,
      popularity: item.popularity,
      count: item.count,
      samples: item.samples,
    });
  }

  return JSON.stringify({
    source: record.source,
    sourceUrl: record.sourceUrl,
    period: {
      startMonth: record.period.startMonth,
      endMonth: record.period.endMonth,
    },
    recommendations,
  });
}

/** Return true when only the generation timestamp differs from usable data. */
export function hasSameRecommendationContent(existing, next) {
  const existingKey = recommendationContentKey(existing);
  return existingKey !== null && existingKey === recommendationContentKey(next);
}

/** Generate a new JSON file only after statistics and candidate validation succeed. */
async function main() {
  const packageMap = JSON.parse(await readFile(mapPath, "utf8"));
  if (!Array.isArray(packageMap) || packageMap.length === 0) {
    throw new Error("recommendation package map must be a non-empty array");
  }

  const period = defaultPeriod();
  const popularities = await fetchAllPopularities(period);
  const recommendations = buildRecommendations(popularities, packageMap);
  if (recommendations.length === 0) {
    throw new Error("no recommendation candidates matched pkgstats; keeping existing data");
  }

  const output = {
    source: "Arch Linux pkgstats",
    sourceUrl,
    generatedAt: new Date().toISOString(),
    period,
    recommendations,
  };

  let existingText;
  try {
    existingText = await readFile(outputPath, "utf8");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }

  if (existingText !== undefined) {
    try {
      const existing = JSON.parse(existingText);
      if (hasSameRecommendationContent(existing, output)) {
        console.log("Recommendation content is unchanged; keeping the existing data file.");
        return;
      }
    } catch {
      // Replace malformed existing JSON with the newly validated payload.
    }
  }

  await mkdir(path.dirname(outputPath), { recursive: true });
  await writeFile(outputPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
  console.log(
    `Updated ${recommendations.length} recommendations from ${popularities.length} pkgstats records (${period.startMonth}-${period.endMonth}).`,
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
