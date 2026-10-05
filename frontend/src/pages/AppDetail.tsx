import { useState, type FormEvent } from 'react'
import { ArrowLeft } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { Link, useParams } from 'react-router'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'

const apps = {
  firefox: {
    name: 'Firefox',
    description: 'Fast, private web browser',
    category: 'Browser',
    details: 'A fast and privacy-focused web browser from Mozilla.',
  },

  vlc: {
    name: 'VLC',
    description: 'Play almost any media format',
    category: 'Media',
    details:
      'A versatile media player that supports a wide range of audio and video formats.',
  },

  gimp: {
    name: 'GIMP',
    description:
      'Powerful open-source image editor for photo manipulation and graphic design.',
    category: 'Graphics',
    details:
      'A powerful open-source image editor for photo manipulation and graphic design.',
  },
}

function AppDetail() {
  const { id } = useParams()
  const [installing, setInstalling] = useState(false)
  const [installed, setInstalled] = useState(false)
  const [passwordRequired, setPasswordRequired] = useState(false)
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)

  const app = id ? apps[id as keyof typeof apps] : undefined

  if (!app) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold">App not found</h1>

          <Link
            to="/discover"
            className="mt-4 inline-block text-primary hover:underline"
          >
            Back to Discover
          </Link>
        </div>
      </div>
    )
  }

  const install = async (authPassword: string | null) => {
    if (!id) {
      return
    }

    setInstalling(true)
    setError(null)

    try {
      await invoke('install_app', {
        package: id,
        password: authPassword,
      })

      setInstalled(true)
      setPassword('')
      setPasswordRequired(false)
    } catch (err) {
      setError(String(err))
    } finally {
      setInstalling(false)
    }
  }

  const handleInstall = async () => {
    if (installing || installed) {
      return
    }

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

  const handlePasswordSubmit = async (
    event: FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault()

    if (!password) {
      return
    }

    setPasswordRequired(false)
    await install(password)
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b">
        <div className="mx-auto max-w-7xl px-6 py-4">
          <Link
            to="/discover"
            className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
          >
            <ArrowLeft className="size-4" />
            Back to Discover
          </Link>
        </div>
      </header>

      <main className="mx-auto max-w-4xl px-6 py-12">
        <Card>
          <CardHeader>
            <div className="flex items-start gap-6">
              <div className="flex size-24 shrink-0 items-center justify-center rounded-2xl bg-muted text-4xl font-bold">
                {app.name[0]}
              </div>

              <div>
                <Badge variant="secondary">{app.category}</Badge>

                <CardTitle className="mt-3 text-3xl">
                  {app.name}
                </CardTitle>

                <p className="mt-2 text-muted-foreground">
                  {app.description}
                </p>
              </div>
            </div>
          </CardHeader>

          <CardContent>
            <p className="mb-8 text-muted-foreground">
              {app.details}
            </p>

            {error && (
              <p className="mb-4 text-sm text-destructive">
                {error}
              </p>
            )}

            <Button
              size="lg"
              className="w-full sm:w-auto"
              disabled={installing || installed}
              onClick={handleInstall}
            >
              {installed
                ? '● Installed'
                : installing
                  ? 'Installing...'
                  : 'Install'}
            </Button>
          </CardContent>
        </Card>
      </main>

      {passwordRequired && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6">
          <form
            onSubmit={handlePasswordSubmit}
            className="w-full max-w-sm rounded-xl border bg-background p-6 shadow-xl"
          >
            <h2 className="text-lg font-semibold">Authentication required</h2>

            <p className="mt-2 text-sm text-muted-foreground">
              Enter your password to install {app.name}.
            </p>

            <Input
              autoFocus
              className="mt-4"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Password"
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
                Cancel
              </Button>

              <Button type="submit" disabled={!password}>
                Continue
              </Button>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}

export default AppDetail
