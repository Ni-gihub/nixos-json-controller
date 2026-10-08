import { useState, type FormEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

type InstallButtonProps = {
  packageId: string
  appName: string
  installed: boolean
  statusKnown?: boolean
  onInstalled?: () => void | Promise<void>
  className?: string
  size?: 'default' | 'sm' | 'lg'
}

function InstallButton({
  packageId,
  appName,
  installed,
  statusKnown = true,
  onInstalled,
  className,
  size = 'default',
}: InstallButtonProps) {
  const [installing, setInstalling] = useState(false)
  const [passwordRequired, setPasswordRequired] = useState(false)
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [installSucceeded, setInstallSucceeded] = useState(false)

  const effectiveInstalled = installed || installSucceeded

  const install = async (authPassword: string | null) => {
    setInstalling(true)
    setError(null)
    try {
      await invoke('install_app', { package: packageId, password: authPassword })
      setInstallSucceeded(true)
      setPassword('')
      setPasswordRequired(false)
      try {
        await onInstalled?.()
      } catch (err) {
        setError(String(err))
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setInstalling(false)
    }
  }

  const handleInstall = async () => {
    if (installing || effectiveInstalled || !statusKnown) return

    setError(null)
    try {
      const available = await invoke<boolean>('sudo_available')
      if (available) {
        await install(null)
      } else {
        setPasswordRequired(true)
      }
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
    <div className="w-full">
      {error && <p className="mb-3 text-sm text-destructive">{error}</p>}
      <Button
        type="button"
        size={size}
        className={className}
        disabled={installing || effectiveInstalled || !statusKnown}
        onClick={(event) => {
          event.preventDefault()
          event.stopPropagation()
          void handleInstall()
        }}
      >
        {effectiveInstalled ? (
          'インストール済み'
        ) : installing ? (
          'インストール中...'
        ) : (
          'インストール'
        )}
      </Button>

      {!statusKnown && !effectiveInstalled && !error && (
        <p className="mt-2 text-xs text-muted-foreground">インストール状態を確認中...</p>
      )}

      {passwordRequired && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 p-6 backdrop-blur-sm"
          onClick={() => {
            setPassword('')
            setPasswordRequired(false)
          }}
        >
          <form
            onSubmit={handlePasswordSubmit}
            className="w-full max-w-sm rounded-2xl border border-blue-100 bg-background p-6 shadow-xl"
            onClick={(event) => event.stopPropagation()}
          >
            <h2 className="text-lg font-semibold">認証が必要です</h2>
            <p className="mt-2 text-sm text-muted-foreground">
              {appName}をインストールするため、パスワードを入力してください。
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
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setPassword('')
                  setPasswordRequired(false)
                }}
              >
                キャンセル
              </Button>
              <Button
                type="submit"
                className="bg-blue-600 text-white hover:bg-blue-700"
                disabled={!password}
              >
                続行
              </Button>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}

export default InstallButton
