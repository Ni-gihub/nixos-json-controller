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
    <Link to={`/apps/${id}`} className="block">
      <Card className="h-full rounded-2xl border-border/70 bg-card shadow-sm transition-all hover:-translate-y-0.5 hover:shadow-md">
        <CardHeader className="flex-row items-center gap-4 p-4 sm:p-5">
          <div className="flex size-16 shrink-0 items-center justify-center rounded-2xl bg-primary/10 text-xl font-bold text-primary">
            {name[0]}
          </div>

          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <CardTitle className="truncate text-base">{name}</CardTitle>
              {installed && (
                <Badge className="shrink-0 rounded-full px-2 py-0.5 text-[11px]">
                  インストール済み
                </Badge>
              )}
            </div>

            <CardDescription className="mt-1 line-clamp-2 text-sm">
              {description}
            </CardDescription>

            <div className="mt-2 flex items-center gap-2 overflow-hidden">
              <Badge variant="outline" className="shrink-0 rounded-full text-[11px]">
                {category}
              </Badge>
              {tags.slice(0, 1).map((tag) => (
                <Badge key={tag} variant="secondary" className="shrink-0 rounded-full text-[11px]">
                  {tag}
                </Badge>
              ))}
            </div>
          </div>
        </CardHeader>
      </Card>
    </Link>
  )
}

export default AppCard
