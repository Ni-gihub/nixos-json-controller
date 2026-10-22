# インストール

このページでは、現在利用可能な NXC CLI のインストール方法を説明します。

> **注意:** デスクトップ版 App Store GUI は開発環境での起動・動作確認ができる段階です。GUI の本番配布用 package、ランチャー統合、profile の更新・削除手順は整備中のため、GUI を本番用パッケージとしてインストールできるとは案内していません。進捗は [Issue #101](https://github.com/Ni-gihub/nixos-json-controller/issues/101) を参照してください。

## 前提条件

- Nix が利用できる Linux 環境
- 現在の Flake package の対応 system は `x86_64-linux` のみ
- Flakes と `nix-command` が利用可能であること

`aarch64-linux` など、ほかのアーキテクチャは現時点でこの Flake package の対応対象に含まれません。別アーキテクチャで利用する場合は、対応する package output が追加されていることを確認してください。

Flakes が無効の場合、Nix の設定で次を有効にしてください。

```ini
experimental-features = nix-command flakes
```

## CLI を Nix profile からインストール

```sh
nix profile install github:Ni-gihub/nixos-json-controller
```

インストール後、コマンドが利用できることを確認します。

```sh
nxc --help
```

NXC 自身を利用者の NixOS configuration の `flake.nix` に追加する必要はありません。ユーザーの Nix profile で管理できます。

## 更新

まず、profile element 名を確認します。

```sh
nix profile list
```

通常は次のコマンドで更新できます。

```sh
nix profile upgrade nixos-json-controller
```

profile element 名が異なる場合は、`nix profile list` に表示された実際の名前を指定してください。

## 削除

profile element 名を確認した後、該当する element を削除します。

```sh
nix profile list
nix profile remove nixos-json-controller
```

表示された名前が異なる場合は、実際の element 名に置き換えてください。profile から NXC を削除しても、NXC が利用者の NixOS 設定に宣言されていたことにはなりません。

## ソースから CLI をビルド

開発・調査目的でソースからビルドする場合は、Rust と Cargo が必要です。

```sh
git clone https://github.com/Ni-gihub/nixos-json-controller.git
cd nixos-json-controller
cargo build --release
```

生成される実行ファイルは `target/release/nixos-json-controller` です。ユーザー用の PATH が通ったディレクトリへ配置する場合は、たとえば次のようにします。

```sh
install -Dm755 target/release/nixos-json-controller ~/.local/bin/nxc
```

これは実行ファイルを手動配置する方法です。Nix profile でのインストールとは管理方法が異なります。

## 次に読むページ

- [CLI の使い方](CLI-Usage.md)
- [トラブルシューティング](Troubleshooting.md)
