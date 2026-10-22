# NixOS JSON Controller (NXC)

NixOS JSON Controller（NXC）は、NixOS の設定変更を安全に扱うための CLI ツールと、Nixpkgs のパッケージを探してインストール操作を行うデスクトップ版 App Store を開発するプロジェクトです。

CLI はパッケージやサービスの変更を NixOS の宣言的な設定へ反映し、必要に応じてシステムの再ビルドまで実行します。GUI は React / TypeScript と Tauri 2 を利用しています。

## はじめに

- [CLI のインストール](Installation)：Nix profile を利用した CLI の導入方法
- [CLI の使い方](CLI-Usage)：パッケージ・サービスの操作と状態確認
- [安全設計と内部の流れ](Safety-and-Architecture)：設定変更・評価・検証について
- [GUI 開発環境](GUI-Development)：開発者向けの起動手順
- [トラブルシューティング](Troubleshooting)：エラー発生時の確認事項
- [開発への参加](Contributing)：テスト、Issue、PR の進め方

## 現在の状態

| 項目 | 状態 |
| --- | --- |
| CLI | Nix profile からインストールして利用可能 |
| CLI のパッケージ・サービス管理 | 対応済みのコマンドから操作可能。設定構造などによっては安全のため変更を拒否する |
| GUI | 開発環境で起動・動作確認が可能 |
| GUI の本番配布 | Nix profile からのインストール、デスクトップ統合、更新・削除手順を整備中 |

GUI の本番配布は完了していません。現在、次のような作業を Issue で管理しています。

- [#101 — App Store を Nix profile から管理する](https://github.com/Ni-gihub/nixos-json-controller/issues/101)
- [#106 — 本番配布前の CSP と Tauri 権限設定の監査](https://github.com/Ni-gihub/nixos-json-controller/issues/106)
- [#169 — GUI 用 Nix Flake package](https://github.com/Ni-gihub/nixos-json-controller/issues/169)
- [#170 — GUI のデスクトップ統合と本番起動環境](https://github.com/Ni-gihub/nixos-json-controller/issues/170)
- [#171 — GUI の利用手順と CI 成果物検証](https://github.com/Ni-gihub/nixos-json-controller/issues/171)

## NXC の主な機能

### CLI

- NixOS パッケージの追加・削除
- NixOS サービスの有効化・無効化
- NixOS Flake と configuration の探索
- 設定で宣言されたパッケージの一覧・検索
- パッケージやサービスの状態・宣言元の確認
- 変更前に実行計画を確認する dry-run

### GUI（開発中）

- Nixpkgs パッケージの検索
- 説明、バージョン、カテゴリなどの詳細表示
- 現在の NixOS におけるインストール状態の表示
- パッケージのインストール操作
- AppStream やローカルのアイコンテーマを利用したアイコン解決

## 安全に利用するために

NXC はユーザーの NixOS 設定を無条件に書き換えるものではありません。対象や設定構造を安全に判定できない場合には、変更を拒否することがあります。変更の前に対象を確認し、必要な場合は `--dry-run` を利用してください。

CLI のパッケージ操作は NixOS configuration の編集やシステムの再ビルドを伴う場合があります。実行内容を理解したうえで利用してください。

## 開発計画

- [プロジェクト憲章 #166](https://github.com/Ni-gihub/nixos-json-controller/issues/166)：目的・スコープ・成功基準
- [PBL #167](https://github.com/Ni-gihub/nixos-json-controller/issues/167)：優先順位付きバックログと完了履歴
- [Issue の書き方](../project-management/issue-writing.md)
- [プロジェクトとバックログの運用](../project-management/backlog-management.md)

CLI コマンドの詳細は [CLI Usage](CLI-Usage) を参照してください。
