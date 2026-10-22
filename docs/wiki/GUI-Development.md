# GUI 開発ガイド

NXC のデスクトップ版 App Store は React / TypeScript、Vite、Tailwind CSS、Tauri 2 を利用しています。現在は開発環境での起動・動作確認が中心で、本番用パッケージの配布は整備中です。

## 必要な環境

- Nix が利用可能な Linux / NixOS 環境
- Rust / Cargo
- Node.js（CI では Node.js 24 を利用）
- pnpm（CI では pnpm 11.9.0 を利用）
- Tauri 2 の Linux ビルド・実行に必要なライブラリ

開発に必要なシステムライブラリと環境変数は、リポジトリの Nix Flake 開発シェルで用意します。

## リポジトリを取得して開発シェルへ入る

```sh
git clone https://github.com/Ni-gihub/nixos-json-controller.git
cd nixos-json-controller
nix develop
```

Wayland では、WebKitGTK が正しい画面倍率を取得できるように GTK / GSettings の環境設定が必要です。通常のシェルから直接起動すると設定が不足することがあるため、リポジトリルートで `nix develop` を実行してください。

Nix で `nix-command` または `flakes` が無効だというエラーが出る場合は、ユーザー設定ファイル `~/.config/nix/nix.conf` に以下を設定してから、再度 `nix develop` を実行します。

```ini
experimental-features = nix-command flakes
```

## Frontend の依存関係をインストール

開発シェル内で実行します。

```sh
cd frontend
pnpm install --frozen-lockfile
```

## GUI を起動

```sh
pnpm tauri dev
```

Frontend の `tauri` スクリプトは、リポジトリの Nix Flake から AppStream データを用意し、そのパスを `NXC_APPSTREAM_DATA` として Tauri CLI に渡します。GUI の開発起動には Nix とリポジトリの Flake が必要です。

## Frontend のチェック

すべて `frontend` ディレクトリで実行します。

```sh
pnpm format:check
pnpm test
pnpm lint
pnpm build
```

| コマンド | 内容 |
| --- | --- |
| `pnpm format:check` | Frontend のフォーマットチェック |
| `pnpm test` | Frontend の自動テスト |
| `pnpm lint` | ESLint による静的チェック |
| `pnpm build` | TypeScript のチェックと Vite の本番用 Frontend ビルド |

Frontend のビルドに成功しても、Tauri の本番パッケージやデスクトップランチャー統合が完成したことにはなりません。

## 本番配布について

現時点では、次のコマンドを利用可能なインストール方法として扱わないでください。

```sh
nix profile install github:Ni-gihub/nixos-json-controller#nxc-app
```

GUI 用 Flake package、Desktop Entry、ランチャーアイコン、本番起動時の環境設定、profile の更新・削除手順は整備中です。状況は [Issue #101](https://github.com/Ni-gihub/nixos-json-controller/issues/101) と [#169–#171](https://github.com/Ni-gihub/nixos-json-controller/issues/169) を参照してください。
