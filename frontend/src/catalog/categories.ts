import { apps, type AppCategory } from './apps'

export const categories: AppCategory[] = [
  'Browser',
  'Development',
  'Media',
  'Graphics',
]

export function appsInCategory(category: AppCategory) {
  return apps.filter((app) => app.category === category)
}
