import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  BookOpen,
  Briefcase,
  Code2,
  Film,
  FolderOpen,
  Gamepad2,
  Globe,
  MessageCircle,
  Network,
  Palette,
  Search,
  Settings,
  SlidersHorizontal,
  Store,
  Terminal,
  type LucideIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import AppCard from "@/components/AppCard";
import RecommendationAppCard from "@/components/RecommendationAppCard";
import RecommendationCarousel from "@/components/RecommendationCarousel";
import DiscoverPreSearch, {
  type RecommendationStatus,
} from "./DiscoverPreSearch";
import { searchApps, type App } from "@/catalog";
import {
  isRecommendationCacheFresh,
  loadBundledRecommendations,
  loadLatestRecommendations,
  readCachedRecommendations,
  retainUsableRecommendationData,
  writeCachedRecommendations,
  type RecommendationData,
} from "@/catalog/recommendation-data";
import { getRecommendedCatalogApps } from "@/catalog/recommendations";
import {
  buildRecommendationFallbackApps,
  formatUnresolvedRecommendationMessage,
} from "@/catalog/recommendation-fallback";
import { useAppState } from "@/lib/app-state";
import { useSearchParams } from "react-router";

const recommendationGenreVisuals: Record<
  string,
  { icon: LucideIcon; surface: string; foreground: string }
> = {
  ブラウザ: {
    icon: Globe,
    surface: "bg-sky-50 dark:bg-sky-950/40",
    foreground: "text-sky-700 dark:text-sky-300",
  },
  コミュニケーション: {
    icon: MessageCircle,
    surface: "bg-violet-50 dark:bg-violet-950/40",
    foreground: "text-violet-700 dark:text-violet-300",
  },
  クリエイティブ: {
    icon: Palette,
    surface: "bg-fuchsia-50 dark:bg-fuchsia-950/40",
    foreground: "text-fuchsia-700 dark:text-fuchsia-300",
  },
  メディア: {
    icon: Film,
    surface: "bg-rose-50 dark:bg-rose-950/40",
    foreground: "text-rose-700 dark:text-rose-300",
  },
  ネットワーク: {
    icon: Network,
    surface: "bg-cyan-50 dark:bg-cyan-950/40",
    foreground: "text-cyan-700 dark:text-cyan-300",
  },
  ゲーム: {
    icon: Gamepad2,
    surface: "bg-emerald-50 dark:bg-emerald-950/40",
    foreground: "text-emerald-700 dark:text-emerald-300",
  },
  開発ツール: {
    icon: Code2,
    surface: "bg-indigo-50 dark:bg-indigo-950/40",
    foreground: "text-indigo-700 dark:text-indigo-300",
  },
  ターミナル: {
    icon: Terminal,
    surface: "bg-slate-100 dark:bg-slate-900/60",
    foreground: "text-slate-700 dark:text-slate-300",
  },
  ユーティリティ: {
    icon: SlidersHorizontal,
    surface: "bg-amber-50 dark:bg-amber-950/40",
    foreground: "text-amber-700 dark:text-amber-300",
  },
  オフィス: {
    icon: Briefcase,
    surface: "bg-orange-50 dark:bg-orange-950/40",
    foreground: "text-orange-700 dark:text-orange-300",
  },
  システム管理: {
    icon: Settings,
    surface: "bg-lime-50 dark:bg-lime-950/40",
    foreground: "text-lime-700 dark:text-lime-300",
  },
  ファイル管理: {
    icon: FolderOpen,
    surface: "bg-teal-50 dark:bg-teal-950/40",
    foreground: "text-teal-700 dark:text-teal-300",
  },
  "ドキュメント・ノート": {
    icon: BookOpen,
    surface: "bg-blue-50 dark:bg-blue-950/40",
    foreground: "text-blue-700 dark:text-blue-300",
  },
};

const fallbackGenreVisual = {
  icon: FolderOpen,
  surface: "bg-secondary",
  foreground: "text-secondary-foreground",
};

/** Search the NixOS package catalog and display matching application cards. */
function Discover() {
  const [searchParams, setSearchParams] = useSearchParams();
  const initialQuery = searchParams.get("query")?.trim() ?? "";
  const [query, setQuery] = useState(initialQuery);
  const [submittedQuery, setSubmittedQuery] = useState(initialQuery);
  const [results, setResults] = useState<App[]>([]);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
  const { states, loading, error, refreshPackages } = useAppState();
  const [recommendationData, setRecommendationData] =
    useState<RecommendationData | null>(null);
  const recommendationDataRef = useRef<RecommendationData | null>(null);
  useEffect(() => {
    recommendationDataRef.current = recommendationData;
  }, [recommendationData]);
  const [recommendedCatalogApps, setRecommendedCatalogApps] = useState<App[]>(
    [],
  );
  const [recommendationStatus, setRecommendationStatus] =
    useState<RecommendationStatus>("loading");
  const [recommendationMessage, setRecommendationMessage] = useState<
    string | null
  >(null);
  const [recommendationReloadKey, setRecommendationReloadKey] = useState(0);
  const retryRecommendations = () =>
    setRecommendationReloadKey((current) => current + 1);
  const recommendationIds =
    recommendationData?.recommendations
      .map((recommendation) => recommendation.id)
      .join("\n") ?? "";
  const appById = new Map(recommendedCatalogApps.map((app) => [app.id, app]));
  const recommendedApps = (recommendationData?.recommendations ?? []).flatMap(
    (recommendation) => {
      const app = appById.get(recommendation.id);
      return app ? [{ ...app, recommendation }] : [];
    },
  );
  const featuredRecommendation = recommendedApps[0] ?? null;
  const recommendationGenreGroups = Array.from(
    new Set(recommendedApps.map((app) => app.recommendation.genre)),
  )
    .map((genre) => ({
      genre,
      apps: recommendedApps.filter((app) => app.recommendation.genre === genre),
    }))
    .map((group) => ({
      ...group,
      apps: group.apps.filter((app) => app.id !== featuredRecommendation?.id),
    }))
    .filter((group) => group.apps.length > 0);

  useEffect(() => {
    if (!initialQuery || initialQuery === submittedQuery) return;
    // Keep the controlled input synchronized with browser navigation.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- URL query is external navigation state
    setQuery(initialQuery);
    setSubmittedQuery(initialQuery);
  }, [initialQuery, submittedQuery]);

  useEffect(() => {
    let cancelled = false;
    const forceRefresh = recommendationReloadKey > 0;

    // A valid cache avoids reading either JSON file on ordinary screen refreshes.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- expose refresh state before async reads
    setRecommendationStatus("loading");
    setRecommendationMessage(null);

    void (async () => {
      const cached = readCachedRecommendations();
      let currentData: RecommendationData | undefined;

      if (cached) {
        currentData = cached.data;
        setRecommendationData(cached.data);

        if (!forceRefresh && isRecommendationCacheFresh(cached)) {
          setRecommendationStatus("ready");
          return;
        }
      } else {
        // Only read the bundled JSON when there is no persisted, valid recommendation cache.
        const bundled = await loadBundledRecommendations();
        if (cancelled) return;
        currentData = retainUsableRecommendationData(
          bundled,
          recommendationDataRef.current,
        );
        setRecommendationData(currentData);
      }

      // Stale cached data remains visible while the network is checked in the background.
      const latest = await loadLatestRecommendations();
      if (cancelled) return;

      if (latest) {
        currentData = latest;
        setRecommendationData(latest);
      }

      // Record the last check even on a network failure, so an offline app doesn't retry on
      // every visit. A successful online response replaces the cached content.
      if (currentData?.recommendations.length) {
        writeCachedRecommendations(currentData);
      } else {
        setRecommendationStatus("unavailable");
        setRecommendationMessage(
          "同梱データとオンラインのおすすめデータを読み込めませんでした。",
        );
      }
    })().catch((err: unknown) => {
      if (cancelled) return;
      setRecommendationStatus("unavailable");
      setRecommendationMessage(String(err));
    });

    return () => {
      cancelled = true;
    };
  }, [recommendationReloadKey]);

  useEffect(() => {
    const ids = recommendationIds.split("\n").filter(Boolean);
    if (ids.length === 0) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- clear derived results when recommendation data is unavailable
      setRecommendedCatalogApps([]);
      return;
    }

    let cancelled = false;
    // Nixpkgs evaluation may take time on a cold cache; avoid showing the empty state as success.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reflect the pending catalog request
    setRecommendationStatus("loading");
    setRecommendationMessage(null);

    void getRecommendedCatalogApps(ids)
      .then((apps) => {
        if (cancelled) return;

        const foundIds = new Set(apps.map((app) => app.id));
        const missingIds = ids.filter((id) => !foundIds.has(id));
        const fallbackApps = buildRecommendationFallbackApps(missingIds);
        const combinedApps = [
          ...apps,
          ...fallbackApps.filter((app) => !foundIds.has(app.id)),
        ];
        const combinedIds = new Set(combinedApps.map((app) => app.id));
        const unresolvedIds = ids.filter((id) => !combinedIds.has(id));
        setRecommendedCatalogApps(combinedApps);

        if (apps.length > 0) {
          if (fallbackApps.length > 0 || unresolvedIds.length > 0) {
            setRecommendationStatus("partial");
            const messages: string[] = [];
            if (fallbackApps.length > 0) {
              messages.push(
                `${fallbackApps.length}件は同梱の基本情報を使って表示しています。`,
              );
            }
            const unresolvedMessage =
              formatUnresolvedRecommendationMessage(unresolvedIds);
            if (unresolvedMessage) messages.push(unresolvedMessage);
            setRecommendationMessage(messages.join(" "));
          } else {
            setRecommendationStatus("ready");
            setRecommendationMessage(null);
          }
          void refreshPackages(combinedApps.map((app) => app.id));
          return;
        }

        const unresolvedMessage =
          formatUnresolvedRecommendationMessage(unresolvedIds);

        if (fallbackApps.length > 0) {
          setRecommendationStatus("fallback");
          const messages = [
            "Nix検索で詳細情報を取得できなかったため、同梱の基本情報でおすすめを表示しています。",
          ];
          if (unresolvedMessage) messages.push(unresolvedMessage);
          setRecommendationMessage(messages.join(" "));
          void refreshPackages(fallbackApps.map((app) => app.id));
        } else {
          setRecommendationStatus("unavailable");
          const messages = [
            "取得したおすすめIDがフォールバック対象にも含まれていません。おすすめJSONとパッケージ対応表を確認してください。",
          ];
          if (unresolvedMessage) messages.push(unresolvedMessage);
          setRecommendationMessage(messages.join(" "));
        }
      })
      .catch((err: unknown) => {
        if (cancelled) return;

        const fallbackApps = buildRecommendationFallbackApps(ids);
        setRecommendedCatalogApps(fallbackApps);
        if (fallbackApps.length > 0) {
          setRecommendationStatus("fallback");
          setRecommendationMessage(
            `Nixpkgsの詳細情報取得に失敗しました: ${String(err)}`,
          );
          void refreshPackages(fallbackApps.map((app) => app.id));
        } else {
          setRecommendationStatus("unavailable");
          setRecommendationMessage(
            `Nixpkgsの詳細情報取得に失敗し、表示用メタデータもありません: ${String(err)}`,
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [recommendationIds, refreshPackages, recommendationReloadKey]);

  useEffect(() => {
    if (
      !initialQuery ||
      initialQuery !== submittedQuery ||
      results.length > 0 ||
      searching
    )
      return;

    // Start the route-triggered lookup immediately so the UI reflects the pending request.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- required loading state for the async search
    setSearching(true);
    setSearchError(null);
    void searchApps(initialQuery)
      .then((nextResults) => {
        setResults(nextResults);
        void refreshPackages(nextResults.map((app) => app.id));
      })
      .catch((err) => {
        setResults([]);
        setSearchError(String(err));
      })
      .finally(() => setSearching(false));
  }, [
    initialQuery,
    refreshPackages,
    results.length,
    searching,
    submittedQuery,
  ]);

  const handleSearch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalized = query.trim();
    if (!normalized) return;
    setSubmittedQuery(normalized);
    setSearchParams({ query: normalized });
    setSearching(true);
    setSearchError(null);
    try {
      const nextResults = await searchApps(normalized);
      setResults(nextResults);
      void refreshPackages(nextResults.map((app) => app.id));
    } catch (err) {
      setResults([]);
      setSearchError(String(err));
    } finally {
      setSearching(false);
    }
  };

  const hasResults = submittedQuery.length > 0;

  return (
    <div className="min-h-[calc(100vh-2.5rem)] bg-background text-foreground">
      <div className="flex min-h-[calc(100vh-2.5rem)]">
        <aside className="hidden w-64 shrink-0 bg-sidebar lg:block">
          <div className="sticky top-10 flex h-[calc(100vh-2.5rem)] flex-col p-6">
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-xl bg-primary text-white shadow-sm shadow-blue-500/25">
                <Store className="size-5" />
              </div>
              <div>
                <p className="font-semibold tracking-tight">nxc App Store</p>
                <p className="text-xs text-muted-foreground">
                  NixOS向けアプリストア
                </p>
              </div>
            </div>

            <nav aria-label="App Store" className="mt-8 space-y-2">
              <div
                aria-current="page"
                className="flex items-center gap-3 rounded-lg bg-sidebar-accent px-3 py-2.5 text-sm font-semibold text-sidebar-accent-foreground transition-colors"
              >
                <Search className="size-4 text-primary" />
                アプリを探す
              </div>
            </nav>
          </div>
        </aside>

        <div className="min-w-0 flex-1">
          <main className="w-full px-5 py-6 sm:px-6 lg:px-8 lg:py-8">
            <section className="border-b border-border pb-6">
              <div>
                <p className="text-sm font-semibold text-primary">
                  Nixpkgsカタログ
                </p>
                <h1 className="mt-2 text-3xl font-semibold tracking-tight sm:text-4xl">
                  アプリを探す
                </h1>
                <p className="mt-2 max-w-3xl text-sm leading-6 text-muted-foreground sm:text-base">
                  Nixpkgsにあるアプリ、言語、ライブラリ、開発ツールを検索できます。
                </p>

                <form onSubmit={handleSearch} className="mt-5 max-w-4xl">
                  <div className="flex w-full gap-2">
                    <div className="relative min-w-0 flex-1">
                      <Search className="absolute left-3.5 top-1/2 size-5 -translate-y-1/2 text-primary" />
                      <Input
                        className="h-11 rounded-xl border-border bg-card pl-10 pr-3 text-sm ring-offset-background placeholder:text-muted-foreground/70 focus-visible:border-primary focus-visible:ring-primary/20 sm:text-base md:text-base!"
                        placeholder="アプリ、言語、ライブラリ、ツールを検索..."
                        value={query}
                        onChange={(event) => {
                          const value = event.target.value;
                          setQuery(value);
                          if (!value.trim()) {
                            setSubmittedQuery("");
                            setResults([]);
                            setSearchError(null);
                            setSearchParams({});
                          }
                        }}
                        autoFocus
                      />
                    </div>
                    <Button
                      type="submit"
                      className="h-11 rounded-full bg-primary px-5 font-semibold text-primary-foreground shadow-sm shadow-blue-500/20 hover:bg-primary/90"
                      disabled={!query.trim() || searching}
                    >
                      <Search className="mr-2 size-4" />
                      検索
                    </Button>
                  </div>
                  <p className="mt-1 px-1 text-xs text-muted-foreground">
                    Enterキーで検索
                  </p>
                </form>
              </div>
            </section>

            <section className="mt-6 lg:mt-8">
              {!hasResults ? (
                <DiscoverPreSearch
                  query={query}
                  hasRecommendations={recommendedApps.length > 0}
                  generatedAt={recommendationData?.generatedAt ?? null}
                  loading={loading}
                  recommendationStatus={recommendationStatus}
                  recommendationMessage={recommendationMessage}
                  onRetry={retryRecommendations}
                >
                  <div className="space-y-9">
                    {featuredRecommendation && (
                      <RecommendationAppCard
                        app={featuredRecommendation}
                        recommendation={featuredRecommendation.recommendation}
                        rank={1}
                        featured
                        installed={
                          states[featuredRecommendation.id]?.active ?? false
                        }
                      />
                    )}

                    {recommendationGenreGroups.length > 0 && (
                      <div className="space-y-8">
                        {recommendationGenreGroups.map((group) => {
                          const visual =
                            recommendationGenreVisuals[group.genre] ??
                            fallbackGenreVisual;
                          const GenreIcon = visual.icon;

                          return (
                            <section key={group.genre} className="space-y-3">
                              <div className="flex items-center justify-between gap-4">
                                <div className="flex min-w-0 items-center gap-3">
                                  <div
                                    aria-hidden="true"
                                    className={`flex size-11 shrink-0 items-center justify-center rounded-xl ring-1 ring-inset ring-black/5 dark:ring-white/10 ${visual.surface}`}
                                  >
                                    <GenreIcon
                                      className={`size-5 ${visual.foreground}`}
                                      strokeWidth={1.9}
                                    />
                                  </div>
                                  <div className="min-w-0">
                                    <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                                      <h3 className="text-lg font-bold tracking-tight sm:text-xl">
                                        {group.genre}
                                      </h3>
                                      <span
                                        className={`rounded-full px-2 py-0.5 text-xs font-semibold tabular-nums ${visual.surface} ${visual.foreground}`}
                                      >
                                        {group.apps.length}件
                                      </span>
                                    </div>
                                    <p className="mt-0.5 text-sm text-muted-foreground">
                                      人気順のランキング
                                    </p>
                                  </div>
                                </div>
                              </div>
                              <RecommendationCarousel
                                label={`${group.genre}のランキング`}
                                resetKey={group.apps
                                  .map((app) => app.id)
                                  .join("|")}
                              >
                                {group.apps.map((app, index) => (
                                  <div
                                    key={app.id}
                                    className="w-[280px] shrink-0 snap-start sm:w-[300px] xl:w-[320px]"
                                  >
                                    <RecommendationAppCard
                                      app={app}
                                      recommendation={app.recommendation}
                                      rank={
                                        index +
                                        (group.genre ===
                                        featuredRecommendation?.recommendation
                                          .genre
                                          ? 2
                                          : 1)
                                      }
                                      featured={false}
                                      installed={
                                        states[app.id]?.active ?? false
                                      }
                                    />
                                  </div>
                                ))}
                              </RecommendationCarousel>
                            </section>
                          );
                        })}
                      </div>
                    )}
                  </div>
                </DiscoverPreSearch>
              ) : (
                <>
                  <div className="mb-6 flex items-end justify-between gap-4">
                    <div>
                      <p className="text-sm font-semibold text-primary">
                        検索結果
                      </p>
                      <h2 className="mt-1 text-xl font-semibold sm:text-2xl">
                        「{submittedQuery}」
                      </h2>
                      {!searching && (
                        <p className="mt-1 text-sm text-muted-foreground">
                          {results.length}件
                        </p>
                      )}
                    </div>
                    {(searching || loading) && (
                      <span className="text-sm text-muted-foreground">
                        検索中...
                      </span>
                    )}
                  </div>

                  {searchError ? (
                    <div className="w-full rounded-xl border border-dashed border-border bg-card p-8 text-center">
                      <p className="font-medium">検索できませんでした。</p>
                      <p className="mt-2 text-sm text-muted-foreground">
                        {searchError}
                      </p>
                    </div>
                  ) : results.length === 0 && !searching ? (
                    <div className="w-full rounded-xl border border-dashed border-border bg-card p-8 text-center">
                      <p className="font-medium">
                        該当するパッケージが見つかりませんでした。
                      </p>
                      <p className="mt-2 text-sm text-muted-foreground">
                        別のキーワードで検索してみてください。
                      </p>
                    </div>
                  ) : (
                    <div className="max-w-5xl space-y-2">
                      {results.map((app) => (
                        <AppCard
                          key={app.id}
                          id={app.id}
                          name={app.name}
                          description={app.description}
                          category={app.category}
                          tags={app.tags}
                          installed={states[app.id]?.active ?? false}
                          icon={app.icon}
                          iconCandidates={app.iconCandidates}
                          searchQuery={submittedQuery}
                        />
                      ))}
                    </div>
                  )}

                  {error && (
                    <p className="mt-5 text-sm text-muted-foreground">
                      一部のインストール状態を取得できませんでした。
                    </p>
                  )}
                </>
              )}
            </section>
          </main>
        </div>
      </div>
    </div>
  );
}

export default Discover;
