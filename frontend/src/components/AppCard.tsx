import { Badge } from "@/components/ui/badge";
import InstallButton from "@/components/InstallButton";
import AppIcon from "@/components/AppIcon";
import { Link } from "react-router";

type AppCardProps = {
  id: string;
  name: string;
  description: string;
  category: string;
  tags: string[];
  installed: boolean;
  statusKnown: boolean;
  statusLoading: boolean;
  icon?: string;
  iconCandidates?: string[];
  searchQuery?: string;
};

/** Displays an application search result with its metadata and install action. */
function AppCard({
  id,
  name,
  description,
  category,
  tags,
  installed,
  statusKnown,
  statusLoading,
  icon,
  iconCandidates,
  searchQuery,
}: AppCardProps) {
  const detailPath = searchQuery
    ? `/apps/${id}?query=${encodeURIComponent(searchQuery)}`
    : `/apps/${id}`;

  return (
    <div className="group py-3">
      <div className="grid grid-cols-[4rem_minmax(0,1fr)] gap-x-4 sm:grid-cols-[4.5rem_minmax(0,1fr)_9rem]">
        <Link
          to={detailPath}
          className="row-span-3 flex size-16 shrink-0 items-center justify-center self-start rounded-2xl border border-primary/10 bg-primary/5 text-xl font-bold text-primary shadow-sm transition-transform duration-200 motion-reduce:transition-none group-hover:scale-[1.03] motion-reduce:group-hover:scale-100"
        >
          <AppIcon
            name={name}
            packageId={id}
            icon={icon}
            iconCandidates={iconCandidates}
            imageClassName="size-12 object-contain"
            fallbackClassName="size-8 text-foreground/70"
          />
        </Link>

        <Link to={detailPath} className="min-w-0">
          <div className="flex items-start gap-2">
            <h3 className="line-clamp-1 text-xl font-semibold leading-7 tracking-tight">
              {name}
            </h3>
            {installed && (
              <Badge
                variant="secondary"
                className="shrink-0 rounded-full px-2.5 py-0.5 text-xs font-medium"
              >
                インストール済み
              </Badge>
            )}
          </div>
          <p className="mt-1 line-clamp-2 text-base leading-6 text-muted-foreground">
            {description}
          </p>
        </Link>

        <div className="col-start-2 mt-2 w-full text-right sm:col-start-3 sm:mt-0">
          <InstallButton
            packageId={id}
            appName={name}
            installed={installed}
            statusKnown={statusKnown}
            statusLoading={statusLoading}
            className="ml-auto h-9 w-36 rounded-full px-3 text-sm whitespace-nowrap"
            size="sm"
          />
        </div>

        <Link
          to={detailPath}
          className="col-start-2 col-span-1 mt-3 flex min-w-0 items-center gap-2 overflow-hidden sm:col-span-2"
        >
          <Badge
            variant="outline"
            className="shrink-0 rounded-full border-border px-2.5 py-0.5 text-sm font-medium"
          >
            {category}
          </Badge>
          {tags.slice(0, 2).map((tag) => (
            <Badge
              key={tag}
              variant="secondary"
              className="shrink-0 rounded-full px-2 py-0.5 text-xs font-medium"
            >
              {tag}
            </Badge>
          ))}
        </Link>
      </div>
    </div>
  );
}

export default AppCard;
