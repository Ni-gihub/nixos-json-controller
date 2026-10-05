{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "nixos-json-controller-dev";

  packages = with pkgs; [
    # Rust
    rustc
    cargo
    rustfmt
    clippy

    # Node.js / frontend
    nodejs
    pnpm

    # Tauri / Rust build dependencies
    pkg-config
    openssl

    # Linux / GTK / WebKit
    gtk3
    webkitgtk_4_1
    glib
    cairo
    pango
    gdk-pixbuf
    atk
    libsoup_3
    webkitgtk_4_1
  ];

  shellHook = ''
    echo "nixos-json-controller development environment"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Node: $(node --version)"
    echo "pnpm: $(pnpm --version)"
  '';
}