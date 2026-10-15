import { useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import { Search, Store, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import AppCard from "@/components/AppCard";
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
import { buildRecommendationFallbackApps } from "@/catalog/recommendation-fallback";
import { useAppState } from "@/lib/app-state";
import { useSearchParams } from "react-router";

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
        if (apps.length > 0) {
          setRecommendedCatalogApps(apps);
          if (missingIds.length > 0) {
            setRecommendationStatus("partial");
            setRecommendationMessage(
              `${missingIds.length}件のおすすめアプリは、このNixpkgs環境で詳細情報を解決できませんでした: ${missingIds.join(", ")}`,
            );
          } else {
            setRecommendationStatus("ready");
            setRecommendationMessage(null);
          }
          void refreshPackages(apps.map((app) => app.id));
          return;
        }

        const fallbackApps = buildRecommendationFallbackApps(ids);
        setRecommendedCatalogApps(fallbackApps);
        if (fallbackApps.length > 0) {
          setRecommendationStatus("fallback");
          setRecommendationMessage(
            "Nix検索は正常終了しましたが、おすすめIDと一致する詳細情報が0件でした。パッケージ属性名や現在のシステム向けの評価結果を確認してください。",
          );
          void refreshPackages(fallbackApps.map((app) => app.id));
        } else {
          setRecommendationStatus("unavailable");
          setRecommendationMessage(
            "取得したおすすめIDがフォールバック対象にも含まれていません。おすすめJSONとパッケージ対応表を確認してください。",
          );
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
    <div className="min-h-screen bg-background text-foreground">
      <div className="flex min-h-screen">
        <aside className="hidden w-64 shrink-0 border-r border-blue-100/80 bg-white/90 lg:block">
          <div className="sticky top-0 flex h-screen flex-col p-6">
            <div className="flex items-center gap-3">
              <div className="flex size-11 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 text-white">
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
                className="flex items-center gap-3 rounded-xl bg-blue-50 px-4 py-3 text-sm font-semibold text-blue-700"
              >
                <Search className="size-4" />
                アプリを探す
              </div>
            </nav>

            <div className="mt-auto rounded-2xl border border-blue-100 bg-gradient-to-br from-blue-50 to-indigo-50 p-4">
              <div className="flex items-center gap-2 text-sm font-semibold text-blue-900">
                <Sparkles className="size-4 text-blue-600" />
                Nixpkgs
              </div>
              <p className="mt-2 text-xs leading-5 text-blue-900/65">
                アプリ、言語、ライブラリ、開発ツールをまとめて検索できます。
              </p>
            </div>
          </div>
        </aside>

        <div className="min-w-0 flex-1">
          <header className="border-b border-blue-100/70 bg-white/90 backdrop-blur lg:hidden">
            <div className="flex h-16 items-center px-5">
              <div className="flex items-center gap-2 font-semibold">
                <Store className="size-5 text-blue-600" />
                nxc App Store
              </div>
            </div>
          </header>

          <main className="w-full px-5 py-6 sm:px-6 lg:px-8 lg:py-8">
            <section className="relative overflow-hidden rounded-2xl border border-blue-100 bg-gradient-to-br from-white via-blue-50/70 to-indigo-100/70 px-5 py-4 sm:px-6 sm:py-5">
              <div className="pointer-events-none absolute -right-12 -top-14 size-48 rounded-full bg-blue-400/15 blur-3xl" />
              <div className="relative">
                <div className="inline-flex items-center gap-2 rounded-full border border-blue-200 bg-white/75 px-3 py-1.5 text-sm font-semibold text-blue-700">
                  <Store className="size-4" />
                  NixOS App Store
                </div>
                <h1 className="mt-3 text-2xl font-bold tracking-tight sm:text-3xl lg:text-3xl">
                  アプリを探す
                </h1>
                <p className="mt-2 max-w-3xl text-sm leading-5 text-muted-foreground sm:text-base">
                  Nixpkgsにあるアプリ、言語、ライブラリ、開発ツールを検索できます。
                </p>

                <form onSubmit={handleSearch} className="mt-3">
                  <div className="flex w-full gap-2">
                    <div className="relative min-w-0 flex-1">
                      <Search className="absolute left-3.5 top-1/2 size-5 -translate-y-1/2 text-blue-500" />
                      <Input
                        className="h-10 rounded-lg border-blue-200 bg-white/95 pl-10 pr-3 text-sm ring-offset-background placeholder:text-muted-foreground/70 focus-visible:border-blue-400 focus-visible:ring-blue-200 sm:h-11 sm:text-base md:text-base!"
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
                      className="h-10 rounded-lg bg-blue-600 px-3 text-white hover:bg-blue-700 sm:h-11 sm:px-4"
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

            <section className="mt-8 lg:mt-10">
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
                  {recommendedApps.map((app) => (
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
                    />
                  ))}
                </DiscoverPreSearch>
              ) : (
                <>
                  <div className="mb-6 flex items-end justify-between gap-4">
                    <div>
                      <p className="text-sm font-semibold text-blue-600">
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
                    <div className="w-full rounded-2xl border border-dashed border-border bg-card/70 p-8 text-center">
                      <p className="font-medium">検索できませんでした。</p>
                      <p className="mt-2 text-sm text-muted-foreground">
                        {searchError}
                      </p>
                    </div>
                  ) : results.length === 0 && !searching ? (
                    <div className="w-full rounded-2xl border border-dashed border-border bg-card/70 p-8 text-center">
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
