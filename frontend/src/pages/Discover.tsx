import { useEffect, useState } from 'react'
import { Search } from 'lucide-react'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'
import { searchApps, type App } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function Discover() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<App[]>([])
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const { states, loading, error, refreshPackages } = useAppState()

  useEffect(() => {
    const normalized = query.trim()
    if (normalized.length < 2) {
      setResults([])
      setSearching(false)
      setSearchError(null)
      return
    }

    let cancelled = false
    const timer = window.setTimeout(async () => {
      setSearching(true)
      setSearchError(null)

      try {
        const nextResults = await searchApps(normalized)
        if (cancelled) return

        setResults(nextResults)
        void refreshPackages(nextResults.map((app) => app.id))
      } catch (err) {
        if (!cancelled) {
          setResults([])
          setSearchError(String(err))
        }
      } finally {
        if (!cancelled) setSearching(false)
      }
    }, 350)

    return () => {
      cancelled = true
      window.clearTimeout(timer)
    }
  }, [query, refreshPackages])

  const hasQuery = query.trim().length > 0

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
          <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">探す</h2>
          <p className="mt-2 text-muted-foreground">
            Nixpkgsにあるパッケージを検索して、NixOSにインストールできます。
          </p>

          <div className="relative mt-6">
            <Search className="absolute left-4 top-1/2 size-5 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="h-12 rounded-2xl border-border/80 bg-card pl-12 pr-4 text-base shadow-sm sm:h-14"
              placeholder="アプリ、言語、ライブラリ、ツールを検索..."
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              autoFocus
            />
          </div>
        </section>

        {!hasQuery ? (
          <section className="rounded-3xl border border-dashed border-border/80 bg-card/60 px-6 py-14 text-center sm:py-20">
            <div className="mx-auto flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary">
              <Search className="size-6" />
            </div>
            <h3 className="mt-4 text-lg font-semibold">検索してください</h3>
            <p className="mt-1 text-sm text-muted-foreground">
              Nixpkgsのパッケージ名や説明から複数の候補を関連度順に探します。
            </p>
          </section>
        ) : (
          <section>
            <div className="mb-4 flex items-center justify-between gap-4">
              <h3 className="text-lg font-semibold">
                検索結果
                {!searching && (
                  <span className="ml-2 text-sm font-normal text-muted-foreground">
                    {results.length}件
                  </span>
                )}
              </h3>
              {(searching || loading) && (
                <span className="text-sm text-muted-foreground">検索中...</span>
              )}
            </div>

            {searchError ? (
              <div className="rounded-3xl border border-dashed border-border/80 bg-card/60 p-10 text-center">
                <p className="font-medium">検索できませんでした。</p>
                <p className="mt-1 text-sm text-muted-foreground">{searchError}</p>
              </div>
            ) : results.length === 0 && !searching ? (
              <div className="rounded-3xl border border-dashed border-border/80 bg-card/60 p-10 text-center">
                <p className="font-medium">
                  {query.trim().length < 2
                    ? '2文字以上入力してください。'
                    : '該当するパッケージが見つかりませんでした。'}
                </p>
                <p className="mt-1 text-sm text-muted-foreground">
                  アプリだけでなく、言語、ライブラリ、開発ツールなども検索できます。
                </p>
              </div>
            ) : (
              <div className="grid gap-3 sm:grid-cols-2">
                {results.map((app) => (
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

            {error && (
              <p className="mt-4 text-sm text-muted-foreground">
                インストール状態を取得できない項目があります。
              </p>
            )}
          </section>
        )}
      </main>
    </div>
  )
}

export default Discover
