import type { ReactNode } from "react";
import { Search } from "lucide-react";
import { Button } from "@/components/ui/button";

export type RecommendationStatus =
  | "loading"
  | "ready"
  | "partial"
  | "fallback"
  | "unavailable";

type DiscoverPreSearchProps = {
  query: string;
  hasRecommendations: boolean;
  generatedAt: string | null;
  loading: boolean;
  recommendationStatus: RecommendationStatus;
  recommendationMessage: string | null;
  onRetry: () => void;
  children: ReactNode;
};

/** Render the Discover page's pre-search guidance, loading state, or recommendations. */
export default function DiscoverPreSearch({
  query,
  hasRecommendations,
  generatedAt,
  loading,
  recommendationStatus,
  recommendationMessage,
  onRetry,
  children,
}: DiscoverPreSearchProps) {
  if (query.trim().length > 0) {
    return (
      <div className="flex min-h-[220px] w-full items-center justify-center rounded-3xl border border-dashed border-blue-200 bg-white/65 px-6 py-12 text-center">
        <div>
          <div className="mx-auto flex size-12 items-center justify-center rounded-2xl bg-blue-50 text-blue-600">
            <Search className="size-5" />
          </div>
          <h2 className="mt-5 text-xl font-semibold">検索してアプリを探す</h2>
          <p className="mx-auto mt-2 max-w-lg text-sm leading-6 text-muted-foreground">
            入力したキーワードで検索するには、Enter
            キーまたは検索ボタンを押してください。
          </p>
        </div>
      </div>
    );
  }

  if (!hasRecommendations && recommendationStatus === "loading") {
    return (
      <div
        aria-live="polite"
        className="flex min-h-[220px] w-full items-center justify-center rounded-3xl border border-dashed border-blue-200 bg-white/65 px-6 py-12 text-center"
      >
        <div>
          <div className="mx-auto flex size-12 items-center justify-center rounded-2xl bg-blue-50 text-blue-600">
            <Search className="size-5" />
          </div>
          <h2 className="mt-5 text-xl font-semibold">\n            人気のアプリを読み込み中\n          </h2>
          <p className="mx-auto mt-2 max-w-lg text-sm leading-6 text-muted-foreground">
            Nixpkgsからアプリ情報を取得しています。初回のパッケージ評価には時間がかかることがあります。
          </p>
        </div>
      </div>
    );
  }

  if (!hasRecommendations && recommendationStatus === "unavailable") {
    return (
      <div className="flex min-h-[260px] w-full items-center justify-center rounded-3xl border border-dashed border-blue-200 bg-white/65 px-6 py-12 text-center">
        <div className="max-w-2xl">
          <div className="mx-auto flex size-12 items-center justify-center rounded-2xl bg-blue-50 text-blue-600">
            <Search className="size-5" />
          </div>
          <h2 className="mt-5 text-xl font-semibold">\n            おすすめアプリを表示できませんでした\n          </h2>
          <p className="mx-auto mt-2 max-w-lg text-sm leading-6 text-muted-foreground">
            同梱のおすすめデータを読み込めなかったか、表示できるパッケージ情報がありません。
          </p>
          {recommendationMessage && (
            <p className="mx-auto mt-3 break-words text-sm text-muted-foreground" role="status">
              {recommendationMessage}
            </p>
          )}
          <Button className="mt-5" onClick={onRetry} type="button">
            再試行
          </Button>
        </div>
      </div>
    );
  }

  if (!hasRecommendations) {
    return (
      <div className="flex min-h-[260px] w-full items-center justify-center rounded-3xl border border-dashed border-blue-200 bg-white/65 px-6 py-12 text-center">
        <div>
          <div className="mx-auto flex size-12 items-center justify-center rounded-2xl bg-blue-50 text-blue-600">
            <Search className="size-5" />
          </div>
          <h2 className="mt-5 text-xl font-semibold">検索してアプリを探す</h2>
          <p className="mx-auto mt-2 max-w-lg text-sm leading-6 text-muted-foreground">
            検索結果は関連度順に表示されます。アプリだけでなく、開発ツールやライブラリも見つけられます。
          </p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="mb-6 flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-sm font-semibold text-blue-600">おすすめ</p>
          <h2 className="mt-1 text-xl font-semibold sm:text-2xl">
            人気のアプリ
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Arch Linux の公開パッケージ統計を参考にしています。
            {generatedAt
              ? ` 更新日: ${new Date(generatedAt).toLocaleDateString("ja-JP")}`
              : ""}
          </p>
        </div>
        {loading && (
          <span className="text-sm text-muted-foreground">
            インストール状態を確認中...
          </span>
        )}
      </div>

      {(recommendationStatus === "partial" ||
        recommendationStatus === "fallback") && (
        <div
          className="mb-5 rounded-xl border border-amber-200 bg-amber-50/70 px-4 py-3 text-sm text-amber-950"
          role="status"
        >
          <p className="font-medium">
            {recommendationStatus === "fallback"
              ? "Nixpkgsの詳細情報を取得できないため、同梱の基本情報でおすすめを表示しています。"
              : "一部のアプリ情報をNixpkgsから取得できませんでした。"}
          </p>
          {recommendationMessage && (
            <p className="mt-1 break-words text-amber-900">\n              {recommendationMessage}\n            </p>
          )}
          <Button
            className="mt-3"
            onClick={onRetry}
            size="sm"
            type="button"
            variant="outline"
          >
            再試行
          </Button>
        </div>
      )}

      <div className="max-w-5xl space-y-2">{children}</div>
    </>
  );
}
