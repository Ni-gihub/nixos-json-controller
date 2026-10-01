# NixOS-JSON-Controller

NixOSの設定変更をCLIから安全に実行するためのコントローラーです。

パッケージの追加・削除、サービスの有効化・無効化などを、NixOSの設定を変更して `nixos-rebuild switch` まで実行します。

## Features

- NixOSパッケージの追加・削除
- NixOSサービスの有効化・無効化
- 日本語などの別名に対応
- NixOS設定用Flakeの自動Discovery
- 現在のsystem / Discovery状態の確認
- パッケージ・サービスの状態や定義元の確認
- 既存設定を安全に編集できない場合の専用module fallback
- 設定変更後のNix評価・rebuildによる検証
- rebuild失敗時の変更ロールバック
- `--dry-run` による実行内容の確認

---

# Installation

## NixOS / Nix flakeからインストール

NixOSで利用する場合はこちらを推奨します。

```bash
nix profile install github:Ni-gihub/nixos-json-controller
```

インストール後、

```bash
nxc --help
```

で確認できます。

現在のFlake packageは `x86_64-linux` を対象としています。

## ソースからビルド

Rust / Cargoを使用する場合は、リポジトリを取得してビルドできます。

```bash
git clone https://github.com/Ni-gihub/nixos-json-controller.git
cd nixos-json-controller
cargo build --release
```

生成された実行ファイルは、

```text
target/release/nixos-json-controller
```

です。

`nxc` として使用する場合は、PATHの通った場所へ配置してください。

```bash
install -Dm755 target/release/nixos-json-controller ~/.local/bin/nxc
```

---

# Quick Start

まずNixOS設定をDiscoveryします。

```bash
nxc discover
```

Discoveryに成功したら、通常の操作を実行できます。

```bash
nxc i firefox
nxc r firefox
nxc e openssh
nxc d openssh
```

現在の状態を確認する場合:

```bash
nxc status
```

インストール済みのsystem-wideなコマンドやアプリを一覧表示する場合:

```bash
nxc list
```

---

# Discovery

NXCは、NixOS設定用のFlakeを自動的に探索します。

```bash
nxc discover
```

Discoveryでは単純に `flake.nix` の存在だけを見るのではなく、候補となったFlakeを検査し、Nix評価によって `nixosConfigurations` を確認します。

```text
候補探索
  ↓
Filesystem Inspection
  ↓
Nix Evaluation
  ↓
nixosConfigurations確認
  ↓
Configuration選択
  ↓
Discovery結果を保存
```

選択されたFlakeとConfigurationは保存され、通常の操作では保存済みのDiscovery結果が使用されます。

保存先:

```text
~/.config/nxc/discovery.json
```

そのため、通常のパッケージ・サービス操作のたびにFlake全体を探索する必要はありません。

## Discoveryをやり直す

NixOS設定の場所や構成を変更した場合は、再度実行してください。

```bash
nxc discover
```

保存済みのDiscovery結果が現在の環境と一致しない場合、NXCは古い情報を使ったまま設定を変更せず、Discoveryの再実行を要求します。

---

# Package

## インストール

```bash
nxc i firefox
```

操作を省略して、

```bash
nxc firefox
```

と書くこともできます。

## 削除

```bash
nxc r firefox
```

## Dry Run

実際に設定を変更せず、実行内容を確認できます。

```bash
nxc i firefox --dry-run
```

---

# Service

## 有効化

```bash
nxc e openssh
```

別名も使用できます。

```bash
nxc e ssh
nxc e sshd
```

## 無効化

```bash
nxc d openssh
```

---

# Japanese aliases

登録されているパッケージ・サービスには別名を使用できます。

例えばFirefoxの場合:

```bash
nxc i ファイアフォックス
nxc i ファイヤーフォックス
nxc i 火狐
```

これらはすべて `firefox` として扱われます。

サービスも同様です。

```bash
nxc e ssh
nxc e sshd
```

は `openssh` として解決されます。

辞書に登録されていない名前は、勝手に別のパッケージやサービスとして解釈されません。

---

# System

## Status

現在のNXCとNixOS systemの状態を確認できます。

```bash
nxc status
```

Discovery済みの場合は、使用中のFlakeとConfigurationも確認できます。

## Installed applications

現在のsystem-wideなコマンドやアプリを一覧表示します。

```bash
nxc list
```

---

# Explain

パッケージやサービスが現在どのような状態になっているかを確認できます。

## Package

```bash
nxc explain package firefox
```

現在のsystemに存在するか、NixOS設定で定義されているか、可能な範囲で定義元も確認できます。

## Service

```bash
nxc explain service openssh
```

NixOS設定上の定義と、現在systemdで有効になっている状態を分けて確認できます。

---

# Safety

NXCは、既存のNixOS設定を見つけたからといって無条件に書き換えることはありません。

既存設定を安全に編集できると判断できない場合は、設定構造を推測して変更する代わりに、NXC専用のmoduleへフォールバックします。

また、設定変更後はNix評価と `nixos-rebuild switch` を実行します。

```text
入力
 ↓
検証
 ↓
名前解決
 ↓
実行計画
 ↓
NixOS設定変更
 ↓
Nix評価
 ↓
nixos-rebuild switch
 ↓
system確認
```

rebuildや検証に失敗した場合は、可能な範囲で変更前の状態へロールバックします。

NixOSの設定を変更するため、通常の操作ではsudo権限が必要です。

---

# Commands

| Command | 説明 |
| --- | --- |
| `nxc <package>` | パッケージをインストール |
| `nxc i <package>` | パッケージをインストール |
| `nxc r <package>` | パッケージを削除 |
| `nxc e <service>` | サービスを有効化 |
| `nxc d <service>` | サービスを無効化 |
| `nxc discover` | NixOS Flakeを探索 |
| `nxc list` | system-wideなコマンド / アプリを一覧表示 |
| `nxc status` | system / Discovery状態を表示 |
| `nxc explain package <name>` | パッケージの状態・定義元を表示 |
| `nxc explain service <name>` | サービスの状態・定義元を表示 |
| `nxc --help` | ヘルプを表示 |

---

# Help

すべてのコマンドは、

```bash
nxc --help
```

で確認できます。

特定の操作について詳しく確認したい場合も、まず `--help` を利用してください。

---

# License

License information will be added as the project matures.
