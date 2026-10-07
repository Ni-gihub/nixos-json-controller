import { useEffect, useState, type FormEvent } from 'react'
import { ArrowLeft, ExternalLink, Package, Tag } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { Link, useParams, useSearchParams } from 'react-router'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { findApp, type App } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function AppDetail() {
  const { id } = useParams()
  const [searchParams] = useSearchParams()
  const returnQuery = searchParams.get('query')?.trim() ?? ''
  const { states, refreshPackage } = useAppState()
  const [app, setApp] = useState<App | undefined>()
  const [loadingApp, setLoadingApp] = useState(true)
  const [installing, setInstalling] = useState(false)
  const [passwordRequired, setPasswordRequired] = useState(false)
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [refreshError, setRefreshError] = useState<string | null>(null)
  const [installSucceeded, setInstallSucceeded] = useState(false)

  const appState = id ? states[id] : undefined
  const installed = installSucceeded || appState?.active === true
  const statusKnown = appState !== undefined

  useEffect(() => {
    if (!id) return

    let cancelled = false
    setLoadingApp(true)
    setError(null)

    void findApp(id)
      .then((result) => {
        if (!cancelled) setApp(result)
      })
      .catch((err) => {
        if (!cancelled) setError(String(err))
      })
      .finally(() => {
        if (!cancelled) setLoadingApp(false)
      })

    return () => {
      cancelled = true
    }
  }, [id])

  useEffect(() => {
    if (!id) return
    void refreshPackage(id).catch((err) => setRefreshError(String(err)))
  }, [id, refreshPackage])

  if (loadingApp) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/20 text-muted-foreground">
        パッケージ情報を取得中...
      </div>
    )
  }

  if (!app || !id) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/20">
        <div className="text-center">
          <h1 className="text-2xl font-bold">パッケージが見つかりません</h1>
          {error && <p className="mt-2 max-w-md text-sm text-muted-foreground">{error}</p>}
          <Link to={returnQuery ? `/discover?query=${encodeURIComponent(returnQuery)}` : '/discover'} className="mt-4 inline-block text-primary hover:underline">
            パッケージを探す
          </Link>
        </div>
      </div>
    )
  }

  const install = async (authPassword: string | null) => {
    setInstalling(true)
    setError(null)
    try {
      await invoke('install_app', { package: id, password: authPassword })
      setInstallSucceeded(true)
      setPassword('')
      setPasswordRequired(false)
      try {
        await refreshPackage(id)
        setRefreshError(null)
      } catch (err) {
        setRefreshError(String(err))
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setInstalling(false)
    }
  }

  const handleInstall = async () => {
    if (installing || installed || !statusKnown) return
    setError(null)
    try {
      const available = await invoke<boolean>('sudo_available')
      if (available) {
        await install(null)
        return
      }
      setPasswordRequired(true)
    } catch (err) {
      setError(String(err))
    }
  }

  const handlePasswordSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!password) return
    setPasswordRequired(false)
    await install(password)
  }

  return (
    <div className="min-h-screen bg-muted/20 text-foreground">
      <header className="sticky top-0 z-30 border-b bg-background/95 backdrop-blur">
        <div className="flex h-16 items-center px-8">
          <Link
            to={returnQuery ? `/discover?query=${encodeURIComponent(returnQuery)}` : "/discover"}
            className="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground"
          >
            <ArrowLeft className="size-4" />
            アプリを探す
          </Link>
          <div className="ml-6 border-l pl-6 text-sm font-semibold">nxc App Store</div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-none px-8 py-10 lg:px-12 lg:py-14">
        <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_420px]">
          <section className="min-w-0 space-y-8">
            <Card className="border-blue-200/80 bg-gradient-to-br from-white via-blue-50/30 to-blue-100/60">
              <CardContent className="p-8 lg:p-10">
                <div className="flex flex-col items-start gap-7 sm:flex-row">
                  <div className="flex size-28 shrink-0 items-center justify-center rounded-3xl bg-gradient-to-br from-primary/15 via-blue-100/70 to-white text-6xl font-bold text-primary shadow-inner shadow-white">
                    {app.name[0]}
                  </div>

                  <div className="min-w-0 pt-1">
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="secondary">{app.category}</Badge>
                      {app.tags.map((tag) => (
                        <Badge key={tag} variant="outline">{tag}</Badge>
                      ))}
                    </div>
                    <h1 className="mt-4 text-5xl font-bold tracking-tight lg:text-6xl">{app.name}</h1>
                    {app.version && (
                      <p className="mt-2 text-sm text-muted-foreground">バージョン {app.version}</p>
                    )}
                    <p className="mt-5 max-w-3xl text-lg leading-8 text-muted-foreground lg:text-xl">
                      {app.description}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="border-blue-200/80 bg-gradient-to-br from-white via-blue-50/30 to-blue-100/60">
              <CardContent className="p-8 lg:p-10">
                <h2 className="text-xl font-semibold">説明・詳細情報</h2>

                <div className="mt-7 grid gap-7 md:grid-cols-2">
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Package className="size-4 text-muted-foreground" />
                      パッケージ
                    </div>
                    <p className="break-all rounded-lg bg-muted/60 px-4 py-3 font-mono text-sm">{app.id}</p>
                  </div>

                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Tag className="size-4 text-muted-foreground" />
                      カテゴリ
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="secondary">{app.category}</Badge>
                      {app.tags.map((tag) => (
                        <Badge key={tag} variant="outline">{tag}</Badge>
                      ))}
                    </div>
                  </div>
                </div>

                {app.homepage && (
                  <div className="mt-8 border-t pt-7">
                    <p className="text-sm font-medium">Homepage</p>
                    <a
                      href={app.homepage}
                      target="_blank"
                      rel="noreferrer"
                      className="mt-2 inline-flex max-w-full items-center gap-2 break-all text-sm text-primary hover:underline"
                    >
                      {app.homepage}
                      <ExternalLink className="size-4 shrink-0" />
                    </a>
                  </div>
                )}
              </CardContent>
            </Card>
          </section>

          <aside className="lg:sticky lg:top-24 lg:self-start">
            <Card className="border-border/60 bg-gradient-to-br from-white via-white to-blue-50/50 shadow-md shadow-blue-100/40">
              <CardContent className="p-8">
                <p className="text-sm font-medium text-muted-foreground">インストール</p>
                <h2 className="mt-2 text-xl font-semibold">{app.name}</h2>

                <div className="mt-6 rounded-2xl border border-white/70 bg-gradient-to-br from-blue-50/80 to-white p-5 border border-white/90">
                  <div className="flex items-center justify-between gap-4 text-sm">
                    <span className="text-muted-foreground">バージョン</span>
                    <span className="font-medium">{app.version || '—'}</span>
                  </div>
                  <div className="mt-3 flex items-center justify-between gap-4 text-sm">
                    <span className="text-muted-foreground">パッケージ</span>
                    <span className="max-w-[190px] truncate font-mono text-xs">{app.id}</span>
                  </div>
                </div>

                {error && <p className="mt-5 text-sm text-destructive">{error}</p>}
                {refreshError && (
                  <div className="mt-5 space-y-2 text-sm text-destructive">
                    <p>インストール状態を更新できませんでした。</p>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        void refreshPackage(id)
                          .then(() => setRefreshError(null))
                          .catch((err) => setRefreshError(String(err)))
                      }}
                    >
                      再試行
                    </Button>
                  </div>
                )}
                {!statusKnown && !refreshError && (
                  <p className="mt-5 text-sm text-muted-foreground">インストール状態を確認中...</p>
                )}

                <Button
                  size="lg"
                  className="mt-6 h-12 w-full"
                  disabled={installing || installed || !statusKnown}
                  onClick={handleInstall}
                >
                  {installed ? '● インストール済み' : installing ? 'インストール中...' : 'インストール'}
                </Button>
              </CardContent>
            </Card>
          </aside>
        </div>
      </main>

      {passwordRequired && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6">
          <form onSubmit={handlePasswordSubmit} className="w-full max-w-sm rounded-xl border bg-background p-6 shadow-xl">
            <h2 className="text-lg font-semibold">認証が必要です</h2>
            <p className="mt-2 text-sm text-muted-foreground">
              {app.name}をインストールするため、パスワードを入力してください。
            </p>
            <Input
              autoFocus
              className="mt-4"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="パスワード"
              autoComplete="current-password"
            />
            <div className="mt-6 flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => { setPassword(''); setPasswordRequired(false) }}>
                キャンセル
              </Button>
              <Button type="submit" disabled={!password}>続行</Button>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}

export default AppDetail
