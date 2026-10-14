#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const frontendDirectory = path.resolve(scriptDirectory, "..");
const mapPath = path.join(scriptDirectory, "recommendation-package-map.json");
const outputPath = path.join(frontendDirectory, "public", "data", "recommendations.json");
const apiUrl = "https://pkgstats.archlinux.de/api/packages";
const sourceUrl = "https://pkgstats.archlinux.de/api/doc";
const pageSize = 10_000;
const maxOffset = 100_000;
const recommendationLimit = 12;

function formatMonth(date) {
  return date.getUTCFullYear() * 100 + date.getUTCMonth() + 1;
}

function defaultPeriod(now = new Date()) {
  // The API's defaults are the most recent 12 complete months. Pin the range
  // across paginated requests so the result is consistent during the run.
  const end = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
  end.setUTCMonth(end.getUTCMonth() - 1);
  const start = new Date(Date.UTC(end.getUTCFullYear(), end.getUTCMonth(), 1));
  start.setUTCMonth(start.getUTCMonth() - 11);
  return { startMonth: formatMonth(start), endMonth: formatMonth(end) };
}

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
    !Number.isInteger(payload.count) ||
    !Number.isInteger(payload.startMonth) ||
    !Number.isInteger(payload.endMonth)
  ) {
    throw new Error(`pkgstats API response has an unexpected shape at offset ${offset}`);
  }

  if (payload.startMonth !== period.startMonth || payload.endMonth !== period.endMonth) {
    throw new Error("pkgstats API returned data for an unexpected period");
  }

  for (const item of payload.packagePopularities) {
    if (
      typeof item.name !== "string" ||
      typeof item.popularity !== "number" ||
      !Number.isFinite(item.popularity) ||
      item.popularity < 0 ||
      item.popularity > 100 ||
      !Number.isInteger(item.count) ||
      !Number.isInteger(item.samples)
    ) {
      throw new Error("pkgstats API returned an invalid package popularity record");
    }
  }

  return payload;
}

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

function buildRecommendations(popularities, packageMap) {
  const popularityByName = new Map(
    popularities.map((record) => [record.name, record]),
  );
  const uniqueMappings = new Map();
  for (const mapping of packageMap) {
    if (
      !mapping ||
      typeof mapping.archPackage !== "string" ||
      typeof mapping.id !== "string" ||
      !/^[A-Za-z_][A-Za-z0-9_'-]*(\.[A-Za-z_][A-Za-z0-9_'-]*)*$/.test(mapping.id)
    ) {
      throw new Error("recommendation package map contains an invalid entry");
    }
    if (!uniqueMappings.has(mapping.archPackage)) {
      uniqueMappings.set(mapping.archPackage, mapping.id);
    }
  }

  const recommendations = [];
  for (const [archPackage, id] of uniqueMappings) {
    const statistic = popularityByName.get(archPackage);
    if (!statistic) continue;
    recommendations.push({
      id,
      archPackage,
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

  return recommendations.slice(0, recommendationLimit);
}

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

  await writeFile(outputPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
  console.log(
    `Updated ${recommendations.length} recommendations from ${popularities.length} pkgstats records (${period.startMonth}-${period.endMonth}).`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
