import { apps, type App } from './apps'

export function findApp(id: string): App | undefined {
  return apps.find((app) => app.id === id)
}

export function searchApps(query: string): App[] {
  const normalized = query.trim().toLowerCase()
  if (!normalized) return apps

  return apps.filter((app) =>
    [app.name, app.description, app.category, ...app.tags].some((value) =>
      value.toLowerCase().includes(normalized),
    ),
  )
}

export function appsWithTag(tag: string): App[] {
  const normalized = tag.trim().toLowerCase()
  if (!normalized) return []

  return apps.filter((app) =>
    app.tags.some((appTag) => appTag.toLowerCase() === normalized),
  )
}

export function getTags(): string[] {
  return [...new Set(apps.flatMap((app) => app.tags))]
}

export function getFeaturedApps(): App[] {
  return apps.filter((app) => app.featured === true)
}
