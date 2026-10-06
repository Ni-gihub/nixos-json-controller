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
  installed: boolean
}

function AppCard({ id, name, description, category, installed }: AppCardProps) {
  return (
    <Link to={`/apps/${id}`} className="block">
      <Card className="h-full transition-colors hover:bg-muted/50">
        <CardHeader>
          <div className="mb-2 flex size-14 items-center justify-center rounded-xl bg-muted text-2xl font-bold">
            {name[0]}
          </div>

          <div className="flex items-center justify-between gap-2">
            <CardTitle>{name}</CardTitle>
            {installed && <Badge>Installed</Badge>}
          </div>
          <CardDescription>{description}</CardDescription>
        </CardHeader>

        <CardFooter>
          <Badge variant="outline">{category}</Badge>
        </CardFooter>
      </Card>
    </Link>
  )
}

export default AppCard