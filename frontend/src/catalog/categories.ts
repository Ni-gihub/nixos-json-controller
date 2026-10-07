import { apps, type AppCategory } from './apps'

export const categories: AppCategory[] = [...new Set(apps.map((app) => app.category))]

export function appsInCategory(category: AppCategory) {
  return apps.filter((app) => app.category === category)
}
