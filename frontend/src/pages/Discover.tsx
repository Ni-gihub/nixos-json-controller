import { FormEvent, useEffect, useState } from 'react'
import { Search, Store } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'
import { searchApps, type App } from '@/catalog'
import { useAppState } from '@/lib/app-state'
import { useSearchParams } from 'react-router'

function Discover() {
  const [searchParams, setSearchParams] = useSearchParams()
  const initialQuery = searchParams.get('query')?.trim() ?? ''
  const [query, setQuery] = useState(initialQuery)
  const [submittedQuery, setSubmittedQuery] = useState(initialQuery)
  const [results, setResults] = useState<App[]>([])
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const { states, loading, error, refreshPackages } = useAppState()

  useEffect(() => {
    if (!initialQuery || initialQuery === submittedQuery) return
    setQuery(initialQuery)
    setSubmittedQuery(initialQuery)
  }, [initialQuery, submittedQuery])

  useEffect(() => {
    if (!initialQuery || initialQuery !== submittedQuery || results.length > 0 || searching) return

    setSearching(true)
    setSearchError(null)
    void searchApps(initialQuery)
      .then((nextResults) => {
        setResults(nextResults)
        void refreshPackages(nextResults.map((app) => app.id))
      })
      .catch((err) => {
        setResults([])
        setSearchError(String(err))
      })
      .finally(() => setSearching(false))
  }, [initialQuery, refreshPackages, results.length, searching, submittedQuery])

  const handleSearch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const normalized = query.trim()
    if (!normalized) return
    setSubmittedQuery(normalized)
    setSearchParams({ query: normalized })
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
    <div className="min-h-screen bg-muted/20 text-foreground">
      <div className="flex min-h-screen">
        <aside className="hidden w-72 shrink-0 border-r border-border/80 bg-background/95 lg:block">
          <div className="sticky top-0 flex h-screen flex-col p-6">
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
                <Store className="size-5" />
              </div>
              <div>
                <p className="font-semibold tracking-tight">nxc App Store</p>
                <p className="text-xs text-muted-foreground">NixOS向けアプリストア</p>
              </div>
            </div>
            <nav aria-label="App Store" className="mt-10 space-y-1">
              <div aria-current="page" className="rounded-lg bg-muted px-3 py-2.5 text-sm font-medium">
                アプリを探す
              </div>
            </nav>
            <div className="mt-auto rounded-xl bg-muted/60 p-4">
              <p className="text-xs font-medium">Nixpkgs</p>
              <p className="mt-1 text-xs leading-5 text-muted-foreground">
                アプリ、言語、ライブラリ、開発ツールを検索できます。
              </p>
            </div>
          </div>
        </aside>

        <div className="min-w-0 flex-1">
          <header className="border-b bg-background lg:hidden">
            <div className="flex h-16 items-center px-5">
              <div className="flex items-center gap-2 font-semibold">
                <Store className="size-5" />
                nxc App Store
              </div>
            </div>
          </header>

          <main className="w-full px-6 py-8 sm:px-8 lg:px-12 lg:py-12">
            <section>
              <p className="text-base font-semibold tracking-wide text-primary">NixOS App Store</p>
              <h1 className="mt-3 text-5xl font-bold tracking-tight lg:text-6xl">アプリを探す</h1>
              <p className="mt-3 max-w-3xl text-lg text-muted-foreground lg:text-xl">
                Nixpkgsにあるアプリ、言語、ライブラリ、開発ツールを検索できます。
              </p>

              <form onSubmit={handleSearch} className="mt-8">
                <div className="flex w-full gap-3">
                  <div className="relative min-w-0 flex-1">
                    <Search className="absolute left-5 top-1/2 size-5 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      className="h-14 rounded-xl border-border bg-card pl-14 pr-5 text-lg shadow-sm lg:h-16 lg:text-xl"
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
                <p className="mt-2 text-sm text-muted-foreground">Enterキーで検索</p>
              </form>
            </section>

            <section className="mt-12 lg:mt-14">
              {!hasResults ? (
                <div className="flex min-h-[360px] w-full items-center justify-center rounded-2xl border border-dashed border-border bg-card/50 px-8 py-16 text-center">
                  <div>
                    <div className="mx-auto flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary">
                      <Search className="size-6" />
                    </div>
                    <h2 className="mt-5 text-xl font-semibold">検索してアプリを探す</h2>
                    <p className="mx-auto mt-2 max-w-lg text-sm leading-6 text-muted-foreground">
                      検索結果は関連度順に表示されます。アプリだけでなく、開発ツールやライブラリも見つけられます。
                    </p>
                  </div>
                </div>
              ) : (
                <>
                  <div className="mb-6 flex items-end justify-between gap-4">
                    <div>
                      <h2 className="text-3xl font-semibold">「{submittedQuery}」の検索結果</h2>
                      {!searching && <p className="mt-1 text-base text-muted-foreground">{results.length}件</p>}
                    </div>
                    {(searching || loading) && <span className="text-sm text-muted-foreground">検索中...</span>}
                  </div>

                  {searchError ? (
                    <div className="w-full rounded-2xl border border-dashed border-border bg-card/50 p-12 text-center">
                      <p className="font-medium">検索できませんでした。</p>
                      <p className="mt-2 text-sm text-muted-foreground">{searchError}</p>
                    </div>
                  ) : results.length === 0 && !searching ? (
                    <div className="w-full rounded-2xl border border-dashed border-border bg-card/50 p-12 text-center">
                      <p className="font-medium">該当するパッケージが見つかりませんでした。</p>
                      <p className="mt-2 text-sm text-muted-foreground">別のキーワードで検索してみてください。</p>
                    </div>
                  ) : (
                    <div className="grid gap-7 md:grid-cols-2 xl:grid-cols-3">
                      {results.map((app) => (
                        <AppCard
                          key={app.id}
                          id={app.id}
                          name={app.name}
                          description={app.description}
                          category={app.category}
                          tags={app.tags}
                          installed={states[app.id]?.active ?? false}
                          searchQuery={submittedQuery}
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
      </div>
    </div>
  )
}

export default Discover
