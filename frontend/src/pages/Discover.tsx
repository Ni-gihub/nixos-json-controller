import { useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'
import { Button } from '@/components/ui/button'
import { categories, searchApps } from '@/catalog'
import { useAppState } from '@/lib/app-state'

function Discover() {
  const [query, setQuery] = useState('')
  const { states, loading, error, refreshAll } = useAppState()

  const visibleApps = useMemo(() => searchApps(query), [query])

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
          <h1 className="text-xl font-semibold">nxc App Store</h1>
          <div className="relative w-72">
            <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="pl-9"
              placeholder="Search apps..."
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-6 py-10">
        <section className="mb-10">
          <h2 className="text-4xl font-bold tracking-tight">Discover</h2>
          <p className="mt-2 text-muted-foreground">
            Find apps for your NixOS system.
          </p>
        </section>

        <section className="mb-10">
          <h3 className="mb-4 text-lg font-semibold">Categories</h3>
          <div className="flex flex-wrap gap-2">
            {categories.map((category) => (
              <Badge key={category} variant="secondary" className="cursor-pointer px-4 py-2">
                {category}
              </Badge>
            ))}
          </div>
        </section>

        <section>
          <div className="mb-4 flex items-center justify-between">
            <h3 className="text-lg font-semibold">Featured Apps</h3>
            {loading && <span className="text-sm text-muted-foreground">Checking system state...</span>}
            {error && (
              <Button variant="outline" size="sm" onClick={() => void refreshAll()}>
                Retry status check
              </Button>
            )}
          </div>

          <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {visibleApps.map((app) => (
              <AppCard
                key={app.id}
                id={app.id}
                name={app.name}
                description={app.description}
                category={app.category}
                installed={states[app.id]?.active ?? false}
              />
            ))}
          </div>
        </section>
      </main>
    </div>
  )
}

export default Discover
