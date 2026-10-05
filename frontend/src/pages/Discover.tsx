import { Search } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import AppCard from '@/components/AppCard'

const apps = [
  {
    id: 'firefox',
    name: 'Firefox',
    description: 'Fast, private web browser',
    category: 'Browser',
  },
  {
    id: 'vlc',
    name: 'VLC',
    description: 'Play almost any media format',
    category: 'Media',
  },
  {
    id: 'gimp',
    name: 'GIMP',
    description: 'Powerful image editor',
    category: 'Graphics',
  },
]

const categories = [
  'Browser',
  'Development',
  'Media',
  'Graphics',
]

function Discover() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
          <h1 className="text-xl font-semibold">
            nxc App Store
          </h1>

          <div className="relative w-72">
            <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />

            <Input
              className="pl-9"
              placeholder="Search apps..."
            />
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-6 py-10">
        <section className="mb-10">
          <h2 className="text-4xl font-bold tracking-tight">
            Discover
          </h2>

          <p className="mt-2 text-muted-foreground">
            Find apps for your NixOS system.
          </p>
        </section>

        <section className="mb-10">
          <h3 className="mb-4 text-lg font-semibold">
            Categories
          </h3>

          <div className="flex flex-wrap gap-2">
            {categories.map((category) => (
              <Badge
                key={category}
                variant="secondary"
                className="cursor-pointer px-4 py-2"
              >
                {category}
              </Badge>
            ))}
          </div>
        </section>

        <section>
          <h3 className="mb-4 text-lg font-semibold">
            Featured Apps
          </h3>

          <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {apps.map((app) => (
              <AppCard key={app.id} {...app} />
            ))}
          </div>
        </section>
      </main>
    </div>
  )
}

export default Discover