# トラブルシューティング

エラーが発生した場合は、まずコマンドの出力を保存し、どの段階で問題が発生したのかを確認してください。NixOS の設定変更を伴う操作では、原因を確認せずに同じ操作を繰り返さないでください。

## `nxc: command not found`

1. インストールが完了しているか確認します。

   ```sh
   nix profile list
   ```

2. シェルの PATH に profile の実行ファイルディレクトリが含まれているかを確認します。
3. profile element の名前が想定と異なる場合は、`nix profile list` の表示に従って更新・削除してください。

## Discovery で対象の Flake が見つからない、または設定が古い

Discovery を実行します。

```sh
nxc discover
```

NXC は対象の Flake と configuration を探索し、選択結果を `~/.config/nxc/discovery.json` に保存します。NixOS 設定の場所や Flake の構成を変更した後は、Discovery を再実行してください。

## Flakes または `nix-command` に関するエラー

Nix の設定で次が有効になっているか確認してください。

```ini
experimental-features = nix-command flakes
```

ユーザー設定ファイルの例は `~/.config/nix/nix.conf` です。組織やシステム全体で Nix の設定が管理されている場合は、その運用ルールに従ってください。

## 設定変更が安全性チェックで拒否される

NXC は変更対象を安全に特定できない場合、処理を拒否することがあります。

- エラーメッセージと対象に指定された設定ファイルを確認します。
- コメントや文字列と実際の Nix 構文を取り違えていないか、手元の設定構造を確認します。
- 実際の構成に対応していない場合は、関連 Issue を検索するか、新しい Issue で最小限の再現例と期待する動作を共有してください。

安全性チェックを回避する目的で検証を無効化しないでください。

## Nix 評価・再ビルドが失敗する

1. エラー出力から、Nix 評価・ビルド・切替・変更後検証のどの段階で失敗したか確認します。
2. 対象の NixOS configuration を確認します。
3. パッケージ・サービス操作の変更後検証に失敗した場合は、復元処理の結果も出力に含まれていないか確認します。復元エラーが報告されている場合は、元のエラーだけでなく復元エラーも確認してください。サービス操作も、`rebuild` が有効で変更または再ビルド要求がある場合は再ビルド後に状態を検証し、検証失敗時には設定の復元と復元後の再ビルドを試みます。`rebuild` が無効な実行では、再ビルド後の検証も、検証失敗を契機とする復元も行いません。

原因が不明な場合は、ログを保全し、設定変更を繰り返す前に状況を整理してください。

## GUI が起動しない、表示倍率がおかしい

GUI の開発起動は、まずリポジトリルートで開発シェルに入ってから行います。

```sh
nix develop
cd frontend
pnpm install --frozen-lockfile
pnpm tauri dev
```

WebKitGTK / GTK / GSettings の環境設定が不足すると、Wayland で表示倍率が異常になることがあります。システムの通常シェルから直接起動していないか確認してください。

## GUI を Nix profile からインストールできない

GUI の本番配布用 package とデスクトップ統合は、現在整備中です。開発用の `pnpm tauri dev` と、本番用アプリのインストールは別です。進捗は [Issue #101](https://github.com/Ni-gihub/nixos-json-controller/issues/101) を確認してください。

## 問題を報告する

Issue を作成する際は、実行したコマンド、エラー全文、NixOS / Nix の環境情報、再現手順、期待した動作を記載してください。パスワードや個人情報などの秘密情報は貼り付けないでください。

[Issue一覧を開く](https://github.com/Ni-gihub/nixos-json-controller/issues)
