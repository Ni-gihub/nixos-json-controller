import { FormEvent, useEffect, useState } from 'react'
import { Link, useSearchParams } from 'react-router'
import { Search, Store } from 'lucide-react'
import { Input } from '@/components/ui/input'
import InstallButton from '@/components/InstallButton'
import { searchApps, type App } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function Discover() {
  const [searchParams, setSearchParams] = useSearchParams()
  const initialQuery = searchParams.get('query')?.trim() ?? ''
  const [query, setQuery] = useState(initialQuery)
  const [submittedQuery, setSubmittedQuery] = useState(initialQuery)
  const [results, setResults] = useState<App[]>([])
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const { states, loading, error, refreshPackages, refreshPackage } = useAppState()

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
    <div className="min-h-screen bg-[#f7f7f8] text-foreground">
      <div className="flex min-h-screen">
        <aside className="hidden w-[266px] shrink-0 border-r border-slate-200/80 bg-[#f4f4f5] lg:block">
          <div className="sticky top-0 flex h-screen flex-col px-4 py-6">
            <div className="flex items-center gap-3 px-1">
              <div className="flex size-9 items-center justify-center rounded-full bg-blue-600 text-sm font-bold text-white">
                N
              </div>
              <p className="text-[17px] font-semibold tracking-tight">nxc App Store</p>
            </div>

            <div className="mt-5 flex h-9 items-center gap-2 rounded-xl bg-slate-200/80 px-3 text-sm text-slate-600">
              <Search className="size-4 shrink-0 text-slate-500" />
              <span className="truncate">{submittedQuery || '検索'}</span>
            </div>

            <nav aria-label="App Store" className="mt-5">
              <Link
                to="/discover"
                className="flex items-center gap-3 rounded-xl bg-slate-200/70 px-3 py-2.5 text-sm font-semibold text-slate-900"
              >
                <Search className="size-4 text-blue-600" />
                見つける
              </Link>
            </nav>
          </div>
        </aside>

        <div className="min-w-0 flex-1">
          <header className="border-b border-slate-200/80 bg-white/90 backdrop-blur lg:hidden">
            <div className="flex h-16 items-center px-5">
              <div className="flex items-center gap-2 font-semibold">
                <Store className="size-5 text-blue-600" />
                nxc App Store
              </div>
            </div>
          </header>

          <main className="w-full px-6 py-8 sm:px-8 lg:px-9 lg:py-10">
            <section className="relative overflow-hidden rounded-[30px] border border-blue-100 bg-gradient-to-br from-white via-white to-blue-50/80 px-7 py-9 shadow-sm sm:px-10 sm:py-10">
              <div className="pointer-events-none absolute -right-20 -top-24 size-72 rounded-full bg-blue-300/15 blur-3xl" />

              <div className="relative">
                <div className="inline-flex items-center gap-2 rounded-full border border-blue-100 bg-white/90 px-3 py-1.5 text-xs font-semibold tracking-[0.12em] text-blue-600">
                  <span className="size-1.5 rounded-full bg-blue-500" />
                  APP STORE · SEARCH
                </div>

                <h1 className="mt-5 text-4xl font-bold tracking-tight sm:text-5xl">見つける</h1>
                <p className="mt-2 text-base text-slate-500 sm:text-lg">
                  人気のアプリも、新しいお気に入りもここから。
                </p>

                <form onSubmit={handleSearch} className="mt-7">
                  <div className="relative">
                    <Search className="absolute left-5 top-1/2 size-5 -translate-y-1/2 text-blue-500" />
                    <Input
                      className="h-[74px] rounded-[22px] border-slate-200 bg-white pl-16 pr-12 text-base shadow-sm ring-offset-background placeholder:text-slate-400 focus-visible:border-blue-300 focus-visible:ring-blue-100 sm:text-lg"
                      placeholder="アプリ・ゲーム・開発ツールを検索"
                      value={query}
                      onChange={(event) => setQuery(event.target.value)}
                      autoFocus
                    />
                  </div>
                </form>
              </div>
            </section>

            <section className="mt-10 lg:mt-12">
              {!hasResults ? (
                <div className="py-16 text-center">
                  <div className="mx-auto flex size-14 items-center justify-center rounded-full bg-blue-50 text-blue-600">
                    <Search className="size-6" />
                  </div>
                  <h2 className="mt-5 text-xl font-semibold">アプリを検索</h2>
                  <p className="mx-auto mt-2 max-w-lg text-sm leading-6 text-slate-500">
                    アプリ、開発ツール、ライブラリなどを検索できます。
                  </p>
                </div>
              ) : (
                <>
                  <div className="mb-6">
                    <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
                      「{submittedQuery}」の検索結果
                    </h2>
                    {!searching && (
                      <p className="mt-1 text-sm text-slate-500">{results.length}件のApp</p>
                    )}
                  </div>

                  <div className="border-t border-slate-200">
                    <div className="flex items-center justify-between py-5">
                      <h3 className="text-xl font-semibold">App</h3>
                      {(searching || loading) && (
                        <span className="text-sm text-slate-500">検索中...</span>
                      )}
                    </div>

                    {searchError ? (
                      <div className="border-t border-slate-200 py-12 text-center">
                        <p className="font-medium">検索できませんでした。</p>
                        <p className="mt-2 text-sm text-slate-500">{searchError}</p>
                      </div>
                    ) : results.length === 0 && !searching ? (
                      <div className="border-t border-slate-200 py-12 text-center">
                        <p className="font-medium">該当するパッケージが見つかりませんでした。</p>
                        <p className="mt-2 text-sm text-slate-500">
                          別のキーワードで検索してみてください。
                        </p>
                      </div>
                    ) : (
                      <div className="border-t border-slate-200">
                        {results.map((app) => {
                          const installed = states[app.id]?.active ?? false
                          const detailPath = `/apps/${app.id}?query=${encodeURIComponent(submittedQuery)}`

                          return (
                            <div
                              key={app.id}
                              className="grid min-h-[128px] grid-cols-1 items-center gap-x-6 gap-y-4 border-b border-slate-200 py-6 sm:grid-cols-[minmax(0,1fr)_auto]"
                            >
                              <Link
                                to={detailPath}
                                className="flex min-w-0 items-center gap-5 rounded-xl outline-none transition-opacity hover:opacity-75 focus-visible:ring-2 focus-visible:ring-blue-500"
                              >
                                <div className="relative flex size-[72px] shrink-0 items-center justify-center overflow-hidden rounded-[18px] border border-slate-200 bg-gradient-to-br from-blue-50 via-white to-indigo-50 text-2xl font-bold text-blue-600 shadow-sm">
                                  <span aria-hidden="true">{app.name[0]}</span>
                                  {app.icon && (
                                    <img
                                      src={app.icon}
                                      alt=""
                                      className="absolute inset-0 size-full object-contain p-2.5"
                                      loading="lazy"
                                      onError={(event) => {
                                        event.currentTarget.style.display = 'none'
                                      }}
                                    />
                                  )}
                                </div>

                                <div className="min-w-0">
                                  <h4 className="truncate text-base font-semibold text-slate-900 sm:text-lg">
                                    {app.name}
                                  </h4>
                                  <p className="mt-0.5 line-clamp-2 text-sm text-slate-500 sm:text-[15px]">
                                    {app.description}
                                  </p>
                                  <p className="mt-1 text-xs text-slate-400">{app.category}</p>
                                </div>
                              </Link>

                              <div className="w-[116px] shrink-0 sm:w-[132px]">
                                <InstallButton
                                  packageId={app.id}
                                  appName={app.name}
                                  installed={installed}
                                  onInstalled={() => refreshPackage(app.id)}
                                  className="h-10 w-full rounded-full bg-blue-50 px-4 text-sm font-semibold text-blue-600 shadow-none hover:bg-blue-100 hover:text-blue-700"
                                  size="sm"
                                />
                              </div>
                            </div>
                          )
                        })}
                      </div>
                    )}
                  </div>

                  {error && (
                    <p className="mt-5 text-sm text-slate-500">
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
