import { FormEvent, useEffect, useState } from 'react'
import { Search, Store, Sparkles } from 'lucide-react'
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
    <div className="min-h-screen bg-background text-foreground">
      <div className="flex min-h-screen">
        <aside className="hidden w-80 shrink-0 border-r border-blue-100/80 bg-white/90 lg:block">
          <div className="sticky top-0 flex h-screen flex-col p-7">
            <div className="flex items-center gap-3">
              <div className="flex size-11 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 text-white">
                <Store className="size-5" />
              </div>
              <div>
                <p className="font-semibold tracking-tight">nxc App Store</p>
                <p className="text-xs text-muted-foreground">NixOS向けアプリストア</p>
              </div>
            </div>

            <nav aria-label="App Store" className="mt-12 space-y-2">
              <div aria-current="page" className="flex items-center gap-3 rounded-xl bg-blue-50 px-4 py-3 text-sm font-semibold text-blue-700">
                <Search className="size-4" />
                アプリを探す
              </div>
            </nav>

            <div className="mt-auto rounded-2xl border border-blue-100 bg-gradient-to-br from-blue-50 to-indigo-50 p-5">
              <div className="flex items-center gap-2 text-sm font-semibold text-blue-900">
                <Sparkles className="size-4 text-blue-600" />
                Nixpkgs
              </div>
              <p className="mt-2 text-xs leading-5 text-blue-900/65">
                アプリ、言語、ライブラリ、開発ツールをまとめて検索できます。
              </p>
            </div>
          </div>
        </aside>

        <div className="min-w-0 flex-1">
          <header className="border-b border-blue-100/70 bg-white/90 backdrop-blur lg:hidden">
            <div className="flex h-16 items-center px-5">
              <div className="flex items-center gap-2 font-semibold">
                <Store className="size-5 text-blue-600" />
                nxc App Store
              </div>
            </div>
          </header>

          <main className="mx-auto w-full max-w-7xl px-6 py-8 sm:px-8 lg:px-10 lg:py-12">
            <section className="relative overflow-hidden rounded-3xl border border-blue-100 bg-gradient-to-br from-white via-blue-50/70 to-indigo-100/70 px-7 py-9 sm:px-10 sm:py-11">
              <div className="pointer-events-none absolute -right-20 -top-24 size-72 rounded-full bg-blue-400/15 blur-3xl" />
              <div className="relative">
                <div className="inline-flex items-center gap-2 rounded-full border border-blue-200 bg-white/75 px-3 py-1.5 text-sm font-semibold text-blue-700">
                  <Store className="size-4" />
                  NixOS App Store
                </div>
                <h1 className="mt-5 text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl">アプリを探す</h1>
                <p className="mt-3 max-w-3xl text-base leading-7 text-muted-foreground sm:text-lg">
                  Nixpkgsにあるアプリ、言語、ライブラリ、開発ツールを検索できます。
                </p>

                <form onSubmit={handleSearch} className="mt-8">
                  <div className="flex w-full gap-3">
                    <div className="relative min-w-0 flex-1">
                      <Search className="absolute left-5 top-1/2 size-5 -translate-y-1/2 text-blue-500" />
                      <Input
                        className="h-14 rounded-2xl border-blue-200 bg-white/95 pl-14 pr-5 text-base ring-offset-background placeholder:text-muted-foreground/70 focus-visible:border-blue-400 focus-visible:ring-blue-200 sm:text-lg lg:h-16"
                        placeholder="アプリ、言語、ライブラリ、ツールを検索..."
                        value={query}
                        onChange={(event) => setQuery(event.target.value)}
                        autoFocus
                      />
                    </div>
                    <Button
                      type="submit"
                      className="h-14 rounded-2xl bg-blue-600 px-6 text-white hover:bg-blue-700 sm:px-8 lg:h-16"
                      disabled={!query.trim() || searching}
                    >
                      <Search className="mr-2 size-4" />
                      検索
                    </Button>
                  </div>
                  <p className="mt-2 px-1 text-sm text-muted-foreground">Enterキーで検索</p>
                </form>
              </div>
            </section>

            <section className="mt-10 lg:mt-12">
              {!hasResults ? (
                <div className="flex min-h-[330px] w-full items-center justify-center rounded-3xl border border-dashed border-blue-200 bg-white/65 px-8 py-16 text-center">
                  <div>
                    <div className="mx-auto flex size-14 items-center justify-center rounded-2xl bg-blue-50 text-blue-600">
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
                      <p className="text-sm font-semibold text-blue-600">検索結果</p>
                      <h2 className="mt-1 text-2xl font-semibold sm:text-3xl">「{submittedQuery}」</h2>
                      {!searching && <p className="mt-1 text-sm text-muted-foreground">{results.length}件</p>}
                    </div>
                    {(searching || loading) && <span className="text-sm text-muted-foreground">検索中...</span>}
                  </div>

                  {searchError ? (
                    <div className="w-full rounded-2xl border border-dashed border-border bg-card/70 p-12 text-center">
                      <p className="font-medium">検索できませんでした。</p>
                      <p className="mt-2 text-sm text-muted-foreground">{searchError}</p>
                    </div>
                  ) : results.length === 0 && !searching ? (
                    <div className="w-full rounded-2xl border border-dashed border-border bg-card/70 p-12 text-center">
                      <p className="font-medium">該当するパッケージが見つかりませんでした。</p>
                      <p className="mt-2 text-sm text-muted-foreground">別のキーワードで検索してみてください。</p>
                    </div>
                  ) : (
                    <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
                      {results.map((app) => (
                        <AppCard
                          key={app.id}
                          id={app.id}
                          name={app.name}
                          description={app.description}
                          category={app.category}
                          tags={app.tags}
                          installed={states[app.id]?.active ?? false}
                          icon={app.icon}
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
