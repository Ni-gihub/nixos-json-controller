{ pkgs ? import <nixpkgs> {} }:

let
  gsettingsSchemasPath = builtins.concatStringsSep ":" [
    "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}"
    "${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}"
  ];

  gstPluginPath = builtins.concatStringsSep ":" [
    "${pkgs.gst_all_1.gstreamer.out}/lib/gstreamer-1.0"
    "${pkgs.gst_all_1.gst-plugins-base}/lib/gstreamer-1.0"
    "${pkgs.gst_all_1.gst-plugins-good}/lib/gstreamer-1.0"
  ];
in
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
    gsettings-desktop-schemas

    # GStreamer / WebKitGTK media support
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
  ];

  # WebKitGTK on Wayland needs these schemas to detect display scaling.
  GSETTINGS_SCHEMAS_PATH = gsettingsSchemasPath;

  # Make GStreamer plugin discovery work in the Nix development shell.
  GST_PLUGIN_SYSTEM_PATH_1_0 = gstPluginPath;

  shellHook = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:''${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

    echo "nixos-json-controller development environment"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Node: $(node --version)"
    echo "pnpm: $(pnpm --version)"
    echo "GSETTINGS_SCHEMAS_PATH=$GSETTINGS_SCHEMAS_PATH"
    echo "GST_PLUGIN_SYSTEM_PATH_1_0=$GST_PLUGIN_SYSTEM_PATH_1_0"
  '';
}