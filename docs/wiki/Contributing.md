# 開発への参加

バグ報告、改善提案、ドキュメント修正、コード変更を歓迎します。実装に着手する前に、既存の Issue と現在のコードを確認し、同じ問題を扱う作業がないか確認してください。

## プロジェクトの方向性

NXC は、NixOS の設定を安全に扱う CLI と、そのコア機能を利用するデスクトップ版 App Store を開発しています。

重要な方針は次のとおりです。

- 設定変更の対象と実行計画を確認すること
- 安全に変更対象を判断できない場合は、無理に書き換えないこと
- CLI、コアロジック、GUI の責務を明確にすること
- 不要な依存関係や重複するキャッシュ層を増やさず、実装を軽量に保つこと
- ユーザーに見える振る舞いを変えた場合は、テストとドキュメントを更新すること

## 作業開始前に確認するもの

- [Issue一覧](https://github.com/Ni-gihub/nixos-json-controller/issues)
- [プロジェクト憲章 #166](https://github.com/Ni-gihub/nixos-json-controller/issues/166)
- [PBL #167](https://github.com/Ni-gihub/nixos-json-controller/issues/167)
- [Issue の書き方](https://github.com/Ni-gihub/nixos-json-controller/blob/main/docs/project-management/issue-writing.md)
- [プロジェクトとバックログの運用](https://github.com/Ni-gihub/nixos-json-controller/blob/main/docs/project-management/backlog-management.md)

既存 Issue に該当する作業がある場合は、その Issue のスコープと受け入れ条件を確認してください。内容が古い場合も、クローズ済みの履歴を理由なく再オープンせず、必要に応じて新しい Issue を作成し関連付けます。

## 開発環境

### Rust / CLI

リポジトリルートで開発環境を用意し、Rust の変更に対して関連テストを実行してください。

```sh
nix develop
cargo test -- --test-threads=1
```

変更に関係する既存テストを更新・追加し、失敗したテストがある場合は原因を確認してください。

### GUI / Frontend

[GUI Development](GUI-Development.md) の手順に従って Nix 開発シェルを起動し、`frontend` ディレクトリで確認します。

```sh
pnpm format:check
pnpm test
pnpm lint
pnpm build
```

## Issue の作成

Issue には、確認できている事実、困っていること、期待する結果、受け入れ条件を記載してください。原因や実装方法が未確定であれば、推測を確定事項として記述せず、調査が必要な点を明記してください。

Bug、PBI、Task、調査（SPIKE）など、目的に合う Issue Form を使ってください。優先度や関連 Issue が分かる場合は明示し、PBL と矛盾しないようにします。

## Pull Request

PR には次の情報を含めてください。

- 何を変更したか、なぜ必要なのか
- 関連する Issue
- 実行したテスト・Lint・Build と結果
- UI やユーザー操作が変わる場合は、変更前後の動作やスクリーンショット
- 未対応事項、既知の制約、残るリスク

セキュリティ境界、設定ファイルの書き込み、安全性判定、権限管理に関わる変更では、通常の動作だけでなく失敗時の振る舞いも確認してください。

## ドキュメント変更

コマンド、配布方法、開発手順、安全性の保証範囲が変わった場合は、README と Wiki 原稿の整合性を確認してください。GUI の本番配布が完了するまでは、開発用コマンドを本番インストール手順として記載しないでください。
