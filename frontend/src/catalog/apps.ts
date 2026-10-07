export type AppCategory = string
export type AppTag = string

export type App = {
  id: string
  name: string
  description: string
  version: string
  category: AppCategory
  tags: AppTag[]
  homepage?: string
}
