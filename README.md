# NixOS-JSON-Controller

NixOSの設定を安全に変更するためのCLIコントローラーです。

パッケージの追加・削除、サービスの有効化・無効化などを、NixOSの宣言的な設定として反映し、必要に応じて `nixos-rebuild switch` まで実行します。

> **Status:** CLIのコア機能は一旦完成したMVPです。今後は実際の利用を通して改善していきます。

## Features

- NixOSパッケージの追加・削除
- NixOSサービスの有効化・無効化
- 日本語などの別名に対応
- NixOS設定用FlakeのDiscovery
- 現在のsystem / Discovery状態の確認
- パッケージ・サービスの状態や定義元の確認
- 宣言されているパッケージの一覧表示
- 宣言されているパッケージの検索
- 既存設定を安全に編集できない場合のNXC専用module fallback
- 設定変更後のNix評価・rebuildによる検証
- rebuildや検証に失敗した場合の変更ロールバック
- `--dry-run` による実行内容の確認

---

# Installation

## NixOS / Nix profile

NixOSでは、通常のアプリケーションと同じようにNix profileへインストールできます。

```bash
nix profile install github:Ni-gihub/nixos-json-controller
```

インストール後、

```bash
nxc --help
```

で確認できます。

現在のFlake packageは `x86_64-linux` を対象としています。

NXC自身をNixOSの `flake.nix` に追加する必要はありません。

## 更新

profileにインストールしたNXCは通常のprofileアプリとして更新できます。

```bash
nix profile upgrade nixos-json-controller
```

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

`nxc` として使用する場合はPATHの通った場所へ配置してください。

```bash
install -Dm755 target/release/nixos-json-controller ~/.local/bin/nxc
```

---

# Quick Start

最初にNixOS設定をDiscoveryします。

```bash
nxc discover
```

Discoveryに成功したら、通常の操作を実行できます。

### Package

```bash
nxc i firefox
nxc r firefox
```

インストール操作は省略して、

```bash
nxc firefox
```

とも書けます。

### Service

```bash
nxc e openssh
nxc d openssh
```

### 状態確認

```bash
nxc status
nxc list
nxc search firefox
```

---

# Discovery

NXCはNixOS設定用のFlakeを自動的に探索します。

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

または、

```bash
nxc firefox
```

## 削除

```bash
nxc r firefox
```

## Dry Run

実際に設定を変更せず、実行計画を確認できます。

```bash
nxc i firefox --dry-run
```

`--dry-run` は設定を変更したりrebuildしたりせず、実行予定の内容だけを表示します。

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

# List

`nxc list` は、NixOSの `environment.systemPackages` に実際に宣言されているパッケージを一覧表示します。

```bash
nxc list
```

NixOSの設定ファイルを単純に検索して一覧を作るのではなく、評価されたNixOS設定の情報を利用します。

そのため、複数のmoduleから宣言されているパッケージも対象になります。

---

# Search

`nxc search` は、現在のNixOS設定に宣言されているパッケージを検索します。

```bash
nxc search firefox
```

検索は大文字・小文字を区別しない部分一致です。

検索結果では、該当パッケージについて次の情報を確認できます。

- NixOS設定に宣言されているか
- 現在のsystemで有効になっているか
- 宣言元の設定ファイル
- system environment
- 同名の実行ファイルが存在する場合のパス

`search` はNixpkgs全体やsystem上の全バイナリを検索するコマンドではありません。

---

# System

## Status

現在のNXCとNixOS systemの状態を確認できます。

```bash
nxc status
```

Discovery済みの場合は、使用中のFlakeとConfigurationも確認できます。

表示される主な情報:

- 現在のgeneration
- system上のコマンド / アプリ数
- 検出されたパッケージ数
- 有効なsystemd service数
- Discovery済みのFlake
- 使用中のNixOS Configuration

---

# Explain

パッケージやサービスが現在どのような状態になっているかを確認できます。

## Package

```bash
nxc explain package firefox
```

現在のsystemに存在するか、NixOS設定で定義されているか、定義元や安全に編集可能な場所を確認できます。

## Service

```bash
nxc explain service openssh
```

NixOS設定上の定義と、現在systemdで有効になっている状態を分けて確認できます。

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

# Safety

NXCは、既存のNixOS設定を見つけたからといって無条件に書き換えることはありません。

パッケージやサービスを追加するときは、NXC専用moduleを利用できる場合はそこへ変更を分離します。

既存設定を安全に編集できない場合は、設定構造を推測して無理に変更するのではなく、安全側に停止または専用moduleへフォールバックします。

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

NixOSの設定を変更するため、通常の変更操作ではsudo権限が必要です。

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
| `nxc list` | 宣言されているsystem packagesを一覧表示 |
| `nxc search <query>` | 宣言されているsystem packagesを検索 |
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

---

# Project Direction

NXCは、NixOSを直接操作するためのCLIとしてコア機能をまとめています。

今後はNXCのコアを利用するGUIやAPIなど、より使いやすいインターフェースへ発展させることを想定しています。

CLI自体は、実際の利用で問題が見つかった場合に改善していく方針です。

---

# License

現在、ライセンスは設定されていません。
