import { Badge } from '@/components/ui/badge'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
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
    <Card className="group flex h-full flex-col overflow-hidden rounded-2xl border-blue-100 bg-white/90 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-300 hover:bg-blue-50/40">
      <Link to={detailPath} className="block flex-1">
        <CardHeader className="p-7 sm:p-8">
          <div className="flex items-start gap-5">
            <div className="flex size-24 shrink-0 items-center justify-center rounded-2xl border border-blue-100 bg-gradient-to-br from-blue-50 via-white to-indigo-100 text-4xl font-bold text-blue-600 transition-transform duration-200 group-hover:scale-[1.03]">
              {icon ? (
                <img src={icon} alt="" className="size-16 object-contain" loading="lazy" />
              ) : (
                name[0]
              )}
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-start justify-between gap-3">
                <CardTitle className="line-clamp-2 text-2xl leading-8">{name}</CardTitle>
                {installed && (
                  <Badge className="shrink-0 rounded-full bg-blue-600 px-3 py-1.5 text-xs text-white hover:bg-blue-600">
                    インストール済み
                  </Badge>
                )}
              </div>

              <CardDescription className="mt-3 line-clamp-4 text-base leading-7 sm:text-lg">
                {description}
              </CardDescription>
            </div>
          </div>

          <div className="mt-7 flex items-center gap-2 overflow-hidden border-t border-blue-50 pt-5">
            <Badge variant="outline" className="shrink-0 rounded-full border-blue-200 bg-blue-50/60 px-3 py-1.5 text-sm text-blue-700">
              {category}
            </Badge>
            {tags.slice(0, 2).map((tag) => (
              <Badge key={tag} variant="secondary" className="shrink-0 rounded-full px-3 py-1.5 text-sm">
                {tag}
              </Badge>
            ))}
          </div>
        </CardHeader>
      </Link>

      <div className="border-t border-blue-50 px-7 pb-7 pt-5 sm:px-8 sm:pb-8">
        <InstallButton
          packageId={id}
          appName={name}
          installed={installed}
          onInstalled={() => refreshPackage(id)}
          className="h-12 w-full rounded-xl bg-blue-600 text-base text-white hover:bg-blue-700"
          size="lg"
        />
      </div>
    </Card>
  )
}

export default AppCard
