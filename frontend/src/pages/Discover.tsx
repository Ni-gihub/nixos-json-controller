import { useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'
import { Button } from '@/components/ui/button'
import { apps, categories, getFeaturedApps, getTags, searchApps, type AppCategory } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function Discover() {
  const [query, setQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<AppCategory | null>(null)
  const [selectedTag, setSelectedTag] = useState<string | null>(null)
  const { states, loading, error, refreshAll } = useAppState()

  const filteredApps = useMemo(() => {
    let result = searchApps(query)

    if (selectedCategory) {
      result = result.filter((app) => app.category === selectedCategory)
    }

    if (selectedTag) {
      result = result.filter((app) =>
        app.tags.some((tag) => tag.toLowerCase() === selectedTag.toLowerCase()),
      )
    }

    return result
  }, [query, selectedCategory, selectedTag])

  const showFeatured = !query.trim() && !selectedCategory && !selectedTag
  const featuredApps = getFeaturedApps()
  const otherApps = filteredApps.filter((app) => !app.featured)

  const clearFilters = () => {
    setSelectedCategory(null)
    setSelectedTag(null)
    setQuery('')
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b">
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-6 px-6 py-4">
          <h1 className="text-xl font-semibold">nxc App Store</h1>
          <div className="relative w-full max-w-sm">
            <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="pl-9"
              placeholder="アプリを検索..."
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-6 py-10">
        <section className="mb-10">
          <h2 className="text-4xl font-bold tracking-tight">アプリを探す</h2>
          <p className="mt-2 text-muted-foreground">
            NixOSにインストールできるアプリを探せます。
          </p>
        </section>

        <section className="mb-8">
          <h3 className="mb-4 text-lg font-semibold">カテゴリ</h3>
          <div className="flex flex-wrap gap-2">
            <Button
              variant={selectedCategory === null ? 'default' : 'outline'}
              size="sm"
              onClick={() => setSelectedCategory(null)}
            >
              すべて
            </Button>
            {categories.map((category) => (
              <Button
                key={category}
                variant={selectedCategory === category ? 'default' : 'outline'}
                size="sm"
                onClick={() => {
                  setSelectedCategory(category)
                  setSelectedTag(null)
                }}
              >
                {category}
              </Button>
            ))}
          </div>
        </section>

        <section className="mb-10">
          <h3 className="mb-4 text-lg font-semibold">タグ</h3>
          <div className="flex flex-wrap gap-2">
            {getTags().map((tag) => (
              <Badge
                key={tag}
                variant={selectedTag === tag ? 'default' : 'secondary'}
                className="cursor-pointer px-3 py-1.5"
                onClick={() => {
                  setSelectedTag(selectedTag === tag ? null : tag)
                  setSelectedCategory(null)
                }}
              >
                {tag}
              </Badge>
            ))}
          </div>
        </section>

        {showFeatured && (
          <section className="mb-10">
            <h3 className="mb-4 text-lg font-semibold">おすすめ</h3>
            <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
              {featuredApps.map((app) => (
                <AppCard
                  key={app.id}
                  id={app.id}
                  name={app.name}
                  description={app.description}
                  category={app.category}
                  tags={app.tags}
                  installed={states[app.id]?.active ?? false}
                />
              ))}
            </div>
          </section>
        )}

        <section>
          <div className="mb-4 flex items-center justify-between">
            <h3 className="text-lg font-semibold">
              {showFeatured ? 'その他のアプリ' : '検索結果'}
            </h3>
            {loading && <span className="text-sm text-muted-foreground">システムの状態を確認中...</span>}
            {error && (
              <Button variant="outline" size="sm" onClick={() => void refreshAll()}>
                状態を再確認
              </Button>
            )}
          </div>

          {filteredApps.length === 0 ? (
            <div className="rounded-xl border border-dashed p-10 text-center">
              <p className="text-muted-foreground">条件に一致するアプリがありません。</p>
              <Button variant="outline" className="mt-4" onClick={clearFilters}>
                フィルターを解除
              </Button>
            </div>
          ) : (
            <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
              {(showFeatured ? otherApps : filteredApps).map((app) => (
                <AppCard
                  key={app.id}
                  id={app.id}
                  name={app.name}
                  description={app.description}
                  category={app.category}
                  tags={app.tags}
                  installed={states[app.id]?.active ?? false}
                />
              ))}
            </div>
          )}
        </section>

        {showFeatured && otherApps.length === 0 && apps.length === featuredApps.length && (
          <p className="mt-6 text-center text-sm text-muted-foreground">
            カタログに新しいアプリを追加すると、ここに表示されます。
          </p>
        )}
      </main>
    </div>
  )
}

export default Discover
