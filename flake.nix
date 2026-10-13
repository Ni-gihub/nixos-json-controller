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
      gsettingsSchemasPath = builtins.concatStringsSep ":" [
        "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}"
        "${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}"
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          cargo
          nodejs
          pkg-config
          pnpm
          rustc
          wrapGAppsHook4
        ];

        buildInputs = with pkgs; [
          dbus
          glib
          gsettings-desktop-schemas
          gtk3
          librsvg
          libsoup_3
          openssl
          webkitgtk_4_1
        ];

        # WebKitGTK on Wayland needs these schemas to detect the display scale.
        GSETTINGS_SCHEMAS_PATH = gsettingsSchemasPath;
        shellHook = ''
          export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
        '';
      };

      packages.${system} = {
        appstream-data = appstreamData;

        default = pkgs.rustPlatform.buildRustPackage {
          pname = "nixos-json-controller";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Cargo tests run in the dedicated CI test job. Keep this derivation
          # focused on validating the distributable CLI and its wrapper.
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
