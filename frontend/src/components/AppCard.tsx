import { Badge } from '@/components/ui/badge'
import InstallButton from '@/components/InstallButton'
import { Link } from 'react-router'
import { useAppState } from '@/lib/app-state'

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
  const { refreshPackage } = useAppState()
  const detailPath = searchQuery ? `/apps/${id}?query=${encodeURIComponent(searchQuery)}` : `/apps/${id}`

  return (
    <div className="group py-2">
      <div className="grid grid-cols-[4.5rem_minmax(0,1fr)_auto] gap-x-4">
        <Link to={detailPath} className="row-span-3 flex size-16 shrink-0 items-center justify-center self-start rounded-2xl bg-muted/50 text-2xl font-bold text-blue-600 transition-transform duration-200 group-hover:scale-[1.03]">
          {icon ? (
            <img src={icon} alt="" className="size-14 object-contain" loading="lazy" />
          ) : (
            name[0]
          )}
        </Link>

        <Link to={detailPath} className="min-w-0">
          <div className="flex items-start gap-2">
            <h3 className="line-clamp-1 text-lg font-semibold leading-6 tracking-tight">{name}</h3>
            {installed && (
              <Badge variant="secondary" className="shrink-0 rounded-full px-2 py-0.5 text-xs font-medium">
                インストール済み
              </Badge>
            )}
          </div>
          <p className="mt-0.5 line-clamp-1 text-sm text-muted-foreground">{description}</p>
        </Link>

        <InstallButton
          packageId={id}
          appName={name}
          installed={installed}
          onInstalled={() => refreshPackage(id)}
          className="h-9 min-w-24 shrink-0 rounded-full px-4 text-sm"
          size="sm"
        />

        <Link to={detailPath} className="col-start-2 col-span-2 mt-2 flex min-w-0 items-center gap-2 overflow-hidden">
          <Badge variant="outline" className="shrink-0 rounded-full border-border px-2.5 py-1 text-xs font-medium">
            {category}
          </Badge>
          {tags.slice(0, 2).map((tag) => (
            <Badge key={tag} variant="secondary" className="shrink-0 rounded-full px-2.5 py-1 text-xs font-medium">
              {tag}
            </Badge>
          ))}
        </Link>
      </div>
    </div>
  )
}

export default AppCard
