import { invoke } from '@tauri-apps/api/core'
import type { App } from './apps'

export async function searchApps(query: string): Promise<App[]> {
  const normalized = query.trim()
  if (!normalized) return []

  return invoke<App[]>('search_catalog', { query: normalized })
}

export async function findApp(id: string): Promise<App | undefined> {
  const app = await invoke<App | null>('get_catalog_app', { id })
  return app ?? undefined
}
