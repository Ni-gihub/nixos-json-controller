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
    <div className="group py-4">
      <div className="grid grid-cols-[5rem_minmax(0,1fr)_auto] gap-x-5">
        <Link to={detailPath} className="row-span-3 flex size-18 shrink-0 items-center justify-center self-start rounded-2xl bg-muted/50 text-2xl font-bold text-blue-600 transition-transform duration-200 group-hover:scale-[1.03]">
          {icon ? (
            <img src={icon} alt="" className="size-16 object-contain" loading="lazy" />
          ) : (
            name[0]
          )}
        </Link>

        <Link to={detailPath} className="min-w-0">
          <div className="flex items-start gap-2">
            <h3 className="line-clamp-1 text-xl font-semibold leading-7 tracking-tight">{name}</h3>
            {installed && (
              <Badge variant="secondary" className="shrink-0 rounded-full px-2.5 py-0.5 text-sm font-medium">
                インストール済み
              </Badge>
            )}
          </div>
          <p className="mt-1 line-clamp-2 text-base leading-6 text-muted-foreground">{description}</p>
        </Link>

        <InstallButton
          packageId={id}
          appName={name}
          installed={installed}
          onInstalled={() => refreshPackage(id)}
          className="h-9 min-w-20 shrink-0 rounded-full px-3 text-sm"
          size="sm"
        />

        <Link to={detailPath} className="col-start-2 col-span-2 mt-2 flex min-w-0 items-center gap-2 overflow-hidden">
          <Badge variant="outline" className="shrink-0 rounded-full border-border px-2.5 py-1 text-sm font-medium">
            {category}
          </Badge>
          {tags.slice(0, 2).map((tag) => (
            <Badge key={tag} variant="secondary" className="shrink-0 rounded-full px-2.5 py-1 text-sm font-medium">
              {tag}
            </Badge>
          ))}
        </Link>
      </div>
    </div>
  )
}

export default AppCard
