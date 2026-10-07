import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardDescription,
  CardFooter,
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
      <Card className="h-full transition-colors hover:bg-muted/50">
        <CardHeader>
          <div className="mb-2 flex size-16 items-center justify-center rounded-xl bg-muted text-2xl font-bold">
            {name[0]}
          </div>

          <div className="flex items-center justify-between gap-2">
            <CardTitle>{name}</CardTitle>
            {installed && <Badge>インストール済み</Badge>}
          </div>
          <CardDescription>{description}</CardDescription>
        </CardHeader>

        <CardFooter className="flex flex-wrap gap-2">
          <Badge variant="outline">{category}</Badge>
          {tags.slice(0, 2).map((tag) => (
            <Badge key={tag} variant="secondary">
              {tag}
            </Badge>
          ))}
        </CardFooter>
      </Card>
    </Link>
  )
}

export default AppCard
