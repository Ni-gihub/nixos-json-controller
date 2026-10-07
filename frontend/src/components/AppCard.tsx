import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Link } from 'react-router'

type AppCardProps = {
  id: string
  name: string
  description: string
  category: string
  tags: string[]
  installed: boolean
  searchQuery?: string
}

function AppCard({ id, name, description, category, tags, installed, searchQuery }: AppCardProps) {
  return (
    <Link to={searchQuery ? `/apps/${id}?query=${encodeURIComponent(searchQuery)}` : `/apps/${id}`} className="block h-full">
      <Card className="h-full rounded-2xl border-blue-100/70 bg-gradient-to-br from-white via-blue-50/20 to-blue-100/70 shadow-md shadow-blue-100/60 transition-all duration-200 hover:-translate-y-1 hover:border-primary/30 hover:shadow-xl hover:shadow-blue-200/40">
        <CardHeader className="p-7">
          <div className="flex items-start gap-4">
            <div className="flex size-20 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-white via-blue-100/80 to-primary/15 text-3xl font-bold text-primary shadow-inner shadow-white ring-1 ring-blue-100/70">
              {name[0]}
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-start justify-between gap-3">
                <CardTitle className="line-clamp-2 text-xl leading-8">{name}</CardTitle>
                {installed && (
                  <Badge className="shrink-0 rounded-full px-2.5 py-0.5 text-[11px]">
                    インストール済み
                  </Badge>
                )}
              </div>

              <CardDescription className="mt-2 line-clamp-3 text-base leading-6">
                {description}
              </CardDescription>
            </div>
          </div>

          <div className="mt-6 flex items-center gap-2 overflow-hidden">
            <Badge variant="outline" className="shrink-0 rounded-full px-2.5 py-1 text-sm">
              {category}
            </Badge>
            {tags.slice(0, 2).map((tag) => (
              <Badge key={tag} variant="secondary" className="shrink-0 rounded-full px-2.5 py-1 text-xs">
                {tag}
              </Badge>
            ))}
          </div>
        </CardHeader>
      </Card>
    </Link>
  )
}

export default AppCard
