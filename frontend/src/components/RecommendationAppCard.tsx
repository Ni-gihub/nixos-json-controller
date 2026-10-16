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
  const rankLabel = `${rank}位`;

  if (featured) {
    return (
      <article className="group col-span-full overflow-hidden rounded-2xl border border-border bg-card transition-colors duration-200 hover:border-[#007aff]/35">
        <div className="grid gap-5 p-5 sm:grid-cols-[auto_minmax(0,1fr)] sm:items-center sm:gap-6 sm:p-6 lg:grid-cols-[7rem_minmax(0,1fr)_18rem]">
          <Link
            to={detailPath}
            aria-label={`${app.name}の詳細`}
            className="flex size-24 items-center justify-center rounded-2xl border border-border bg-muted/40 transition-colors group-hover:bg-muted sm:size-28"
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

          <div className="min-w-0">
            <Badge
              variant="outline"
              className="gap-1 rounded-full border-[#007aff]/25 bg-[#007aff]/[0.06] px-2.5 text-[#0066cc]"
            >
              <Sparkles className="size-3.5" />
              総合ランキング {rank}位
            </Badge>
            <div className="mt-2 flex flex-wrap items-center gap-2">
              <Link
                to={detailPath}
                className="text-2xl font-semibold tracking-tight hover:text-[#007aff] sm:text-3xl"
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
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <Badge variant="outline" className="rounded-full border-border">
                {app.category}
              </Badge>
              {app.tags.slice(0, 2).map((tag) => (
                <Badge key={tag} variant="secondary" className="rounded-full">
                  {tag}
                </Badge>
              ))}
            </div>
          </div>

          <div className="rounded-xl border border-border bg-muted/30 p-4 sm:col-span-2 lg:col-span-1">
            <div className="flex items-center justify-between gap-3">
              <span className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                <TrendingUp className="size-4 text-[#007aff]" />
                Arch Linuxでの人気度
              </span>
              <span className="text-lg font-semibold tabular-nums text-[#007aff]">
                {popularity}%
              </span>
            </div>
            <div
              aria-label={`Arch Linux pkgstats popularity ${popularity}%`}
              role="img"
              className="mt-3 h-1.5 overflow-hidden rounded-full bg-muted"
            >
              <div
                className="h-full rounded-full bg-[#007aff]"
                style={{
                  width: `${Math.min(100, Math.max(0, recommendation.popularity))}%`,
                }}
              />
            </div>
            <p className="mt-2 text-xs leading-5 text-muted-foreground">
              Arch
              Linuxの公開統計（pkgstats）に基づく割合です。NixOSでの利用率ではありません。
            </p>
            <div className="mt-4 grid grid-cols-2 gap-2">
              <Link
                to={detailPath}
                className="inline-flex h-10 min-w-0 items-center justify-center gap-1 whitespace-nowrap rounded-full border border-border bg-card px-2 text-sm font-medium transition-colors hover:bg-muted"
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
    <article className="group flex h-full min-w-0 flex-col rounded-2xl border border-border bg-card p-4 transition-colors duration-200 hover:border-[#007aff]/35">
      <div className="flex items-center justify-between gap-3">
        <span className="inline-flex items-center gap-2 text-xs font-semibold text-muted-foreground">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-sm font-semibold tabular-nums text-foreground">
            {rankLabel}
          </span>
          ジャンル内ランキング
        </span>
        <Badge
          variant="secondary"
          className="shrink-0 rounded-full bg-muted px-2.5 tabular-nums text-foreground/80"
        >
          {popularity}%
        </Badge>
      </div>

      <div className="mt-4 flex items-start gap-3">
        <Link
          to={detailPath}
          aria-label={`${app.name}の詳細`}
          className="flex size-14 shrink-0 items-center justify-center rounded-xl border border-border/70 bg-muted/40 transition-colors group-hover:bg-muted"
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
              className="line-clamp-1 text-base font-semibold leading-6 tracking-tight hover:text-[#007aff]"
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
          className="h-full rounded-full bg-[#007aff]"
          style={{
            width: `${Math.min(100, Math.max(0, recommendation.popularity))}%`,
          }}
        />
      </div>

      <div className="mt-3 flex min-w-0 items-center gap-1.5 overflow-hidden">
        <Badge
          variant="outline"
          className="shrink-0 rounded-full border-border text-xs"
        >
          {app.category}
        </Badge>
        {app.tags.slice(0, 1).map((tag) => (
          <Badge
            key={tag}
            variant="secondary"
            className="min-w-0 truncate rounded-full bg-muted text-xs"
          >
            {tag}
          </Badge>
        ))}
      </div>

      <div className="mt-auto flex items-center gap-2 pt-4">
        <Link
          to={detailPath}
          className="inline-flex h-9 min-w-0 flex-1 items-center justify-center gap-1 whitespace-nowrap rounded-full border border-border px-2 text-sm font-medium text-[#007aff] transition-colors hover:bg-muted"
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
