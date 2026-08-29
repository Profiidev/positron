{
  description = "Positron";

  nixConfig = {
    extra-substituters = [
      "https://projects.cache.profidev.io"
    ];

    extra-trusted-public-keys = [
      "profidev.cachix.org:tg4xEn64UMdvA5jJYT8omo/CQHk8+spLyeGT2YAku70="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "positron";
          version = "0.3.1";

          src = ./.;

          cargoBuildFlags = [
            "-p"
            "positron"
          ];

          DESKTOP_APP_TARGET = "true";

          npmDeps = pkgs.importNpmLock {
            npmRoot = src;
          };

          nativeBuildInputs = with pkgs; [
            cacert
            cargo-tauri.hook
            nodejs
            pkg-config
            importNpmLock.npmConfigHook
            npmHooks.npmInstallHook
            glib
            wrapGAppsHook4
          ];

          buildInputs = with pkgs; [
            webkitgtk_4_1
            openssl
            glib-networking
            gsettings-desktop-schemas
            gtk-layer-shell
          ];

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Compile fails due to perl missing
          doCheck = false;
        };
      }
    );
}
