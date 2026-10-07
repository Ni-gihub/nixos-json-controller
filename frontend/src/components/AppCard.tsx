import { Badge } from '@/components/ui/badge'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Link } from 'react-router'

type AppCardProps = {
  id: string
  name: string
  description: string
  category: string
  tags: string[]
  installed: boolean
  icon?: string
  searchQuery?: string
}

function AppCard({ id, name, description, category, tags, installed, icon, searchQuery }: AppCardProps) {
  return (
    <Link to={searchQuery ? `/apps/${id}?query=${encodeURIComponent(searchQuery)}` : `/apps/${id}`} className="block h-full">
      <Card className="group h-full overflow-hidden rounded-2xl border-blue-100 bg-white/90 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-300 hover:bg-blue-50/40">
        <CardHeader className="p-6 sm:p-7">
          <div className="flex items-start gap-4">
            <div className="flex size-20 shrink-0 items-center justify-center rounded-2xl border border-blue-100 bg-gradient-to-br from-blue-50 via-white to-indigo-100 text-3xl font-bold text-blue-600 transition-transform duration-200 group-hover:scale-[1.03]">
              {name[0]}
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-start justify-between gap-3">
                <CardTitle className="line-clamp-2 text-xl leading-7">{name}</CardTitle>
                {installed && (
                  <Badge className="shrink-0 rounded-full bg-blue-600 px-2.5 py-1 text-[11px] text-white hover:bg-blue-600">
                    インストール済み
                  </Badge>
                )}
              </div>

              <CardDescription className="mt-2 line-clamp-3 text-sm leading-6 sm:text-base">
                {description}
              </CardDescription>
            </div>
          </div>

          <div className="mt-6 flex items-center gap-2 overflow-hidden border-t border-blue-50 pt-4">
            <Badge variant="outline" className="shrink-0 rounded-full border-blue-200 bg-blue-50/60 px-2.5 py-1 text-sm text-blue-700">
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
