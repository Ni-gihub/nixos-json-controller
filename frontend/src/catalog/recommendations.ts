import { invoke } from "@tauri-apps/api/core";
import type { App } from "./apps";

const IDS_PER_REQUEST = 40;

/**
 * Resolve recommendations in bounded batches. Rust intentionally limits each request to 50
 * IDs; batching here lets the catalog grow without dropping every ID after the first batch.
 */
export async function getRecommendedCatalogApps(ids: string[]): Promise<App[]> {
  if (ids.length === 0) return [];

  const resolved = new Map<string, App>();
  let firstError: unknown;

  for (let offset = 0; offset < ids.length; offset += IDS_PER_REQUEST) {
    const batch = ids.slice(offset, offset + IDS_PER_REQUEST);
    try {
      const apps = await invoke<App[]>("get_catalog_apps", { ids: batch });
      for (const app of apps) {
        if (!resolved.has(app.id)) resolved.set(app.id, app);
      }
    } catch (error) {
      firstError ??= error;
    }
  }

  if (resolved.size === 0 && firstError !== undefined) throw firstError;
  return [...resolved.values()];
}
