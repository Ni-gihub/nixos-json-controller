# NXC Wiki 原稿と自動公開

このディレクトリの Markdown ファイルを、GitHub Wiki の管理元として使用します。ページ本文は通常の PR でレビューし、`main` にマージされた後、GitHub Actions が Wiki へ自動で同期します。

## ページ一覧

- [Home](Home.md)：プロジェクト概要とナビゲーション
- [Installation](Installation.md)：CLI のインストール・更新・削除
- [CLI Usage](CLI-Usage.md)：コマンドリファレンス
- [Safety and Architecture](Safety-and-Architecture.md)：設定変更の流れと安全設計
- [GUI Development](GUI-Development.md)：App Store GUI の開発環境
- [Troubleshooting](Troubleshooting.md)：よくある問題の確認方法
- [Contributing](Contributing.md)：開発参加と Issue / PR 運用

この `README.md` は原稿管理・公開手順用であり、Wiki ページには同期しません。

## 初回のみ必要な設定

1. リポジトリの **Settings → Features → Wikis** で Wiki が有効であることを確認します。
2. [Wiki](https://github.com/Ni-gihub/nixos-json-controller/wiki) を開き、まだ初期化されていない場合は `Create the first page` からページを1つ作成して保存します。ダミーの本文で構いません。この操作は Wiki 用 Git リポジトリを初期化するために必要です。
3. リポジトリの **Settings → Actions → General** を開き、Workflow permissions の設定で Actions がワークフローに宣言された `contents: write` 権限を使用できることを確認します。

同期ワークフローは、このリポジトリ自身の Wiki に `github.token` で書き込みます。別のリポジトリ用のトークンや追加の Secret は必要ありません。

## 自動公開の動作

ワークフローは [`.github/workflows/publish-wiki.yml`](../../.github/workflows/publish-wiki.yml) で管理しています。

- `docs/wiki/` 以下の変更が `main` に push されたときに実行されます。通常は、そのファイルを含む PR がマージされたタイミングです。
- `main` にある変更内容を取得し、Wiki のルートに各ページをコピーしてコミット・push します。
- 変更がない場合は、不要なコミットを作成しません。
- `workflow_dispatch` による手動実行にも対応しています。初回設定後や再同期が必要なときは **Actions → Publish GitHub Wiki → Run workflow** を使います。

PR が作成・更新されただけでは Wiki に公開しません。未レビューの変更が一般公開されないよう、`main` へのマージ後に同期します。

## 手動で同期する場合

通常は GitHub Actions が自動同期するため、この手順は初回確認やワークフローの障害時にだけ使用します。リポジトリのルートで実行してください。Wiki を初期化してから使用します。

```sh
git clone https://github.com/Ni-gihub/nixos-json-controller.wiki.git ../nixos-json-controller.wiki

for page in Home Installation CLI-Usage Safety-and-Architecture GUI-Development Troubleshooting Contributing; do
  cp "docs/wiki/$page.md" "../nixos-json-controller.wiki/$page.md"
  # Repository Markdown uses .md links; GitHub Wiki page links omit the suffix.
  perl -pi -e 's/\]\(([A-Za-z0-9_-]+)\.md\)/]($1)/g' "../nixos-json-controller.wiki/$page.md"
done

git -C ../nixos-json-controller.wiki add -- Home.md Installation.md CLI-Usage.md Safety-and-Architecture.md GUI-Development.md Troubleshooting.md Contributing.md
git -C ../nixos-json-controller.wiki commit -m "docs: sync NXC Wiki pages"
git -C ../nixos-json-controller.wiki push origin HEAD
```

If the Wiki clone directory already exists, update it with `git -C ../nixos-json-controller.wiki pull --ff-only` instead of cloning again. Review the result of `git status` before committing.

## ページの更新方法

1. `docs/wiki/` の該当 Markdown を編集します。
2. ブランチで変更し、通常どおり PR を作成します。
3. PR が `main` にマージされたら、Actions の `Publish GitHub Wiki` ワークフローが自動で Wiki に同期します。
4. [公開 Wiki](https://github.com/Ni-gihub/nixos-json-controller/wiki) とワークフローの実行結果を確認します。

管理対象の7ページは、このディレクトリの内容が正となります。GitHub Wiki の画面から管理対象ページを直接編集しても、次回同期時に原稿の内容で上書きされる場合があります。修正は原則としてこのディレクトリに対する PR で行ってください。

ワークフローは現在の7ページを同期し、Wiki のその他のページは削除しません。ページ名を変更・削除するときは、旧ページが残らないよう同期処理も更新してください。

## 更新時の注意

- CLI のコマンドや配布方法を変更した場合は、README とこの Wiki 原稿をあわせて見直してください。
- GUI は開発環境で起動できる状態ですが、本番用 Nix package としての配布・更新・削除手順は整備中です。完成するまではインストール可能な機能として案内しないでください。
- 対応状況はコードと Issue を確認し、完了済み・開発中・未対応を区別して記載してください。
