import { invoke } from "@tauri-apps/api/core";

export type CatalogPreparationStatus = {
  action: "reused-cache" | "reused-memory" | "generated" | string;
  sourceRevision?: string;
  sourceNarHash: string;
  system: string;
  packageCount: number;
  generatedAt: number;
  cachePath: string;
  cacheBytes: number;
  sourceCheckMs: number;
  cacheLoadMs: number;
  generationMs: number;
};

/** Verify the active Nixpkgs source and prepare a matching local search catalog. */
export function prepareCatalog(
  forceRefresh = false,
): Promise<CatalogPreparationStatus> {
  return invoke<CatalogPreparationStatus>("prepare_catalog", { forceRefresh });
}
