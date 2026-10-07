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
}

function AppCard({ id, name, description, category, tags, installed }: AppCardProps) {
  return (
    <Link to={`/apps/${id}`} className="block h-full">
      <Card className="h-full rounded-xl border-border/70 bg-card shadow-sm transition-all duration-200 hover:-translate-y-0.5 hover:border-border hover:shadow-md">
        <CardHeader className="p-6">
          <div className="flex items-start gap-4">
            <div className="flex size-16 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-xl font-bold text-primary">
              {name[0]}
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-start justify-between gap-3">
                <CardTitle className="line-clamp-2 text-base leading-6">{name}</CardTitle>
                {installed && (
                  <Badge className="shrink-0 rounded-full px-2.5 py-0.5 text-[11px]">
                    インストール済み
                  </Badge>
                )}
              </div>

              <CardDescription className="mt-2 line-clamp-3 text-sm leading-5">
                {description}
              </CardDescription>
            </div>
          </div>

          <div className="mt-5 flex items-center gap-2 overflow-hidden">
            <Badge variant="outline" className="shrink-0 rounded-full px-2.5 py-1 text-xs">
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
