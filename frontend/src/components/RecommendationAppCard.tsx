import { ArrowUpRight, Sparkles, TrendingUp } from "lucide-react";
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

/** A store-style recommendation card, separate from the unchanged search result row. */
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
  const rankLabel = String(rank).padStart(2, "0");

  if (featured) {
    return (
      <article className="group relative col-span-full overflow-hidden rounded-3xl border border-blue-200/80 bg-gradient-to-br from-blue-50 via-white to-indigo-50 shadow-sm transition-shadow duration-200 hover:shadow-md dark:border-blue-900/60 dark:from-blue-950/30 dark:via-card dark:to-indigo-950/20">
        <div
          aria-hidden="true"
          className="pointer-events-none absolute -right-12 -top-20 size-72 rounded-full bg-blue-400/15 blur-3xl"
        />
        <div
          aria-hidden="true"
          className="pointer-events-none absolute -bottom-24 left-1/3 size-60 rounded-full bg-indigo-400/10 blur-3xl"
        />
        <div className="relative grid gap-5 p-5 sm:grid-cols-[auto_minmax(0,1fr)] sm:items-center sm:gap-7 sm:p-7 lg:grid-cols-[7rem_minmax(0,1fr)_18rem]">
          <Link
            to={detailPath}
            aria-label={`${app.name}の詳細`}
            className="flex size-24 items-center justify-center rounded-3xl border border-white/80 bg-white/85 shadow-sm transition-transform duration-200 group-hover:scale-[1.03] sm:size-28 dark:border-white/10 dark:bg-slate-900/70"
          >
            <AppIcon
              name={app.name}
              packageId={app.id}
              icon={app.icon}
              iconCandidates={app.iconCandidates}
              imageClassName="size-16 object-contain sm:size-20"
              fallbackClassName="size-12 text-blue-700/70 dark:text-blue-200"
            />
          </Link>

          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <Badge className="gap-1 rounded-full bg-amber-100 px-2.5 text-amber-900 hover:bg-amber-100 dark:bg-amber-300/15 dark:text-amber-200">
                <Sparkles className="size-3.5" />
                総合ランキング {rank}位
              </Badge>
            </div>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <Link
                to={detailPath}
                className="text-2xl font-bold tracking-tight hover:text-blue-700 sm:text-3xl dark:hover:text-blue-300"
              >
                {app.name}
              </Link>
              {installed && (
                <Badge variant="secondary" className="rounded-full">
                  インストール済み
                </Badge>
              )}
            </div>
            <p className="mt-2 line-clamp-3 max-w-2xl text-sm leading-6 text-muted-foreground sm:text-base">
              {app.description || "Nixpkgsからインストールできるアプリです。"}
            </p>
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <Badge
                variant="outline"
                className="rounded-full border-blue-200/90 bg-white/60"
              >
                {app.category}
              </Badge>
              {app.tags.slice(0, 2).map((tag) => (
                <Badge key={tag} variant="secondary" className="rounded-full">
                  {tag}
                </Badge>
              ))}
            </div>
          </div>

          <div className="rounded-2xl border border-white/80 bg-white/75 p-4 shadow-sm backdrop-blur-sm dark:border-white/10 dark:bg-slate-950/35">
            <div className="flex items-center justify-between gap-3">
              <span className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                <TrendingUp className="size-4 text-blue-600 dark:text-blue-300" />
                Arch Linuxでの人気度
              </span>
              <span className="text-lg font-bold tabular-nums text-blue-700 dark:text-blue-300">
                {popularity}%
              </span>
            </div>
            <div
              aria-label={`Arch Linux pkgstats popularity ${popularity}%`}
              role="img"
              className="mt-3 h-2 overflow-hidden rounded-full bg-blue-100 dark:bg-blue-950"
            >
              <div
                className="h-full rounded-full bg-gradient-to-r from-blue-500 to-indigo-500"
                style={{
                  width: `${Math.min(100, Math.max(0, recommendation.popularity))}%`,
                }}
              />
            </div>
            <p className="mt-2 text-xs leading-5 text-muted-foreground">
              Arch Linuxの公開統計（pkgstats）に基づく割合です。NixOSでの利用率ではありません。
            </p>
            <div className="mt-4 grid grid-cols-2 gap-2">
              <Link
                to={detailPath}
                className="inline-flex h-10 min-w-0 items-center justify-center gap-1 whitespace-nowrap rounded-full border border-border bg-background px-2 text-sm font-medium transition-colors hover:bg-muted"
              >
                詳細を見る
                <ArrowUpRight className="size-4 shrink-0" />
              </Link>
              <InstallButton
                packageId={app.id}
                appName={app.name}
                installed={installed}
                onInstalled={() => refreshPackage(app.id)}
                className="h-10 w-full whitespace-nowrap rounded-full px-2 text-sm"
                size="sm"
              />
            </div>
          </div>
        </div>
      </article>
    );
  }

  return (
    <article className="group flex h-full min-w-0 flex-col rounded-2xl border border-border/70 bg-card p-4 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:shadow-md dark:hover:border-blue-900">
      <div className="flex items-center justify-between gap-3">
        <span className="inline-flex items-center gap-2 text-xs font-semibold tracking-wide text-muted-foreground">
          <span className="flex size-8 items-center justify-center rounded-full bg-muted text-sm font-bold tabular-nums text-foreground">
            {rankLabel}
          </span>
          ジャンル内ランキング
        </span>
        <Badge
          variant="secondary"
          className="shrink-0 rounded-full px-2.5 tabular-nums"
        >
          {popularity}%
        </Badge>
      </div>

      <div className="mt-4 flex items-start gap-3">
        <Link
          to={detailPath}
          aria-label={`${app.name}の詳細`}
          className="flex size-16 shrink-0 items-center justify-center rounded-2xl bg-muted/70 transition-transform duration-200 group-hover:scale-[1.03]"
        >
          <AppIcon
            name={app.name}
            packageId={app.id}
            icon={app.icon}
            iconCandidates={app.iconCandidates}
            imageClassName="size-11 object-contain"
            fallbackClassName="size-8 text-foreground/65"
          />
        </Link>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-1.5">
            <Link
              to={detailPath}
              className="line-clamp-1 text-base font-semibold leading-6 tracking-tight hover:text-blue-700 dark:hover:text-blue-300"
            >
              {app.name}
            </Link>
            {installed && (
              <Badge variant="secondary" className="rounded-full text-[10px]">
                インストール済み
              </Badge>
            )}
          </div>
          <p className="mt-1 line-clamp-3 min-h-[3.75rem] text-sm leading-5 text-muted-foreground">
            {app.description || "Nixpkgsで利用できるアプリです。"}
          </p>
        </div>
      </div>

      <div className="mt-4 h-1.5 overflow-hidden rounded-full bg-muted">
        <div
          aria-hidden="true"
          className="h-full rounded-full bg-gradient-to-r from-blue-500 to-indigo-500"
          style={{
            width: `${Math.min(100, Math.max(0, recommendation.popularity))}%`,
          }}
        />
      </div>

      <div className="mt-3 flex min-w-0 items-center gap-1.5 overflow-hidden">
        <Badge variant="outline" className="shrink-0 rounded-full text-xs">
          {app.category}
        </Badge>
        {app.tags.slice(0, 1).map((tag) => (
          <Badge
            key={tag}
            variant="secondary"
            className="min-w-0 truncate rounded-full text-xs"
          >
            {tag}
          </Badge>
        ))}
      </div>

      <div className="mt-auto flex items-center gap-2 pt-4">
        <Link
          to={detailPath}
          className="inline-flex h-9 min-w-0 flex-1 items-center justify-center gap-1 whitespace-nowrap rounded-full border border-border px-2 text-sm font-medium transition-colors hover:bg-muted"
        >
          詳細を見る
          <ArrowUpRight className="size-4" />
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
      <p className="mt-2 text-[11px] leading-4 text-muted-foreground">
        Arch Linuxの公開統計（pkgstats）に基づく割合です。NixOSでの利用率ではありません。
      </p>
    </article>
  );
}
