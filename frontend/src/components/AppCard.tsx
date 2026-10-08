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
    <div className="group border-b border-blue-100 py-6 md:odd:border-r md:odd:pr-8 md:even:pl-8">
      <div className="grid grid-cols-[5rem_minmax(0,1fr)] gap-x-4 gap-y-2">
        <Link to={detailPath} className="row-span-3 flex size-20 shrink-0 items-center justify-center self-start rounded-2xl border border-blue-100 bg-gradient-to-br from-blue-50 via-white to-indigo-100 text-3xl font-bold text-blue-600 shadow-sm ring-1 ring-blue-50 transition-transform duration-200 group-hover:scale-[1.03] group-hover:shadow-md">
          {icon ? (
            <img src={icon} alt="" className="size-16 object-contain drop-shadow-sm" loading="lazy" />
          ) : (
            name[0]
          )}
        </Link>

        <div className="min-w-0">
          <Link to={detailPath} className="block min-w-0">
            <div className="flex items-start justify-between gap-3">
              <h3 className="line-clamp-1 text-xl font-bold leading-7 tracking-tight text-slate-900">{name}</h3>
              {installed && (
                <Badge className="shrink-0 rounded-full bg-blue-600 px-2.5 py-1 text-xs text-white hover:bg-blue-600">
                  インストール済み
                </Badge>
              )}
            </div>
          </Link>
        </div>

        <div className="flex min-w-0 items-center gap-3">
          <Link to={detailPath} className="min-w-0 flex-1">
            <p className="line-clamp-2 text-sm leading-6 text-slate-700">{description}</p>
          </Link>
          <InstallButton
            packageId={id}
            appName={name}
            installed={installed}
            onInstalled={() => refreshPackage(id)}
            className="h-9 shrink-0 rounded-lg bg-blue-600 px-4 text-sm text-white hover:bg-blue-700"
            size="sm"
          />
        </div>

        <Link to={detailPath} className="col-start-2 flex min-w-0 items-center gap-2 overflow-hidden">
          <Badge variant="outline" className="shrink-0 rounded-full border-blue-200 bg-blue-50 px-2.5 py-1 text-xs font-semibold text-blue-700">
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
