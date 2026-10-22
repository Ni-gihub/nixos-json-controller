# CLI コマンドリファレンス

NXC CLI は、NixOS configuration に対するパッケージ・サービス操作と、現在の宣言状態を確認する機能を提供します。

すべてのコマンドとオプションは次で確認できます。

```sh
nxc --help
```

## 最初のセットアップ：Discovery

最初に NixOS configuration の Flake を探索します。

```sh
nxc discover
```

NXC は候補となる Flake と configuration を調査し、選択結果を保存します。保存先は次のとおりです。

```text
~/.config/nxc/discovery.json
```

NixOS 設定の場所や構成を変更した場合は、Discovery を再実行してください。

```sh
nxc discover
```

## パッケージ管理

### インストール

```sh
nxc i firefox
```

操作名を省略したインストール形式も利用できます。

```sh
nxc firefox
```

### 削除

```sh
nxc r firefox
```

### 実行計画だけを確認する

```sh
nxc i firefox --dry-run
```

dry-run は設定変更や `nixos-rebuild switch` を実行せず、予定される処理を確認するためのものです。dry-run の結果は、実際の処理が必ず成功することを保証するものではありません。

## サービス管理

サービスを有効化する例：

```sh
nxc e openssh
```

サービスを無効化する例：

```sh
nxc d openssh
```

辞書に登録された別名を利用できる場合があります。たとえば `ssh` や `sshd` は `openssh` として解決されます。

## 状態の確認

### システムと NXC の状態

```sh
nxc status
```

### NixOS configuration に宣言されたパッケージを一覧表示

```sh
nxc list
```

`nxc list` は、評価された configuration の `environment.systemPackages` に宣言されたパッケージを表示します。システムに存在するすべてのコマンドや、Nixpkgs 全体のパッケージ一覧ではありません。

### 宣言されたパッケージを検索

```sh
nxc search firefox
```

この検索も、NixOS configuration に宣言されたパッケージ集合が対象です。Nixpkgs 全体の検索とは役割が異なります。

### パッケージの状態と宣言元を確認

```sh
nxc explain package firefox
```

### サービスの状態と宣言元を確認

```sh
nxc explain service openssh
```

## 権限と安全性

設定変更や再ビルドに必要な操作では、sudo 認証を求められる場合があります。また、変更対象の設定構造を安全に判断できない場合、NXC は処理を拒否することがあります。

詳細は [Safety and Architecture](Safety-and-Architecture.md) を参照してください。
