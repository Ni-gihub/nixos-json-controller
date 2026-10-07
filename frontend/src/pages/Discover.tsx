import { FormEvent, useState } from 'react'
import { Search } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'
import { searchApps, type App } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function Discover() {
  const [query, setQuery] = useState('')
  const [submittedQuery, setSubmittedQuery] = useState('')
  const [results, setResults] = useState<App[]>([])
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const { states, loading, error, refreshPackages } = useAppState()

  const handleSearch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    const normalized = query.trim()
    if (!normalized) return

    setSubmittedQuery(normalized)
    setSearching(true)
    setSearchError(null)

    try {
      const nextResults = await searchApps(normalized)
      setResults(nextResults)
      void refreshPackages(nextResults.map((app) => app.id))
    } catch (err) {
      setResults([])
      setSearchError(String(err))
    } finally {
      setSearching(false)
    }
  }

  const hasResults = submittedQuery.length > 0

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b border-border/70 bg-background">
        <div className="mx-auto flex h-16 max-w-7xl items-center px-8">
          <h1 className="text-lg font-semibold tracking-tight">nxc App Store</h1>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-8 py-12 lg:py-16">
        <section className="mx-auto max-w-4xl">
          <p className="text-sm font-semibold tracking-wide text-primary">NixOS App Store</p>
          <div className="mt-3 flex items-end justify-between gap-8">
            <div>
              <h2 className="text-4xl font-bold tracking-tight lg:text-5xl">探す</h2>
              <p className="mt-3 text-base text-muted-foreground lg:text-lg">
                Nixpkgsにあるアプリ、言語、ライブラリ、開発ツールを検索できます。
              </p>
            </div>
          </div>

          <form onSubmit={handleSearch} className="mt-8">
            <div className="flex gap-3">
              <div className="relative min-w-0 flex-1">
                <Search className="absolute left-5 top-1/2 size-5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  className="h-14 rounded-xl border-border bg-card pl-14 pr-5 text-base shadow-sm lg:h-16 lg:text-lg"
                  placeholder="アプリ、言語、ライブラリ、ツールを検索..."
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  autoFocus
                />
              </div>
              <Button
                type="submit"
                className="h-14 rounded-xl px-6 lg:h-16 lg:px-8"
                disabled={!query.trim() || searching}
              >
                <Search className="mr-2 size-4" />
                検索
              </Button>
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              Enterキーで検索
            </p>
          </form>
        </section>

        <section className="mx-auto mt-12 max-w-6xl lg:mt-16">
          {!hasResults ? (
            <div className="rounded-2xl border border-dashed border-border bg-card/50 px-8 py-16 text-center">
              <div className="mx-auto flex size-12 items-center justify-center rounded-xl bg-primary/10 text-primary">
                <Search className="size-5" />
              </div>
              <h3 className="mt-4 text-lg font-semibold">検索してアプリを探す</h3>
              <p className="mx-auto mt-2 max-w-lg text-sm text-muted-foreground">
                検索結果は関連度順に表示されます。アプリだけでなく、開発ツールやライブラリも見つけられます。
              </p>
            </div>
          ) : (
            <>
              <div className="mb-6 flex items-center justify-between gap-4">
                <div>
                  <h3 className="text-xl font-semibold">「{submittedQuery}」の検索結果</h3>
                  {!searching && (
                    <p className="mt-1 text-sm text-muted-foreground">{results.length}件</p>
                  )}
                </div>
                {(searching || loading) && (
                  <span className="text-sm text-muted-foreground">検索中...</span>
                )}
              </div>

              {searchError ? (
                <div className="rounded-2xl border border-dashed border-border bg-card/50 p-12 text-center">
                  <p className="font-medium">検索できませんでした。</p>
                  <p className="mt-2 text-sm text-muted-foreground">{searchError}</p>
                </div>
              ) : results.length === 0 && !searching ? (
                <div className="rounded-2xl border border-dashed border-border bg-card/50 p-12 text-center">
                  <p className="font-medium">該当するパッケージが見つかりませんでした。</p>
                  <p className="mt-2 text-sm text-muted-foreground">
                    別のキーワードで検索してみてください。
                  </p>
                </div>
              ) : (
                <div className="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
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
                <p className="mt-5 text-sm text-muted-foreground">
                  一部のインストール状態を取得できませんでした。
                </p>
              )}
            </>
          )}
        </section>
      </main>
    </div>
  )
}

export default Discover
