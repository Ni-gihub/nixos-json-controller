# NixOS-JSON-Controller

NixOSの設定変更を安全かつ決定的に実行するためのCLIコントローラー。

ユーザーの入力を **JSON / CLI → 検証 → 名前解決 → 実行計画 → NixOS設定変更 → nixos-rebuild** という流れで処理します。

```text
User
 │
 ▼
CLI / JSON
 │
 ▼
Parser
 │
 ▼
Validator
 │
 ▼
Resolver
 │
 ▼
Dictionary
 │
 ▼
ExecutionPlan
 │
 ▼
Executor
 │
 ▼
NixOS configuration
 │
 ▼
nixos-rebuild switch
```

## Features

* NixOSのパッケージをCLIから追加・削除
* NixOSサービスの有効化・無効化
* 日本語などの別名からパッケージ・サービスを解決
* 辞書に存在しない名前を拒否
* JSONによる構造化された入力
* Rustによる入力検証・名前解決・実行制御
* NixOSの既存設定を利用した設定変更
* `nixos-rebuild switch` まで一連の処理を実行

## Why?

NixOSでは設定ファイルを変更してから `nixos-rebuild` を実行することで、システム設定を宣言的に管理できます。

一方で、単純なパッケージ追加やサービス変更であっても、設定ファイルを直接編集して再ビルドする必要があります。

このプロジェクトでは、その操作をCLIから安全に行えるようにします。

特に、自然言語をそのままOS操作に渡すのではなく、

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

## Install

### Requirements

* NixOS
* Rust / Cargo
* Git
* Nix flakes
* `nixos-rebuild`

RustプロジェクトはCargoでビルドできます。CargoはRustのパッケージ管理・ビルドツールです。

### From source

リポジトリを取得します。

```bash
git clone https://github.com/Ni-gihub/nixos-json-controller.git
cd nixos-json-controller
```

ビルドします。

```bash
cargo build --release
```

実行ファイルは以下に生成されます。

```text
target/release/nixos-json-controller
```

Cargoを使ってインストールする場合:

```bash
cargo install --path .
```

`cargo install --path` はローカルのCargoプロジェクトをビルドしてインストールする方法です。

インストール後:

```bash
nixos-json-controller --help
```

## Usage

### Install package

```bash
nxc i firefox
```

短縮コマンド:

```bash
nxc i firefox
```

日本語の別名にも対応しています。

```bash
nxc i ファイアフォックス
nxc i ファイヤーフォックス
nxc i 火狐
```

辞書によって最終的にNixOSのパッケージ名へ変換されます。

```text
ファイヤーフォックス
        ↓
     Dictionary
        ↓
      firefox
```

### Remove package

```bash
nxc r firefox
```

### Enable service

```bash
nxc e openssh
```

別名:

```bash
nxc e ssh
nxc e sshd
```

### Disable service

```bash
nxc d openssh
```

### Default package install

パッケージの場合、操作を省略できます。

```bash
nxc firefox
```

これは以下と同じ意味です。

```bash
nxc i firefox
```

### Help

```bash
nxc --help
```

または:

```bash
nxc -h
```

## Commands

| Command           | Action       |
| ----------------- | ------------ |
| `nxc <package>`   | パッケージをインストール |
| `nxc i <package>` | パッケージをインストール |
| `nxc r <package>` | パッケージを削除     |
| `nxc e <service>` | サービスを有効化     |
| `nxc d <service>` | サービスを無効化     |
| `nxc --help`      | ヘルプを表示       |

## Dictionary

パッケージとサービスには、それぞれ専用の辞書があります。

```text
src/dictionary/
├── dictionary.rs
├── package.rs
├── packages.json
├── service.rs
└── services.json
```

### Package dictionary

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

例えば、

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

### Service dictionary

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

そのため、

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

## Processing Flow

コマンドは以下の順番で処理されます。

### 1. Command

CLIから入力を受け取ります。

```bash
nxc i firefox
```

↓

```text
Action::InstallPackage
Target("firefox")
```

### 2. Validator

入力形式を検証します。

* actionが有効か
* targetが空でないか
* 空白だけではないか
* targetが長すぎないか

### 3. Resolver

入力されたtargetをDictionaryから解決します。

```text
"ファイヤーフォックス"
        ↓
     Dictionary
        ↓
     "firefox"
```

パッケージ操作ならパッケージ辞書、サービス操作ならサービス辞書を使用します。

### 4. ExecutionPlan

解決された情報から、実行する操作を固定します。

```text
Action
Target
DryRun
```

この段階ではまだNixOSを変更しません。

### 5. Executor

ExecutionPlanに従ってNixOSの設定ファイルを変更します。

例えばパッケージ追加:

```nix
environment.systemPackages = with pkgs; [
  firefox
];
```

### 6. nixos-rebuild

設定変更後、NixOSを再ビルドします。

```bash
nixos-rebuild switch
```

## Safety Model

このプロジェクトでは、**LLMに直接OS操作をさせない**ことを重要な設計方針としています。

```text
┌─────────────┐
│     LLM     │
│ JSON生成    │
└──────┬──────┘
       │ JSON
       ▼
┌─────────────┐
│    Rust     │
│ Controller  │
│             │
│ Validate    │
│ Resolve     │
│ Plan        │
│ Execute     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    NixOS    │
│ Configuration│
└─────────────┘
```

LLMは実行権限を持たず、Rust Controllerが実際の変更処理を担当します。

これにより、自然言語の曖昧な解釈をそのままOS操作へ渡さず、構造化された入力を検証してから実行する構成を目指しています。

## Project Structure

```text
src/
├── cli.rs
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
    ├── flake.rs
    ├── generator.rs
    ├── module.rs
    └── rebuild.rs
```

## Testing

テストはCargoで実行できます。

```bash
cargo test
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

現在、Command / Validator / Dictionary / Resolver / Planner / Executor / NixOS関連の各処理に対してユニットテストを実装しています。

## Development

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

フォーマット:

```bash
cargo fmt
```

## Current Status

現在は開発中です。

実装済み:

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

## License

License information will be added as the project matures.
