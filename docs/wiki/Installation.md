# インストール

このページでは、現在利用可能な NXC CLI のインストール方法を説明します。

> **注意:** デスクトップ版 App Store GUI は開発環境での起動・動作確認ができる段階です。GUI の本番配布用 package、ランチャー統合、profile の更新・削除手順は整備中のため、GUI を本番用パッケージとしてインストールできるとは案内していません。進捗は [Issue #101](https://github.com/Ni-gihub/nixos-json-controller/issues/101) を参照してください。

## 前提条件

- **NixOS を実行している `x86_64-linux` 環境**
- Flakes と `nix-command` が利用可能であること

現在の Flake package は `x86_64-linux` のみを対象としています。また、NXC CLI は NixOS の状態取得・設定変更と `nixos-rebuild switch` を前提にしているため、Nix を導入しただけの Ubuntu / Debian など、非 NixOS 環境はサポート対象ではありません。

`aarch64-linux` など、ほかのアーキテクチャも現在の Flake package の対応対象に含まれません。別アーキテクチャへの対応は、対応する Flake output が追加されていることを確認してください。

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

開発・調査目的でソースからビルドする場合は、Rust / Cargo 1.85.0 以降が必要です。これは `edition = "2024"` を使うための最低バージョンです。依存関係がより新しいツールチェーンを要求する場合もあるため、Nix が利用できる場合はリポジトリの開発シェルを使う方法を推奨します。

```sh
git clone https://github.com/Ni-gihub/nixos-json-controller.git
cd nixos-json-controller
nix develop
cargo build --release
```

`nix develop` を使わない場合は、Rust と Cargo のバージョンを確認します。

```sh
rustc --version
cargo --version
```

開発シェルを利用する場合も、現在の `flake.nix` が定義している開発環境は `x86_64-linux` 向けです。

生成される実行ファイルは `target/release/nixos-json-controller` です。ユーザー用の PATH が通ったディレクトリへ配置する場合は、たとえば次のようにします。

```sh
install -Dm755 target/release/nixos-json-controller ~/.local/bin/nxc
```

これは実行ファイルを手動配置する方法です。Nix profile でのインストールとは管理方法が異なります。

## 次に読むページ

- [CLI の使い方](CLI-Usage.md)
- [トラブルシューティング](Troubleshooting.md)
