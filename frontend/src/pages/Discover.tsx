import { useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'
import { Button } from '@/components/ui/button'
import { categories, searchApps, type AppCategory } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function Discover() {
  const [query, setQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<AppCategory | null>(null)
  const { states, loading, error, refreshAll } = useAppState()

  const hasSearch = query.trim().length > 0 || selectedCategory !== null

  const filteredApps = useMemo(() => {
    let result = searchApps(query)

    if (selectedCategory) {
      result = result.filter((app) => app.category === selectedCategory)
    }

    return result
  }, [query, selectedCategory])

  const clearFilters = () => {
    setQuery('')
    setSelectedCategory(null)
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b border-border/70 bg-background/90 backdrop-blur">
        <div className="mx-auto flex max-w-4xl items-center px-5 py-4 sm:px-6">
          <h1 className="text-lg font-semibold tracking-tight">nxc App Store</h1>
        </div>
      </header>

      <main className="mx-auto max-w-4xl px-5 py-8 sm:px-6 sm:py-12">
        <section className="mb-8 sm:mb-10">
          <p className="mb-2 text-sm font-medium text-primary">NixOS App Store</p>
          <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">アプリを探す</h2>
          <p className="mt-2 text-muted-foreground">
            欲しいアプリを検索して、NixOSにインストールできます。
          </p>

          <div className="relative mt-6">
            <Search className="absolute left-4 top-1/2 size-5 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="h-12 rounded-2xl border-border/80 bg-card pl-12 pr-4 text-base shadow-sm sm:h-14"
              placeholder="アプリを検索..."
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              autoFocus
            />
          </div>
        </section>

        <section className="mb-8">
          <div className="mb-3 flex items-center justify-between">
            <h3 className="text-sm font-semibold">カテゴリ</h3>
            {hasSearch && (
              <Button variant="ghost" size="sm" onClick={clearFilters}>
                クリア
              </Button>
            )}
          </div>

          <div className="flex gap-2 overflow-x-auto pb-2">
            <Button
              className="shrink-0 rounded-full"
              variant={selectedCategory === null ? 'default' : 'outline'}
              size="sm"
              onClick={() => setSelectedCategory(null)}
            >
              すべて
            </Button>
            {categories.map((category) => (
              <Button
                key={category}
                className="shrink-0 rounded-full"
                variant={selectedCategory === category ? 'default' : 'outline'}
                size="sm"
                onClick={() => setSelectedCategory(category)}
              >
                {category}
              </Button>
            ))}
          </div>
        </section>

        {!hasSearch ? (
          <section className="rounded-3xl border border-dashed border-border/80 bg-card/60 px-6 py-14 text-center sm:py-20">
            <div className="mx-auto flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary">
              <Search className="size-6" />
            </div>
            <h3 className="mt-4 text-lg font-semibold">アプリを検索してください</h3>
            <p className="mt-1 text-sm text-muted-foreground">
              アプリ名、説明、カテゴリ、タグから探せます。
            </p>
          </section>
        ) : (
          <section>
            <div className="mb-4 flex items-center justify-between gap-4">
              <h3 className="text-lg font-semibold">
                検索結果
                <span className="ml-2 text-sm font-normal text-muted-foreground">
                  {filteredApps.length}件
                </span>
              </h3>
              {loading && (
                <span className="text-sm text-muted-foreground">状態を確認中...</span>
              )}
              {error && (
                <Button variant="outline" size="sm" onClick={() => void refreshAll()}>
                  再確認
                </Button>
              )}
            </div>

            {filteredApps.length === 0 ? (
              <div className="rounded-3xl border border-dashed border-border/80 bg-card/60 p-10 text-center">
                <p className="font-medium">アプリが見つかりませんでした。</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  別のキーワードやカテゴリを試してください。
                </p>
                <Button variant="outline" className="mt-5 rounded-full" onClick={clearFilters}>
                  検索をクリア
                </Button>
              </div>
            ) : (
              <div className="grid gap-3 sm:grid-cols-2">
                {filteredApps.map((app) => (
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
        )}
      </main>
    </div>
  )
}

export default Discover
