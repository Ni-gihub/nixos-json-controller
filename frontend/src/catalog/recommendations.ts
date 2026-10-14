import { invoke } from "@tauri-apps/api/core";
import type { App } from "./apps";

export async function getRecommendedCatalogApps(ids: string[]): Promise<App[]> {
  if (ids.length === 0) return [];
  return invoke<App[]>("get_catalog_apps", { ids });
}
