{
  description = "NixOS JSON Controller";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      packages.${system}.default =
        pkgs.rustPlatform.buildRustPackage {
          pname = "nixos-json-controller";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          doCheck = false;

          postInstall = ''
            mv $out/bin/nixos-json-controller $out/bin/nxc
          '';
        };
    };
}