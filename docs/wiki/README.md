# NXC Wiki ページ

このディレクトリには、GitHub Wiki に掲載するための Markdown ページをまとめています。Wiki の内容をレビューしやすくし、リポジトリ内で履歴管理するための原稿です。

## ページ一覧

- [Home](Home.md)：プロジェクト概要とナビゲーション
- [Installation](Installation.md)：CLI のインストール・更新・削除
- [CLI Usage](CLI-Usage.md)：コマンドリファレンス
- [Safety and Architecture](Safety-and-Architecture.md)：設定変更の流れと安全設計
- [GUI Development](GUI-Development.md)：App Store GUI の開発環境
- [Troubleshooting](Troubleshooting.md)：よくある問題の確認方法
- [Contributing](Contributing.md)：開発参加と Issue / PR 運用

## GitHub Wiki への公開

これらのファイルはリポジトリの `docs/wiki/` 以下にあるため、通常の Wiki ページとして自動公開されるわけではありません。

1. リポジトリの **Settings → Features → Wikis** で Wiki が有効になっていることを確認します。
2. Wiki ページをまだ作成していない場合は、GitHub の Wiki 画面から最初のページを作成・保存します。これにより Wiki 用リポジトリが初期化されます。
3. Wiki の Git リポジトリにこれらの Markdown ファイルをコピーし、Wiki のルートへ配置します。
4. `Home.md` を Home ページとして使用します。`README.md` は移行手順のためのファイルなので、Wiki のページとしては必須ではありません。
5. Wiki 内の相対リンクと各ページを確認してから公開します。

Wiki 専用リポジトリが初期化されていない場合、この原稿だけでは `/wiki` の公開ページ自体は作成されません。

## 更新時の注意

- CLI のコマンドや配布方法を変更した場合は、README とこの Wiki 原稿をあわせて見直してください。
- GUI は開発環境で起動できる状態ですが、本番用 Nix package としての配布・更新・削除手順は整備中です。完成するまではインストール可能な機能として案内しないでください。
- 対応状況はコードと Issue を確認し、完了済み・開発中・未対応を区別して記載してください。
