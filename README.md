# NixOS JSON Controller（NXC）

NXC は、NixOS の設定を安全に変更するための CLI コントローラーと、そのコア機能を利用するデスクトップ版 App Store を開発するプロジェクトです。

CLI ではパッケージの追加・削除やサービスの有効化・無効化を NixOS の宣言的な設定に反映し、必要に応じて nixos-rebuild switch まで実行します。GUI では Nixpkgs のパッケージを検索し、アプリの詳細やインストール状態を確認してインストール操作を行えます。

## 現在の状態

- **CLI：** Nix profile からインストールして利用できます。
- **GUI：** React / TypeScript と Tauri 2 で開発中です。開発環境での起動・動作確認ができます。
- **GUI の本番インストール：** Nix profile からの配布、ランチャー統合、更新・削除手順は未整備です。実装は Issue [#101](https://github.com/Ni-gihub/nixos-json-controller/issues/101) で管理しています。
- **既知の問題：** App Store の検索結果でアイコンが透明になる問題を調査中です。詳細は [Issue #96](https://github.com/Ni-gihub/nixos-json-controller/issues/96) を参照してください。

現在の Flake package は x86_64-linux を対象としています。

## 主な機能

### CLI

- NixOS パッケージの追加・削除
- NixOS サービスの有効化・無効化
- 登録された日本語名などの別名によるパッケージ・サービス指定
- NixOS 用 Flake の探索と選択結果の保存
- 現在の system generation、パッケージ、コマンド、サービス状態の表示
- NixOS 設定に宣言されたパッケージの一覧・検索
- パッケージやサービスの状態、宣言元、編集対象の説明
- 実行計画を確認する dry-run
- Nix 評価や rebuild 後の検証、および失敗時のロールバック処理

### GUI（開発中）

- Nixpkgs を利用したパッケージ検索
- アプリの説明・バージョン・カテゴリ・タグ・ホームページなどの詳細表示
- 現在の NixOS におけるインストール状態の表示
- 検索結果や詳細画面からのインストール操作
- sudo 認証が必要な場合のパスワード入力 UI
- AppStream やローカルのアイコンテーマを利用したアイコン解決

GUI はまだ本番用の Nix package として配布できる状態ではありません。通常利用向けのプロファイルインストール手順は、Issue #101 の完了後に追加します。

---

# CLI のインストール

## Nix profile からインストールする

NixOS 上で、リポジトリの default package をインストールします。

    nix profile install github:Ni-gihub/nixos-json-controller

インストールできたら、コマンドを確認します。

    nxc --help

NXC 自身を利用者の NixOS configuration の flake.nix に追加する必要はありません。ユーザーの Nix profile で管理できます。

### 更新

まず、インストール済みの profile element 名を確認します。

    nix profile list

表示された NXC の element 名を使って更新します。通常は次のコマンドを利用できます。

    nix profile upgrade nixos-json-controller

profile element 名が異なる場合は、nix profile list の結果に合わせて指定してください。

### 削除

profile 内の NXC の element 名を確認し、その element を削除します。

    nix profile list
    nix profile remove nixos-json-controller

削除対象の名前が異なる場合は、実際に表示された名前を指定してください。NXC を profile から削除しても、NixOS の設定に NXC 自身を登録しているわけではありません。

## ソースから CLI をビルドする

開発や調査のためにソースからビルドする場合は、Rust と Cargo が必要です。

    git clone https://github.com/Ni-gihub/nixos-json-controller.git
    cd nixos-json-controller
    cargo build --release

生成される CLI 実行ファイルは次の場所です。

    target/release/nixos-json-controller

ローカルで nxc という名前で利用したい場合は、PATH の通ったユーザー用ディレクトリなどへ配置してください。

    install -Dm755 target/release/nixos-json-controller ~/.local/bin/nxc

これはソースから実行ファイルを配置する方法です。Nix profile による管理や、GUI の本番インストールとは別の方法です。

---

# CLI の基本的な使い方

設定を変更する前に、NixOS configuration の Flake を探索します。

    nxc discover

Discovery が完了したら、パッケージやサービスの操作を行います。

## パッケージ

インストール：

    nxc i firefox

操作名を省略してもインストールできます。

    nxc firefox

削除：

    nxc r firefox

変更を行わず、実行予定の内容だけを確認する場合：

    nxc i firefox --dry-run

dry-run は設定変更と nixos-rebuild を実行せず、実行計画を確認するための機能です。実際の設定変更が必要な操作では、権限や対象の設定構造などに応じて処理が拒否されることがあります。

## サービス

サービスの有効化：

    nxc e openssh

無効化：

    nxc d openssh

辞書に登録された別名も利用できます。たとえば ssh や sshd は openssh として解決されます。

## 状態確認

NXC と現在の NixOS system の情報：

    nxc status

NixOS configuration で宣言されているパッケージ一覧：

    nxc list

宣言されているパッケージの検索：

    nxc search firefox

パッケージの状態・宣言元を確認：

    nxc explain package firefox

サービスの状態・宣言元を確認：

    nxc explain service openssh

すべてのコマンドとオプションは次で確認できます。

    nxc --help

---

# NixOS configuration の Discovery

NXC は対象となる NixOS Flake と configuration を探索し、選択結果を保存します。

    nxc discover

Discovery は単純に flake.nix の存在だけで判断するものではありません。候補となる Flake を調査し、Nix の評価によって nixosConfigurations などを確認して対象を決めます。

Discovery 結果は次の場所に保存されます。

    ~/.config/nxc/discovery.json

通常の操作では、この保存済み情報が使われます。flake.nix / flake.lock や環境が Discovery 時点から変わっているなど、保存情報が現在の環境と一致しない場合、NXC は古い情報を使ったまま変更を続けず、Discovery の再実行を要求します。

NixOS 設定の場所や構成を変更した場合は、再度 Discovery を実行してください。

    nxc discover

---

# CLI の一覧・検索について

## nxc list

nxc list は、評価された NixOS configuration の environment.systemPackages に宣言されているパッケージを一覧表示します。

これは、現在の system に存在するすべてのアプリやコマンドの一覧でも、Nixpkgs 全体の検索結果でもありません。

## nxc search

nxc search は、NixOS configuration に宣言されているパッケージ集合から検索します。

    nxc search firefox

検索結果では主に次の情報を確認できます。

- パッケージが NixOS 設定に宣言されていること
- 現在の system に存在するかどうか
- 宣言元の設定ファイル
- system-wide な実行ファイルの場所（該当する場合）

そのため、Nixpkgs のパッケージを自由に検索する GUI のカタログ検索とは役割が異なります。

---

# 安全性と設定変更の流れ

NXC はユーザーの NixOS 設定を無条件に書き換えることを目的としていません。対象、変更先、設定構造を確認し、可能な場合は NXC 専用 module へ変更を分離します。書き換え先を確実に判断できない場合などは、安全側に停止することがあります。

概念的な処理の流れは次のとおりです。

    入力
      ↓
    入力検証
      ↓
    パッケージ・サービス名の解決
      ↓
    実行計画の作成
      ↓
    設定変更
      ↓
    Nix 評価
      ↓
    nixos-rebuild switch
      ↓
    結果の検証

Nix 評価や rebuild 後の検証に失敗した場合は、変更前の状態へ戻すロールバック処理を行います。ロールバックの結果は、実際のエラー出力も含めて確認してください。

NixOS の宣言的な設定変更と rebuild を伴うため、変更操作では必要に応じて sudo 認証が求められます。

---

# GUI の開発環境

GUI は React / TypeScript、Vite、Tailwind CSS、Tauri 2 を利用しています。現在の GUI は開発環境での実行が中心です。

## 必要なもの

- Nix が利用できる NixOS / Linux 環境
- Rust / Cargo
- Node.js（CI では Node.js 24 を利用）
- pnpm（CI では pnpm 11.9.0 を利用）
- Tauri 2 の Linux ビルド・実行に必要なライブラリ

## 起動する

リポジトリを取得し、Frontend のディレクトリで依存関係をインストールします。

    git clone https://github.com/Ni-gihub/nixos-json-controller.git
    cd nixos-json-controller/frontend
    pnpm install --frozen-lockfile

その後、リポジトリの Flake を使って AppStream データを用意し、Tauri の開発環境を起動します。

    pnpm tauri dev

frontend/package.json の tauri スクリプトは、リポジトリルートの Nix Flake から appstream-data 出力をビルドし、そのパスを NXC_APPSTREAM_DATA として渡して Tauri CLI を実行します。したがって、GUI の開発起動にはリポジトリの Nix Flake と Nix が必要です。

## Frontend の確認コマンド

frontend ディレクトリで実行します。

    pnpm test
    pnpm lint
    pnpm build

- pnpm test：アイコンの候補選択やフォールバックに関するテスト
- pnpm lint：ESLint による静的チェック
- pnpm build：TypeScript のビルドチェックと Vite の本番用 Frontend ビルド

これらは開発者向けコマンドです。pnpm build が成功することだけでは、Tauri の本番パッケージや Niri のランチャー統合が完成したことにはなりません。

## GUI の本番インストールについて

現時点では、次のコマンドはまだ利用できるものとして扱わないでください。

    nix profile install github:Ni-gihub/nixos-json-controller#nxc-app

GUI 用の Flake package、Desktop Entry、静的なランチャーアイコン、本番起動時の環境設定、profile の更新・削除手順は整備中です。これらを実装して実機で検証する作業は [Issue #101](https://github.com/Ni-gihub/nixos-json-controller/issues/101) で管理しています。

本番対応が完了したら、GUI 専用パッケージとして CLI と独立してインストール・更新・削除できる形にする予定です。

---

# 主なコマンド一覧

| コマンド | 説明 |
| --- | --- |
| nxc <package> | パッケージをインストール |
| nxc i <package> | パッケージをインストール |
| nxc r <package> | パッケージを削除 |
| nxc e <service> | サービスを有効化 |
| nxc d <service> | サービスを無効化 |
| nxc discover | NixOS Flake を探索 |
| nxc list | 設定に宣言された system packages を一覧表示 |
| nxc search <query> | 宣言された system packages を検索 |
| nxc status | 現在の system / Discovery 状態を表示 |
| nxc explain package <name> | パッケージの状態・宣言元を表示 |
| nxc explain service <name> | サービスの状態・宣言元を表示 |
| nxc --help | ヘルプを表示 |

設定変更コマンドでは dry-run オプションを使えます。参照・表示用コマンドに dry-run を指定するとエラーになります。

---

# 関連 Issue

- [#43 — 検索後のインストール状態確認の遅延を調査・改善する](https://github.com/Ni-gihub/nixos-json-controller/issues/43)
- [#88 — デスクトップ版 App Store の文字を読みやすくする](https://github.com/Ni-gihub/nixos-json-controller/issues/88)
- [#96 — アプリのアイコンが透明になる問題を根本原因から調査する](https://github.com/Ni-gihub/nixos-json-controller/issues/96)
- [#101 — nxc App Store を Nix profile から本番インストールできるようにする](https://github.com/Ni-gihub/nixos-json-controller/issues/101)

---

# ライセンス

現在、リポジトリには利用者向けライセンスがまだ設定されていません。再配布や利用条件を明確にする場合は、ライセンスを別途設定する必要があります。
