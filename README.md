# NixOS-JSON-Controller

NixOSの設定変更を安全かつ決定的に実行するCLIコントローラー。

パッケージやサービスの操作を、

```text
CLI / JSON
    ↓
Parser
    ↓
Validator
    ↓
Resolver
    ↓
Dictionary
    ↓
ExecutionPlan
    ↓
Executor
    ↓
NixOS configuration
    ↓
nixos-rebuild switch
```

という流れで処理します。

また、NixOSの設定リポジトリを自動的に探索・検査する **Discovery機能** を備えています。

```text
nxc discover
    ↓
Candidate Search
    ↓
Filesystem Inspection
    ↓
Nix Evaluation
    ↓
Configuration Selection
    ↓
Discovery State
```

DiscoveryによってNixOS設定の場所と使用する `nixosConfiguration` を特定し、その結果を保存します。

通常の `nxc` 操作では保存済みのDiscovery結果を利用するため、毎回重い探索やNix評価を実行する必要はありません。

---

## Features

* NixOSパッケージの追加・削除
* NixOSサービスの有効化・無効化
* 日本語などの別名からパッケージ・サービスを解決
* 辞書に存在しない名前を拒否
* JSONによる構造化された入力
* Rustによる入力検証・名前解決・実行制御
* NixOSの既存設定を利用した設定変更
* `nixos-rebuild switch` まで一連の処理を実行
* NixOS設定用Flakeの自動Discovery
* `nixosConfigurations` のNix評価
* ホスト名・システム情報などを利用したConfiguration選択
* Discovery結果の保存
* 通常操作時の重いDiscovery処理を回避

---

## Why?

NixOSでは設定ファイルを変更してから `nixos-rebuild` を実行することで、システム設定を宣言的に管理できます。

一方で、単純なパッケージ追加やサービス変更であっても、

1. 設定ファイルを探す
2. 設定を変更する
3. `nixos-rebuild` を実行する

という操作が必要になります。

NixOS-JSON-Controllerでは、この操作をCLIから安全に実行できるようにします。

特に、自然言語をそのままOS操作へ渡すのではなく、

```text
自然言語
   ↓
LLM
   ↓
JSON
   ↓
Rust Controller
   ↓
NixOS
```

という責任分離を想定しています。

LLMはJSONを生成するだけで、実際のOS変更はRust Controllerが担当します。

---

# Installation

## Requirements

* NixOS
* Nix
* Nix flakes
* Rust / Cargo
* Git
* `nixos-rebuild`

---

## From source

リポジトリを取得します。

```bash
git clone https://github.com/Ni-gihub/nixos-json-controller.git
cd nixos-json-controller
```

### Cargoでインストール

```bash
cargo install --path .
```

インストールされる実行ファイルは、

```text
~/.cargo/bin/nixos-json-controller
```

です。

`~/.cargo/bin` がPATHに含まれていれば、以下のコマンドで実行できます。

```bash
nixos-json-controller --help
```

このプロジェクトではCLI名として `nxc` を使用します。

```bash
nxc --help
```

`nxc` が見つからない場合は、`~/.cargo/bin` がPATHに含まれていることを確認してください。

```bash
echo $PATH
```

必要であれば一時的に、

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

を実行できます。

---

## Build only

インストールせずにビルドする場合:

```bash
cargo build --release
```

実行ファイル:

```text
target/release/nixos-json-controller
```

直接実行することもできます。

```bash
./target/release/nixos-json-controller --help
```

---

# Discovery

NixOS-JSON-Controllerには、NixOS設定用Flakeを探索するDiscovery機能があります。

```bash
nxc discover
```

Discoveryでは単純に `flake.nix` が存在するディレクトリを探すだけではなく、候補となったFlakeを検査し、Nix評価によって `nixosConfigurations` を確認します。

基本的な流れは、

```text
Search
  ↓
候補Flakeを探索
  ↓
Filesystem Inspection
  ↓
Flakeの構造を確認
  ↓
Nix Evaluation
  ↓
nixosConfigurationsを評価
  ↓
候補を選択
  ↓
Discovery Resultを保存
```

です。

例えば、

```text
NixOS Flake Discovery

Searching for candidates...
Found 2 candidates.

Inspecting candidates...

Candidate ranking:
[1] /home/user/Projects/nix-config
    score: 108
    NixOS configurations: 1
      - laptop (nixos-laptop, x86_64-linux)

[2] /home/user/Projects/nixos-json-controller
    score: 15
    NixOS configurations: 0

Discovery successful.

Flake: /home/user/Projects/nix-config
Configuration: laptop
Hostname: nixos-laptop
System: x86_64-linux
Selection: hostname match

Discovery result saved.
```

のように、候補を検査したうえで使用するFlakeとConfigurationを決定します。

## Discovery result

Discoveryで決定された情報はユーザーの設定ディレクトリに保存されます。

```text
~/.config/nxc/discovery.json
```

通常の `nxc` 操作では、この保存済みDiscovery結果を利用します。

そのため、

```bash
nxc discover
```

で一度設定を検出した後は、

```bash
nxc i firefox
nxc r firefox
nxc e openssh
nxc d openssh
```

などの通常操作で毎回Flake探索やNix評価を行いません。

---

## Re-discovery

NixOS設定の場所や構成を変更した場合は、Discoveryを再実行します。

```bash
nxc discover
```

保存済みのDiscovery結果が現在の環境と整合しない場合、通常操作はそのまま実行せず、

```text
NixOS flake discovery is not available.
Run `nxc discover` first.
```

などのエラーを返します。

これにより、古いDiscovery情報を使用したままNixOS設定を変更することを避けます。

---

# Usage

## Discover NixOS configuration

最初にNixOS設定をDiscoveryします。

```bash
nxc discover
```

Discoveryが成功したら、通常の操作を実行できます。

---

## Install package

```bash
nxc i firefox
```

パッケージ操作では操作を省略することもできます。

```bash
nxc firefox
```

これは、

```bash
nxc i firefox
```

と同じ意味です。

---

## Japanese aliases

辞書に登録されている別名を使用できます。

```bash
nxc i ファイアフォックス
nxc i ファイヤーフォックス
nxc i 火狐
```

例えば、

```text
ファイヤーフォックス
        ↓
    Dictionary
        ↓
      firefox
```

として処理されます。

---

## Remove package

```bash
nxc r firefox
```

---

## Enable service

```bash
nxc e openssh
```

別名にも対応しています。

```bash
nxc e ssh
nxc e sshd
```

内部では、

```text
ssh
 ↓
openssh
```

のようにサービス辞書からCanonical Nameへ解決されます。

---

## Disable service

```bash
nxc d openssh
```

---

## Help

```bash
nxc --help
```

または、

```bash
nxc -h
```

---

# Commands

| Command           | Action                |
| ----------------- | --------------------- |
| `nxc <package>`   | パッケージをインストール          |
| `nxc i <package>` | パッケージをインストール          |
| `nxc r <package>` | パッケージを削除              |
| `nxc e <service>` | サービスを有効化              |
| `nxc d <service>` | サービスを無効化              |
| `nxc discover`    | NixOS FlakeをDiscovery |
| `nxc --help`      | ヘルプを表示                |

---

# Processing Flow

## 1. Command

CLIから入力を受け取ります。

```bash
nxc i firefox
```

↓

```text
Action::InstallPackage
Target("firefox")
```

---

## 2. Validator

入力形式を検証します。

* Actionが有効か
* Targetが存在するか
* Targetが空でないか
* Targetが空白だけではないか
* Targetに不正な空白がないか
* Targetが長すぎないか

---

## 3. Resolver

入力されたTargetをDictionaryから解決します。

```text
"ファイヤーフォックス"
        ↓
     Dictionary
        ↓
     "firefox"
```

パッケージ操作ならPackage Dictionary、サービス操作ならService Dictionaryを使用します。

---

## 4. ExecutionPlan

解決された情報から、実行する操作を固定します。

```text
Action
Target
DryRun
```

この段階ではまだNixOSを変更しません。

---

## 5. Executor

ExecutionPlanに従ってNixOSの設定を変更します。

例えばパッケージ追加の場合:

```nix
environment.systemPackages = with pkgs; [
  firefox
];
```

---

## 6. nixos-rebuild

設定変更後、NixOSを再ビルドします。

```bash
sudo nixos-rebuild switch --flake <flake>#<configuration>
```

Discoveryによって決定されたFlakeとConfigurationが使用されます。

---

# Discovery Architecture

Discoveryは通常のコマンド実行とは独立した機能として実装されています。

```text
src/nixos/discovery/

├── candidate.rs
├── context.rs
├── inspection.rs
├── nix.rs
├── result.rs
├── search.rs
├── selector.rs
└── state.rs
```

### Candidate

探索で発見されたFlake候補を表します。

候補には探索元や検査結果などの情報が付与されます。

### Search

Discovery対象となるFlake候補を探索します。

### Inspection

候補となったFlakeのファイルシステムやNix評価結果を検査します。

### Nix

Nixコマンドを利用してFlakeの情報や `nixosConfigurations` を評価します。

### Selector

複数の候補やConfigurationから使用する対象を選択します。

現在はホスト名やシステム情報などを利用した選択を行います。

### Result

Discoveryの最終結果を表します。

```text
Flake root
Flake file
Selected configuration
Selection method
```

などを保持します。

### State

Discovery結果を保存・読み込みします。

保存先:

```text
~/.config/nxc/discovery.json
```

### Context

通常の `nxc` 操作からDiscovery結果を利用するためのContextです。

通常操作では重いNix Discoveryを実行せず、保存済みのDiscovery結果を利用します。

---

# Dictionary

パッケージとサービスには、それぞれ専用の辞書があります。

```text
src/dictionary/

├── dictionary.rs
├── package.rs
├── packages.json
├── service.rs
└── services.json
```

## Package dictionary

例えば、

```json
{
  "firefox": [
    "firefox",
    "ファイアフォックス",
    "ファイヤーフォックス",
    "火狐"
  ]
}
```

の場合、

```text
firefox
ファイアフォックス
ファイヤーフォックス
火狐
```

はすべて、

```text
firefox
```

として扱われます。

---

## Service dictionary

例えば、

```json
{
  "openssh": [
    "openssh",
    "ssh",
    "sshd",
    "sshサーバー"
  ]
}
```

の場合、

```bash
nxc e ssh
```

は内部的に、

```text
ssh
 ↓
openssh
```

へ解決されます。

---

# Safety Model

このプロジェクトでは、**LLMに直接OS操作をさせないこと**を重要な設計方針としています。

```text
┌─────────────┐
│     LLM     │
│  JSON生成   │
└──────┬──────┘
       │ JSON
       ▼
┌─────────────┐
│    Rust     │
│ Controller  │
│             │
│  Validate   │
│  Resolve    │
│  Plan       │
│  Execute    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    NixOS    │
│Configuration│
└─────────────┘
```

LLMは実行権限を持たず、Rust Controllerが実際の変更処理を担当します。

これにより、自然言語の曖昧な解釈をそのままOS操作へ渡さず、構造化された入力を検証してから実行する構成を目指しています。

---

# Project Structure

```text
src/

├── cli.rs
│
├── command/
│   ├── action.rs
│   ├── command.rs
│   ├── parser.rs
│   └── target.rs
│
├── dictionary/
│   ├── dictionary.rs
│   ├── package.rs
│   ├── packages.json
│   ├── service.rs
│   └── services.json
│
├── validator/
│   ├── error.rs
│   └── validator.rs
│
├── resolver/
│   ├── error.rs
│   ├── package.rs
│   ├── resolver.rs
│   └── service.rs
│
├── planner/
│   ├── execution_plan.rs
│   └── planner.rs
│
├── executor/
│   ├── error.rs
│   ├── executor.rs
│   ├── package.rs
│   └── service.rs
│
└── nixos/
    ├── discovery/
    │   ├── candidate.rs
    │   ├── context.rs
    │   ├── inspection.rs
    │   ├── nix.rs
    │   ├── result.rs
    │   ├── search.rs
    │   ├── selector.rs
    │   └── state.rs
    │
    ├── flake.rs
    ├── generator.rs
    ├── module.rs
    └── rebuild.rs
```

---

# Testing

テストはCargoで実行できます。

```bash
cargo test
```

テストをシリアル実行する場合:

```bash
cargo test -- --test-threads=1
```

特定のモジュールだけテストする場合:

```bash
cargo test --lib dictionary
```

```bash
cargo test --lib resolver
```

```bash
cargo test --lib planner
```

```bash
cargo test --lib nixos
```

現在、Command / Validator / Dictionary / Resolver / Planner / Executor / NixOS / Discovery関連のユニットテストを実装しています。

また、パイプライン全体を確認するIntegration Testも実装しています。

---

# Development

開発用ビルド:

```bash
cargo build
```

リリースビルド:

```bash
cargo build --release
```

テスト:

```bash
cargo test
```

テストをシリアル実行:

```bash
cargo test -- --test-threads=1
```

フォーマット:

```bash
cargo fmt
```

---

# Current Status

現在のコア機能:

* [x] CLI
* [x] JSON / Command parsing
* [x] Input validation
* [x] Package dictionary
* [x] Service dictionary
* [x] Alias resolution
* [x] Execution Plan
* [x] Package installation
* [x] Package removal
* [x] Service enable / disable
* [x] NixOS configuration modification
* [x] `nixos-rebuild switch`
* [x] Unit tests
* [x] NixOS Flake Discovery
* [x] Flake candidate search
* [x] Filesystem inspection
* [x] Nix evaluation
* [x] `nixosConfigurations` detection
* [x] Configuration selection
* [x] Discovery state persistence
* [x] Saved Discovery result validation

今後の予定:

* [ ] Dictionary validation
* [ ] Duplicate alias prevention
* [ ] Unknown target handlingの強化
* [ ] Package / Service dictionaryの整理
* [ ] JSON入力経路の強化
* [ ] Dry-runの拡張
* [ ] Confirmation機能
* [ ] Rollback支援
* [ ] 音声入力 / Whisper連携
* [ ] LLMによるJSON生成

---

# License

License information will be added as the project matures.
