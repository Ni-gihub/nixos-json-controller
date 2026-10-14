import type { ReactNode } from "react";
import { Search } from "lucide-react";

type DiscoverPreSearchProps = {
  query: string;
  hasRecommendations: boolean;
  generatedAt: string | null;
  loading: boolean;
  children: ReactNode;
};

/** Render the Discover page's pre-search guidance or its recommendation list. */
export default function DiscoverPreSearch({
  query,
  hasRecommendations,
  generatedAt,
  loading,
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
            入力したキーワードで検索するには、Enter キーまたは検索ボタンを押してください。
          </p>
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
          <h2 className="mt-1 text-xl font-semibold sm:text-2xl">人気のアプリ</h2>
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

      <div className="max-w-5xl space-y-2">{children}</div>
    </>
  );
}
