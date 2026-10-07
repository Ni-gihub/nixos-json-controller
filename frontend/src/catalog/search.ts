import { invoke } from '@tauri-apps/api/core'
import type { App } from './apps'

const searchCache = new Map<string, App[]>()

export async function searchApps(query: string): Promise<App[]> {
  const normalized = query.trim()
  if (!normalized) return []

  const cached = searchCache.get(normalized.toLowerCase())
  if (cached) return cached

  const results = await invoke<App[]>('search_catalog', { query: normalized })
  searchCache.set(normalized.toLowerCase(), results)
  return results
}

export async function findApp(id: string): Promise<App | undefined> {
  const app = await invoke<App | null>('get_catalog_app', { id })
  return app ?? undefined
}
