import { invoke } from '@tauri-apps/api/core'
import type { App } from './apps'

const searchCache = new Map<string, App[]>()

export async function searchApps(query: string): Promise<App[]> {
  const normalized = query.trim()
  if (!normalized) return []

  const cacheKey = normalized.toLowerCase()
  const cached = searchCache.get(cacheKey)
  if (cached) return cached

  const results = await invoke<App[]>('search_catalog', { query: normalized })
  searchCache.set(cacheKey, results)
  return results
}

export async function findApp(id: string, query?: string): Promise<App | undefined> {
  // Search results already contain the detail metadata and resolved icon candidates.
  // Reuse that exact result when navigating from the discover page instead of searching Nixpkgs again.
  const cacheKey = query?.trim().toLowerCase()
  const cachedResults = cacheKey ? searchCache.get(cacheKey) : undefined
  const cachedApp = cachedResults?.find((app) => app.id === id)
  if (cachedApp) return cachedApp

  // Keep direct detail links and hard refreshes working when no search result is cached.
  const app = await invoke<App | null>('get_catalog_app', { id })
  return app ?? undefined
}
