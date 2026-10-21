import { useCallback, useEffect, useRef, useState } from "react";
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
import RecommendationAppCard from "@/components/RecommendationAppCard";
import VirtualizedAppList from "@/components/VirtualizedAppList";
import RecommendationCarousel from "@/components/RecommendationCarousel";
import DiscoverPreSearch, {
  type RecommendationStatus,
} from "./DiscoverPreSearch";
import { searchApps, type App } from "@/catalog";
import { clearSearchCache } from "@/catalog";
import {
  prepareCatalog,
  type CatalogPreparationStatus,
} from "@/catalog/preparation";
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
import {
  classifySearchUrlTransition,
  isCurrentSearchRequest,
} from "./discover-search-state";
import { useLocation, useNavigationType, useSearchParams } from "react-router";

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
  const location = useLocation();
  const navigationType = useNavigationType();
  const [searchParams, setSearchParams] = useSearchParams();
  const initialQuery = searchParams.get("query")?.trim() ?? "";
  const [query, setQuery] = useState(initialQuery);
  const [submittedQuery, setSubmittedQuery] = useState(initialQuery);
  const [results, setResults] = useState<App[]>([]);
  const [searching, setSearching] = useState(false);
  const [searchElapsedMs, setSearchElapsedMs] = useState<number | null>(null);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [nextOffset, setNextOffset] = useState(0);
  const [hasMoreResults, setHasMoreResults] = useState(false);
  const [resultsLimited, setResultsLimited] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [loadMoreError, setLoadMoreError] = useState<string | null>(null);
  const [catalogPreparing, setCatalogPreparing] = useState(true);
  const [catalogRefreshKey, setCatalogRefreshKey] = useState(0);
  const [catalogAttemptId, setCatalogAttemptId] = useState(0);
  const [catalogStatus, setCatalogStatus] =
    useState<CatalogPreparationStatus | null>(null);
  const [catalogError, setCatalogError] = useState<string | null>(null);
  const { states, loading, error, refreshPackages } = useAppState();
  const searchRequestIdRef = useRef(0);
  const loadMoreSentinelRef = useRef<HTMLDivElement | null>(null);
  const loadingMoreRef = useRef(false);
  const loadingMoreRequestIdRef = useRef<number | null>(null);
  const requestedStateIdsRef = useRef(new Set<string>());
  const pendingStateIdsRef = useRef(new Set<string>());
  const stateRefreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(
    null,
  );
  const stateRefreshInFlightRef = useRef(false);
  const pendingUrlQueryRef = useRef<string | null>(null);
  const lastHandledLocationKeyRef = useRef<string | null>(null);

  const resetQueuedPackageStates = useCallback(() => {
    requestedStateIdsRef.current.clear();
    pendingStateIdsRef.current.clear();
    if (stateRefreshTimerRef.current !== null) {
      clearTimeout(stateRefreshTimerRef.current);
      stateRefreshTimerRef.current = null;
    }
  }, []);

  const runSearch = useCallback(
    (searchQuery: string) => {
      const startedAt = performance.now();
      const requestId = ++searchRequestIdRef.current;
      resetQueuedPackageStates();
      setSearching(true);
      setSearchElapsedMs(null);
      setSearchError(null);
      setResults([]);
      setNextOffset(0);
      setHasMoreResults(false);
      setResultsLimited(false);
      setLoadingMore(false);
      setLoadMoreError(null);
      loadingMoreRef.current = false;
      loadingMoreRequestIdRef.current = null;

      void searchApps(searchQuery, 0, 40)
        .then((page) => {
          if (!isCurrentSearchRequest(requestId, searchRequestIdRef.current)) {
            return;
          }

          setResults(page.results);
          const elapsedMs = Math.round(performance.now() - startedAt);
          setSearchElapsedMs(elapsedMs);
          if (import.meta.env.DEV && import.meta.env.VITE_PERF_DEBUG === "1") {
            console.info("[nxc-perf] catalog_search", {
              elapsed_ms: elapsedMs,
              result_count: page.results.length,
            });
          }
          setNextOffset(page.nextOffset);
          setHasMoreResults(page.hasMore);
          setResultsLimited(page.resultsLimited);
        })
        .catch((err: unknown) => {
          if (!isCurrentSearchRequest(requestId, searchRequestIdRef.current)) {
            return;
          }

          setResults([]);
          setSearchError(String(err));
        })
        .finally(() => {
          if (isCurrentSearchRequest(requestId, searchRequestIdRef.current)) {
            setSearching(false);
          }
        });
    },
    [resetQueuedPackageStates],
  );

  // Status checks run only for rendered/near-visible cards. Briefly batch rows
  // entering the virtual window so scrolling does not launch one Nix evaluation
  // per card.
  /** Dispatch queued package status checks one batch at a time. */
  const dispatchQueuedPackageStates = useCallback(() => {
    const scheduleNextBatch = () => {
      if (
        pendingStateIdsRef.current.size === 0 ||
        stateRefreshTimerRef.current !== null ||
        stateRefreshInFlightRef.current
      ) {
        return;
      }

      stateRefreshTimerRef.current = setTimeout(() => {
        stateRefreshTimerRef.current = null;
        const batch = Array.from(pendingStateIdsRef.current).slice(0, 40);
        for (const packageName of batch) {
          pendingStateIdsRef.current.delete(packageName);
        }
        stateRefreshInFlightRef.current = true;
        void refreshPackages(batch).finally(() => {
          for (const packageName of batch) {
            // Preserve the marker when a newer viewport has already queued the same ID.
            if (!pendingStateIdsRef.current.has(packageName)) {
              requestedStateIdsRef.current.delete(packageName);
            }
          }
          stateRefreshInFlightRef.current = false;
          scheduleNextBatch();
        });
      }, 120);
    };

    scheduleNextBatch();
  }, [refreshPackages]);

  /** Queue state checks for packages in the current virtual viewport. */
  const requestVisiblePackageStates = useCallback(
    (packageNames: string[]) => {
      for (const packageName of packageNames) {
        if (
          states[packageName] !== undefined ||
          requestedStateIdsRef.current.has(packageName)
        ) {
          continue;
        }
        requestedStateIdsRef.current.add(packageName);
        pendingStateIdsRef.current.add(packageName);
      }
      dispatchQueuedPackageStates();
    },
    [dispatchQueuedPackageStates, states],
  );

  useEffect(
    () => () => {
      if (stateRefreshTimerRef.current !== null) {
        clearTimeout(stateRefreshTimerRef.current);
        stateRefreshTimerRef.current = null;
      }
      pendingStateIdsRef.current.clear();
    },
    [],
  );

  const loadMoreResults = useCallback(
    (retry = false) => {
      const searchQuery = submittedQuery.trim();
      if (
        !searchQuery ||
        searching ||
        !hasMoreResults ||
        loadingMoreRef.current ||
        (loadMoreError !== null && !retry)
      ) {
        return;
      }

      const requestId = searchRequestIdRef.current;
      const offset = nextOffset;
      loadingMoreRef.current = true;
      loadingMoreRequestIdRef.current = requestId;
      setLoadingMore(true);
      setLoadMoreError(null);

      void searchApps(searchQuery, offset, 40)
        .then((page) => {
          if (!isCurrentSearchRequest(requestId, searchRequestIdRef.current)) {
            return;
          }

          setResults((current) => {
            const knownIds = new Set(current.map((app) => app.id));
            return [
              ...current,
              ...page.results.filter((app) => {
                if (knownIds.has(app.id)) return false;
                knownIds.add(app.id);
                return true;
              }),
            ];
          });
          setNextOffset(page.nextOffset);
          setHasMoreResults(page.hasMore);
          setResultsLimited(page.resultsLimited);
        })
        .catch((err: unknown) => {
          if (isCurrentSearchRequest(requestId, searchRequestIdRef.current)) {
            setLoadMoreError(String(err));
          }
        })
        .finally(() => {
          if (loadingMoreRequestIdRef.current !== requestId) return;
          loadingMoreRequestIdRef.current = null;
          loadingMoreRef.current = false;
          if (isCurrentSearchRequest(requestId, searchRequestIdRef.current)) {
            setLoadingMore(false);
          }
        });
    },
    [hasMoreResults, loadMoreError, nextOffset, searching, submittedQuery],
  );

  useEffect(() => {
    const sentinel = loadMoreSentinelRef.current;
    if (
      !sentinel ||
      typeof IntersectionObserver === "undefined" ||
      !hasMoreResults ||
      searching ||
      loadingMore ||
      loadMoreError !== null ||
      searchError !== null
    ) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          loadMoreResults();
        }
      },
      { rootMargin: "600px 0px" },
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [
    hasMoreResults,
    loadMoreError,
    loadMoreResults,
    loadingMore,
    searchError,
    searching,
  ]);

  useEffect(() => {
    let cancelled = false;

    // Clear WebView pages before validating the source so results from a previous
    // catalog cannot survive a source change or failed refresh.
    clearSearchCache();
    searchRequestIdRef.current += 1;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- discard results from the previous source before async validation
    setResults([]);
    setSearchElapsedMs(null);
    setNextOffset(0);
    setHasMoreResults(false);
    setResultsLimited(false);
    setLoadingMore(false);
    setLoadMoreError(null);
    loadingMoreRef.current = false;
    loadingMoreRequestIdRef.current = null;
    setSearching(false);
    setSearchError(null);
    setCatalogPreparing(true);
    setCatalogError(null);

    void prepareCatalog(catalogRefreshKey > 0)
      .then((status) => {
        if (cancelled) return;
        setCatalogStatus(status);
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setCatalogStatus(null);
        setCatalogError(String(err));
      })
      .finally(() => {
        if (cancelled) return;
        setCatalogPreparing(false);
        setCatalogAttemptId((current) => current + 1);
      });

    return () => {
      cancelled = true;
    };
  }, [catalogRefreshKey]);

  const submittedQueryRef = useRef(submittedQuery);
  useEffect(() => {
    submittedQueryRef.current = submittedQuery;
  }, [submittedQuery]);

  useEffect(() => {
    const currentQuery = submittedQueryRef.current.trim();
    if (catalogPreparing || catalogAttemptId === 0 || !currentQuery) {
      return;
    }
    clearSearchCache();
    // eslint-disable-next-line react-hooks/set-state-in-effect -- restart the current query only after catalog verification finishes
    runSearch(currentQuery);
  }, [catalogAttemptId, catalogPreparing, runSearch]);

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
    const isInitialLocation = lastHandledLocationKeyRef.current === null;
    if (lastHandledLocationKeyRef.current === location.key) return;
    lastHandledLocationKeyRef.current = location.key;

    const transition = classifySearchUrlTransition(
      initialQuery,
      pendingUrlQueryRef.current,
      navigationType,
    );

    if (transition.kind === "stale") return;
    if (transition.kind === "internal") {
      pendingUrlQueryRef.current = null;
      return;
    }

    pendingUrlQueryRef.current = null;
    if (!isInitialLocation) {
      // Invalidate in-flight work before applying browser or external URL navigation.
      searchRequestIdRef.current += 1;
      setSearching(false);
      setResults([]);
      setNextOffset(0);
      setHasMoreResults(false);
      setResultsLimited(false);
      setLoadingMore(false);
      setLoadMoreError(null);
      loadingMoreRef.current = false;
      loadingMoreRequestIdRef.current = null;
      setSearchError(null);
      setQuery(transition.query);
      setSubmittedQuery(transition.query);
    }

    // Avoid starting a legacy search while the source-matched local catalog is still preparing.
    if (transition.query && !catalogPreparing && catalogAttemptId > 0) {
      runSearch(transition.query);
    }
  }, [
    catalogAttemptId,
    catalogPreparing,
    initialQuery,
    location.key,
    navigationType,
    runSearch,
  ]);

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

  const handleSearch = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalized = query.trim();
    if (!normalized) return;

    setSubmittedQuery(normalized);
    pendingUrlQueryRef.current = normalized;
    setSearchParams({ query: normalized });
    runSearch(normalized);
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
                            // Cancel any older request so it cannot repopulate cleared results.
                            searchRequestIdRef.current += 1;
                            resetQueuedPackageStates();
                            setSearching(false);
                            setSubmittedQuery("");
                            setResults([]);
                            setNextOffset(0);
                            setHasMoreResults(false);
                            setResultsLimited(false);
                            setLoadingMore(false);
                            setLoadMoreError(null);
                            loadingMoreRef.current = false;
                            loadingMoreRequestIdRef.current = null;
                            setSearchError(null);
                            pendingUrlQueryRef.current = "";
                            setSearchParams({});
                          }
                        }}
                        autoFocus
                      />
                    </div>
                    <Button
                      type="submit"
                      className="h-11 rounded-full bg-primary px-5 font-semibold text-primary-foreground shadow-sm shadow-blue-500/20 hover:bg-primary/90"
                      disabled={!query.trim() || searching || catalogPreparing}
                    >
                      <Search className="mr-2 size-4" />
                      検索
                    </Button>
                  </div>
                  <p className="mt-1 px-1 text-xs text-muted-foreground">
                    Enterキーで検索
                  </p>
                </form>

                {catalogPreparing && (
                  <p
                    className="mt-2 flex items-center gap-2 px-1 text-xs text-muted-foreground"
                    role="status"
                  >
                    <span
                      aria-hidden="true"
                      className="size-1.5 shrink-0 animate-pulse rounded-full bg-primary"
                    />
                    カタログを準備中です。完了すると検索できます。
                  </p>
                )}
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
                          states[featuredRecommendation.id]?.active === true
                        }
                        statusKnown={
                          states[featuredRecommendation.id]?.active === true ||
                          states[featuredRecommendation.id]?.active === false
                        }
                        statusLoading={
                          loading &&
                          states[featuredRecommendation.id] === undefined
                        }
                      />
                    )}

                    {recommendationGenreGroups.length > 0 && (
                      <div className="space-y-8">
                        {recommendationGenreGroups.map((group) => {
                          const visual = Object.hasOwn(
                            recommendationGenreVisuals,
                            group.genre,
                          )
                            ? recommendationGenreVisuals[group.genre]
                            : fallbackGenreVisual;
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
                                        states[app.id]?.active === true
                                      }
                                      statusKnown={
                                        states[app.id]?.active === true ||
                                        states[app.id]?.active === false
                                      }
                                      statusLoading={
                                        loading && states[app.id] === undefined
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
                          {results.length}件表示中
                          {searchElapsedMs !== null
                            ? " · 検索 " + searchElapsedMs + " ms"
                            : ""}
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
                  ) : results.length === 0 &&
                    !searching &&
                    !hasMoreResults ? (
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
                      {results.length > 0 ? (
                        <VirtualizedAppList
                          key={`${submittedQuery}:${searching ? "loading" : "ready"}`}
                          apps={results}
                          states={states}
                          loading={loading}
                          searchQuery={submittedQuery}
                          onVisiblePackages={requestVisiblePackageStates}
                        />
                      ) : (
                        <p
                          className="py-4 text-center text-sm text-muted-foreground"
                          role="status"
                        >
                          表示できる候補を確認しています...
                        </p>
                      )}
                      <div
                        ref={loadMoreSentinelRef}
                        className="h-px"
                        aria-hidden="true"
                      />
                      {loadingMore && (
                        <p
                          className="py-4 text-center text-sm text-muted-foreground"
                          role="status"
                        >
                          追加の検索結果を読み込み中...
                        </p>
                      )}
                      {loadMoreError && (
                        <div
                          className="mt-4 space-y-2 rounded-xl border border-dashed border-border p-4 text-sm"
                          role="alert"
                        >
                          <p>続きの取得に失敗しました。</p>
                          <p className="break-words text-muted-foreground">
                            {loadMoreError}
                          </p>
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={() => loadMoreResults(true)}
                          >
                            再試行
                          </Button>
                        </div>
                      )}
                      {!hasMoreResults && results.length > 0 && !searching && (
                        <p
                          className="py-4 text-center text-sm text-muted-foreground"
                          role="status"
                        >
                          {resultsLimited
                            ? "検索結果が多いため、先頭5,000件まで表示しました。"
                            : "すべての検索結果を表示しました。"}
                        </p>
                      )}
                    </div>
                  )}

                  {error && (
                    <p className="mt-5 text-sm text-muted-foreground">
                      一部のインストール状態を取得できませんでした。
                    </p>
                  )}
                </>
              )}
              <div className="mt-10 border-t border-border/60 pt-3">
                <div
                  className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2"
                  aria-live="polite"
                >
                  <div className="min-w-0 flex-1 space-y-0.5 text-xs text-muted-foreground">
                    {catalogPreparing ? (
                      <>
                        <p>Nixpkgsカタログを確認・準備中...</p>
                        <p className="break-words">
                          初回は全パッケージ情報を取得してローカルカタログを作成するため、時間がかかる場合があります。
                        </p>
                      </>
                    ) : catalogStatus ? (
                      <>
                        <p>
                          ローカルカタログ準備完了 ·{" "}
                          {catalogStatus.packageCount.toLocaleString("ja-JP")}{" "}
                          件
                        </p>
                        <p className="break-words">
                          {catalogStatus.action === "generated"
                            ? "カタログを生成"
                            : catalogStatus.action === "reused-memory"
                              ? "メモリ上のカタログを再利用"
                              : "検証済みキャッシュを再利用"}{" "}
                          ·{" " + catalogStatus.system + " · "}
                          {catalogStatus.sourceRevision
                            ? "rev " +
                              catalogStatus.sourceRevision.slice(0, 12) +
                              " · "
                            : ""}
                          {"照合 " + catalogStatus.sourceCheckMs + " ms · "}
                          {catalogStatus.action === "generated"
                            ? "生成 " + catalogStatus.generationMs + " ms"
                            : catalogStatus.action === "reused-memory"
                              ? "メモリ再利用"
                              : "読込 " + catalogStatus.cacheLoadMs + " ms"}
                          {" · " +
                            (catalogStatus.cacheBytes / (1024 * 1024)).toFixed(
                              1,
                            ) +
                            " MiB"}
                        </p>
                      </>
                    ) : (
                      <>
                        <p>ローカルカタログを利用できません</p>
                        <p className="break-words">
                          ソースを検証できないため、検索は従来のNix検索にフォールバックします。古いカタログは使用しません。
                        </p>
                        {catalogError && (
                          <p className="break-words">{catalogError}</p>
                        )}
                      </>
                    )}
                  </div>
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    className="ml-auto h-7 shrink-0 px-2 text-xs text-muted-foreground"
                    disabled={catalogPreparing}
                    onClick={() =>
                      setCatalogRefreshKey((current) => current + 1)
                    }
                  >
                    {catalogPreparing ? "準備中..." : "カタログを更新"}
                  </Button>
                </div>
              </div>
            </section>
          </main>
        </div>
      </div>
    </div>
  );
}

export default Discover;
