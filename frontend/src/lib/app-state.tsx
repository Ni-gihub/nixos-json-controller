import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { apps } from '@/data/apps'

export type AppState = {
  name: string
  configured: boolean
  active: boolean
}

type AppStateContextValue = {
  states: Record<string, AppState>
  loading: boolean
  error: string | null
  refreshAll: () => Promise<void>
  refreshPackage: (packageName: string) => Promise<void>
}

const AppStateContext = createContext<AppStateContextValue | null>(null)

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [states, setStates] = useState<Record<string, AppState>>({})
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refreshAll = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<AppState[]>('get_app_states', {
        packages: apps.map((app) => app.id),
      })
      setStates(Object.fromEntries(result.map((state) => [state.name, state])))
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  const refreshPackage = useCallback(async (packageName: string) => {
    setError(null)
    const state = await invoke<AppState>('get_app_state', { package: packageName })
    setStates((current) => ({ ...current, [state.name]: state }))
  }, [])

  useEffect(() => {
    void refreshAll()
  }, [refreshAll])

  const value = useMemo(
    () => ({ states, loading, error, refreshAll, refreshPackage }),
    [states, loading, error, refreshAll, refreshPackage],
  )

  return <AppStateContext.Provider value={value}>{children}</AppStateContext.Provider>
}

export function useAppState() {
  const context = useContext(AppStateContext)
  if (!context) {
    throw new Error('useAppState must be used within AppStateProvider')
  }
  return context
}
