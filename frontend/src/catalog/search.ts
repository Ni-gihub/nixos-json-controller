import { apps, type App } from './apps'

export function findApp(id: string): App | undefined {
  return apps.find((app) => app.id === id)
}

export function searchApps(query: string): App[] {
  const normalized = query.trim().toLowerCase()
  if (!normalized) return apps

  return apps.filter((app) =>
    [app.name, app.description, app.category].some((value) =>
      value.toLowerCase().includes(normalized),
    ),
  )
}
