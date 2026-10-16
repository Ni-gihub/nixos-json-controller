import { ArrowUpRight, Star, TrendingUp } from "lucide-react";
import { Link } from "react-router";
import { Badge } from "@/components/ui/badge";
import AppIcon from "@/components/AppIcon";
import InstallButton from "@/components/InstallButton";
import type { App } from "@/catalog/apps";
import type { RecommendationRecord } from "@/catalog/recommendation-data";
import { useAppState } from "@/lib/app-state";

type RecommendationAppCardProps = {
  app: App;
  recommendation: RecommendationRecord;
  rank: number;
  featured: boolean;
  installed: boolean;
};

/** A restrained App Store-style card, with a denser layout for genre rankings. */
export default function RecommendationAppCard({
  app,
  recommendation,
  rank,
  featured,
  installed,
}: RecommendationAppCardProps) {
  const { refreshPackage } = useAppState();
  const detailPath = `/apps/${app.id}`;
  const popularity = recommendation.popularity.toFixed(2);

  if (featured) {
    return (
      <article className="group col-span-full overflow-hidden rounded-2xl border border-primary/15 bg-card shadow-[0_12px_32px_-24px_rgba(0,92,230,0.48)] transition-[border-color,box-shadow] duration-200 hover:border-primary/35 hover:shadow-[0_18px_38px_-24px_rgba(0,92,230,0.38)]">
        <div className="grid lg:grid-cols-[minmax(0,1fr)_18rem]">
          <div className="flex items-start gap-5 p-5 sm:gap-6 sm:p-7">
            <Link
              to={detailPath}
              aria-label={`${app.name}の詳細`}
              className="flex size-24 shrink-0 items-center justify-center rounded-2xl border border-primary/15 bg-primary/5 shadow-sm transition-colors group-hover:bg-primary/10 sm:size-28"
            >
              <AppIcon
                name={app.name}
                packageId={app.id}
                icon={app.icon}
                iconCandidates={app.iconCandidates}
                imageClassName="size-16 object-contain sm:size-20"
                fallbackClassName="size-12 text-foreground/70"
              />
            </Link>

            <div className="min-w-0 flex-1">
              <div className="inline-flex items-center gap-2 rounded-full bg-primary/10 px-3 py-1 text-sm font-semibold text-primary">
                <Star className="size-4" />
                総合ランキング {rank}位
              </div>
              <div className="mt-2 flex flex-wrap items-center gap-2">
                <Link
                  to={detailPath}
                  className="text-2xl font-semibold tracking-tight hover:text-primary sm:text-3xl"
                >
                  {app.name}
                </Link>
                {installed && (
                  <Badge variant="secondary" className="rounded-md">
                    インストール済み
                  </Badge>
                )}
              </div>
              <p className="mt-2 line-clamp-3 max-w-2xl text-sm leading-6 text-muted-foreground sm:text-base">
                {app.description || "Nixpkgsからインストールできるアプリです。"}
              </p>
              <div className="mt-4 flex flex-wrap items-center gap-2 text-sm">
                <span className="rounded-md bg-primary/10 px-2 py-1 font-semibold text-primary">
                  {app.category}
                </span>
                {app.tags.slice(0, 2).map((tag) => (
                  <span
                    key={tag}
                    className="rounded-md bg-secondary px-2 py-1 text-secondary-foreground"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          </div>

          <aside className="border-t border-primary/10 bg-primary/5 px-5 py-5 sm:px-7 sm:py-6 lg:border-l lg:border-l-primary/10 lg:border-t-0">
            <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <TrendingUp className="size-4 text-primary" />
              Arch Linuxでの人気度
            </div>
            <div className="mt-2 flex items-baseline gap-1">
              <span className="text-3xl font-bold tracking-tight tabular-nums text-primary">
                {popularity}
              </span>
              <span className="text-sm text-muted-foreground">%</span>
            </div>
            <div
              aria-label={`Arch Linux pkgstats popularity ${popularity}%`}
              role="img"
              className="mt-3 h-1.5 overflow-hidden rounded-full bg-primary/15"
            >
              <div
                className="h-full rounded-full bg-primary"
                style={{
                  width: `${Math.min(100, Math.max(0, recommendation.popularity))}%`,
                }}
              />
            </div>
            <p className="mt-3 text-xs leading-5 text-muted-foreground">
              Arch
              Linuxの公開統計（pkgstats）に基づく割合です。NixOSでの利用率ではありません。
            </p>
            <div className="mt-5 flex flex-wrap gap-2">
              <Link
                to={detailPath}
                className="inline-flex h-9 items-center justify-center gap-1.5 rounded-full border border-primary/20 bg-card px-3 text-sm font-semibold text-primary transition-colors hover:border-primary/35 hover:bg-primary/5"
              >
                詳細を見る
                <ArrowUpRight className="size-4" />
              </Link>
              <InstallButton
                packageId={app.id}
                appName={app.name}
                installed={installed}
                onInstalled={() => refreshPackage(app.id)}
                className="h-9 rounded-full px-4 text-sm"
                size="sm"
              />
            </div>
          </aside>
        </div>
      </article>
    );
  }

  return (
    <article className="group flex h-full min-w-0 flex-col rounded-2xl border border-border/80 bg-card p-4 shadow-[0_4px_16px_-14px_rgba(20,55,105,0.5)] transition-[border-color,box-shadow,transform] duration-200 hover:-translate-y-0.5 hover:border-primary/30 hover:shadow-[0_12px_28px_-20px_rgba(0,122,255,0.42)]">
      <div className="flex items-center justify-between gap-3 border-b border-border/80 pb-3">
        <div className="flex items-baseline gap-1.5">
          <span className="text-2xl font-bold leading-none tracking-tight tabular-nums text-primary">
            {String(rank).padStart(2, "0")}
          </span>
          <span className="text-xs text-muted-foreground">位</span>
        </div>
        <span className="rounded-md bg-primary/10 px-2 py-1 text-xs font-semibold tabular-nums text-primary">
          {popularity}%
        </span>
      </div>

      <div className="mt-4 flex items-start gap-3">
        <Link
          to={detailPath}
          aria-label={`${app.name}の詳細`}
          className="flex size-14 shrink-0 items-center justify-center rounded-2xl border border-primary/15 bg-primary/5 shadow-sm transition-colors group-hover:bg-primary/10"
        >
          <AppIcon
            name={app.name}
            packageId={app.id}
            icon={app.icon}
            iconCandidates={app.iconCandidates}
            imageClassName="size-10 object-contain"
            fallbackClassName="size-8 text-foreground/65"
          />
        </Link>
        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 flex-wrap items-center gap-1.5">
            <Link
              to={detailPath}
              className="line-clamp-1 text-base font-semibold leading-6 tracking-tight hover:text-primary"
            >
              {app.name}
            </Link>
            {installed && (
              <Badge
                variant="secondary"
                className="rounded-md px-1.5 py-0 text-[10px]"
              >
                インストール済み
              </Badge>
            )}
          </div>
          <p className="mt-1 line-clamp-3 min-h-[3.75rem] text-sm leading-5 text-muted-foreground">
            {app.description || "Nixpkgsで利用できるアプリです。"}
          </p>
        </div>
      </div>

      <div className="mt-4 flex min-w-0 items-center gap-2 border-t border-primary/10 pt-3 text-xs">
        <span className="shrink-0 rounded-md bg-primary/10 px-2 py-1 font-semibold text-primary">
          {app.category}
        </span>
        {app.tags.slice(0, 1).map((tag) => (
          <span
            key={tag}
            className="min-w-0 truncate rounded-md bg-secondary px-2 py-1 text-secondary-foreground"
          >
            {tag}
          </span>
        ))}
      </div>

      <div className="mt-auto flex items-center gap-2 pt-4">
        <Link
          to={detailPath}
          className="inline-flex h-9 min-w-0 flex-1 items-center justify-center gap-1 rounded-full px-2 text-sm font-semibold text-primary transition-colors hover:bg-primary/10 hover:text-primary"
        >
          詳細を見る
          <ArrowUpRight className="size-4 shrink-0" />
        </Link>
        <div className="min-w-24 shrink-0">
          <InstallButton
            packageId={app.id}
            appName={app.name}
            installed={installed}
            onInstalled={() => refreshPackage(app.id)}
            className="h-9 w-full rounded-full px-3 text-sm"
            size="sm"
          />
        </div>
      </div>
    </article>
  );
}
