import { useEffect, useState } from 'react'
import { ArrowLeft, ExternalLink, Package, Tag } from 'lucide-react'
import { Link, useParams, useSearchParams } from 'react-router'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import InstallButton from '@/components/InstallButton'
import { findApp, type App } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function AppDetail() {
  const { id } = useParams()
  const [searchParams] = useSearchParams()
  const returnQuery = searchParams.get('query')?.trim() ?? ''
  const { states, refreshPackage } = useAppState()
  const [app, setApp] = useState<App | undefined>()
  const [loadingApp, setLoadingApp] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [refreshError, setRefreshError] = useState<string | null>(null)

  const appState = id ? states[id] : undefined
  const installed = appState?.active === true
  const statusKnown = appState !== undefined

  useEffect(() => {
    if (!id) return
    let cancelled = false
    setLoadingApp(true)
    setError(null)
    void findApp(id)
      .then((result) => { if (!cancelled) setApp(result) })
      .catch((err) => { if (!cancelled) setError(String(err)) })
      .finally(() => { if (!cancelled) setLoadingApp(false) })
    return () => { cancelled = true }
  }, [id])

  useEffect(() => {
    if (!id) return
    void refreshPackage(id).catch((err) => setRefreshError(String(err)))
  }, [id, refreshPackage])

  if (loadingApp) {
    return <div className="flex min-h-screen items-center justify-center bg-muted/20 text-muted-foreground">パッケージ情報を取得中...</div>
  }

  if (!app || !id) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/20">
        <div className="text-center">
          <h1 className="text-2xl font-bold">パッケージが見つかりません</h1>
          {error && <p className="mt-2 max-w-md text-sm text-muted-foreground">{error}</p>}
          <Link to={returnQuery ? `/discover?query=${encodeURIComponent(returnQuery)}` : '/discover'} className="mt-4 inline-block text-primary hover:underline">パッケージを探す</Link>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="sticky top-0 z-30 border-b border-blue-100/80 bg-white/90 backdrop-blur">
        <div className="flex h-16 items-center px-6 sm:px-8">
          <Link
            to={returnQuery ? `/discover?query=${encodeURIComponent(returnQuery)}` : "/discover"}
            className="inline-flex items-center gap-2 rounded-lg px-2 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-blue-50 hover:text-blue-700"
          >
            <ArrowLeft className="size-4" />
            アプリを探す
          </Link>
          <div className="ml-4 border-l border-blue-100 pl-4 text-sm font-semibold">nxc App Store</div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-none px-6 py-8 sm:px-8 lg:px-12 lg:py-12">
        <div className="mx-auto max-w-6xl">
          <section className="min-w-0 space-y-7">
            <Card className="overflow-hidden rounded-3xl border-blue-100 bg-gradient-to-br from-white via-blue-50/50 to-indigo-100/70">
              <CardContent className="relative p-7 sm:p-9 lg:p-10">
                <div className="pointer-events-none absolute -right-20 -top-24 size-64 rounded-full bg-blue-400/15 blur-3xl" />
                <div className="relative flex flex-col items-start gap-7 sm:flex-row">
                  <div className="flex size-32 shrink-0 items-center justify-center rounded-3xl border border-blue-100 bg-white/80 text-6xl font-bold text-blue-600">
                    {app.icon ? (
                      <img src={app.icon} alt="" className="size-24 object-contain" />
                    ) : (
                      app.name[0]
                    )}
                  </div>
                  <div className="min-w-0 flex-1 pt-1">
                    <div className="flex flex-wrap items-start justify-between gap-5">
                      <div className="min-w-0">
                        <div className="flex flex-wrap gap-2">
                          <Badge className="rounded-full bg-blue-600 px-3 py-1 text-white hover:bg-blue-600">{app.category}</Badge>
                          {app.tags.map((tag) => <Badge key={tag} variant="outline" className="rounded-full border-blue-200 bg-white/60">{tag}</Badge>)}
                        </div>
                      </div>
                      <InstallButton
                        packageId={id}
                        appName={app.name}
                        installed={installed}
                        statusKnown={statusKnown}
                        onInstalled={() => refreshPackage(id)}
                        className="h-10 shrink-0 rounded-full bg-blue-600 px-4 text-sm text-white hover:bg-blue-700"
                        size="sm"
                      />
                    </div>
                    <h1 className="mt-4 text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl">{app.name}</h1>
                    {app.version && <p className="mt-2 text-sm text-muted-foreground">バージョン {app.version}</p>}
                    <p className="mt-5 max-w-3xl text-base leading-7 text-muted-foreground sm:text-lg lg:text-xl">{app.description}</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="rounded-3xl border-blue-100 bg-white/90">
              <CardContent className="p-7 sm:p-9 lg:p-10">
                <h2 className="text-xl font-semibold">説明・詳細情報</h2>
                <div className="mt-7 grid gap-7 md:grid-cols-2">
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium"><Package className="size-4 text-blue-600" />パッケージ</div>
                    <p className="break-all rounded-xl border border-blue-100 bg-blue-50/50 px-4 py-3 font-mono text-sm">{app.id}</p>
                  </div>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium"><Tag className="size-4 text-blue-600" />カテゴリ</div>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="secondary">{app.category}</Badge>
                      {app.tags.map((tag) => <Badge key={tag} variant="outline">{tag}</Badge>)}
                    </div>
                  </div>
                </div>
                {app.homepage && (
                  <div className="mt-8 border-t border-blue-100 pt-7">
                    <p className="text-sm font-medium">Homepage</p>
                    <a href={app.homepage} target="_blank" rel="noreferrer" className="mt-2 inline-flex max-w-full items-center gap-2 break-all text-sm text-blue-600 hover:underline">
                      {app.homepage}<ExternalLink className="size-4 shrink-0" />
                    </a>
                  </div>
                )}
              </CardContent>
            </Card>
          </section>

        </div>
      </main>
    </div>
  )
}

export default AppDetail
