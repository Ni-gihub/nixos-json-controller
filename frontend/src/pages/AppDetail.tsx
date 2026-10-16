import { useEffect, useState } from "react";
import { ArrowLeft, ExternalLink, Package, Tag } from "lucide-react";
import { Link, useParams, useSearchParams } from "react-router";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import InstallButton from "@/components/InstallButton";
import AppIcon from "@/components/AppIcon";
import { findApp, type App } from "@/catalog";
import { useAppState } from "@/lib/app-state";

/** Loads and displays details for the selected catalog package. */
function AppDetail() {
  const { id } = useParams();
  const [searchParams] = useSearchParams();
  const returnQuery = searchParams.get("query")?.trim() ?? "";
  const { states, refreshPackage } = useAppState();
  const [app, setApp] = useState<App | undefined>();
  const [loadingApp, setLoadingApp] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const appState = id ? states[id] : undefined;
  const installed = appState?.active === true;
  const statusKnown = appState !== undefined;

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    // Clear stale route data before the async lookup for the new package starts.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- route changes intentionally reset the loading state
    setLoadingApp(true);
    setApp(undefined);
    setError(null);
    void findApp(id, returnQuery)
      .then((result) => {
        if (!cancelled) setApp(result);
      })
      .catch((err) => {
        if (!cancelled) setError(String(err));
      })
      .finally(() => {
        if (!cancelled) setLoadingApp(false);
      });
    return () => {
      cancelled = true;
    };
  }, [id, returnQuery]);

  useEffect(() => {
    if (!id) return;

    // Initial status lookup is non-fatal; InstallButton reports errors for user-initiated actions.
    void refreshPackage(id).catch(() => undefined);
  }, [id, refreshPackage]);

  if (loadingApp) {
    return (
      <div className="flex min-h-[calc(100vh-2.5rem)] items-center justify-center bg-muted/20 text-muted-foreground">
        パッケージ情報を取得中...
      </div>
    );
  }

  if (!app || !id) {
    return (
      <div className="flex min-h-[calc(100vh-2.5rem)] items-center justify-center bg-muted/20">
        <div className="text-center">
          <h1 className="text-2xl font-bold">パッケージが見つかりません</h1>
          {error && (
            <p className="mt-2 max-w-md text-sm text-muted-foreground">
              {error}
            </p>
          )}
          <Link
            to={
              returnQuery
                ? `/discover?query=${encodeURIComponent(returnQuery)}`
                : "/discover"
            }
            className="mt-4 inline-block text-primary hover:underline"
          >
            パッケージを探す
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-[calc(100vh-2.5rem)] bg-background text-foreground">
      <header className="sticky top-10 z-30 border-b border-primary/10 bg-background/95 backdrop-blur">
        <div className="flex h-14 items-center px-5 sm:px-6">
          <Link
            to={
              returnQuery
                ? `/discover?query=${encodeURIComponent(returnQuery)}`
                : "/discover"
            }
            className="inline-flex items-center gap-2 rounded-lg px-2 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-primary/10 hover:text-primary"
          >
            <ArrowLeft className="size-4" />
            アプリを探す
          </Link>
          <div className="ml-4 border-l border-primary/15 pl-4 text-sm font-semibold">
            nxc App Store
          </div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-none px-5 py-6 sm:px-6 lg:px-8 lg:py-8">
        <div className="mx-auto max-w-5xl">
          <section className="min-w-0 space-y-6">
            <Card className="overflow-hidden rounded-2xl border-primary/15 bg-card shadow-md shadow-blue-950/5">
              <CardContent className="relative p-5 sm:p-6 lg:p-8">
                <div className="relative flex flex-col items-start gap-5 sm:flex-row">
                  <div className="flex size-24 shrink-0 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-4xl font-bold text-primary shadow-sm">
                    <AppIcon
                      name={app.name}
                      packageId={app.id}
                      icon={app.icon}
                      iconCandidates={app.iconCandidates}
                      imageClassName="size-18 object-contain"
                      fallbackClassName="size-12 text-foreground/70"
                    />
                  </div>
                  <div className="min-w-0 flex-1 pt-1">
                    <div className="flex flex-wrap items-start justify-between gap-5">
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm">
                          <span className="font-medium text-primary">
                            {app.category}
                          </span>
                          {app.tags.map((tag) => (
                            <span key={tag} className="text-muted-foreground">
                              {tag}
                            </span>
                          ))}
                        </div>
                      </div>
                      <InstallButton
                        packageId={id}
                        appName={app.name}
                        installed={installed}
                        statusKnown={statusKnown}
                        onInstalled={() => refreshPackage(id)}
                        className="h-10 shrink-0 rounded-full bg-primary px-4 text-sm text-white hover:bg-primary/90"
                        size="sm"
                      />
                    </div>
                    <h1 className="mt-3 text-3xl font-semibold tracking-tight sm:text-4xl">
                      {app.name}
                    </h1>
                    {app.version && (
                      <p className="mt-2 text-sm text-muted-foreground">
                        バージョン {app.version}
                      </p>
                    )}
                    <p className="mt-4 max-w-3xl text-base leading-6 text-muted-foreground sm:text-base lg:text-lg">
                      {app.description}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="rounded-2xl border-primary/15 bg-card shadow-sm">
              <CardContent className="p-5 sm:p-6 lg:p-8">
                <h2 className="text-lg font-semibold tracking-tight">
                  説明・詳細情報
                </h2>
                <div className="mt-5 grid gap-5 md:grid-cols-2">
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Package className="size-4 text-primary" />
                      パッケージ
                    </div>
                    <p className="break-all rounded-xl border border-primary/10 bg-primary/5 px-4 py-3 font-mono text-sm">
                      {app.id}
                    </p>
                  </div>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Tag className="size-4 text-primary" />
                      カテゴリ
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="secondary">{app.category}</Badge>
                      {app.tags.map((tag) => (
                        <Badge key={tag} variant="outline">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </div>
                {app.homepage && (
                  <div className="mt-6 border-t border-border pt-5">
                    <p className="text-sm font-medium">Homepage</p>
                    <a
                      href={app.homepage}
                      target="_blank"
                      rel="noreferrer"
                      className="mt-2 inline-flex max-w-full items-center gap-2 break-all text-sm text-primary hover:underline"
                    >
                      {app.homepage}
                      <ExternalLink className="size-4 shrink-0" />
                    </a>
                  </div>
                )}
              </CardContent>
            </Card>
          </section>
        </div>
      </main>
    </div>
  );
}

export default AppDetail;
