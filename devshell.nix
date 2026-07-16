{ pkgs }:

pkgs.mkShell {
  name = "nixos-json-controller-dev";

  packages = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy

    pkg-config
    openssl
  ];

  shellHook = ''
    echo "nixos-json-controller development environment"
    rustc --version
    cargo --version
  '';
}