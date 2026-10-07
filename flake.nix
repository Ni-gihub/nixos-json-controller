{
  description = "NixOS JSON Controller";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-appstream-data = {
      url = "github:snowfallorg/nixos-appstream-data";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-appstream-data }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
      appstreamData = nixos-appstream-data.packages.${system}.appstream-data-all;
    in
    {
      packages.${system} = {
        appstream-data = appstreamData;

        default =
          pkgs.rustPlatform.buildRustPackage {
            pname = "nixos-json-controller";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            doCheck = false;

            nativeBuildInputs = [ pkgs.makeWrapper ];

            postInstall = ''
              mv $out/bin/nixos-json-controller $out/bin/nxc
              wrapProgram $out/bin/nxc \
                --set NXC_APPSTREAM_DATA ${appstreamData}
            '';
          };
      };
    };
}
