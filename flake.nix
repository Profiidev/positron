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
    let
      mkPositron =
        pkgs:
        pkgs.rustPlatform.buildRustPackage rec {
          pname = "positron";
          version = "0.3.8";

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
            glib-networking
            gsettings-desktop-schemas
            gtk-layer-shell
            aws-lc
          ];

          cargoHash = "sha256-cHeSd1DfnK7pjmi9ysL1sJEkhN4WzWGCOd3TQpvOSjo=";

          # Compile fails due to perl missing
          doCheck = false;
        };
    in
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = mkPositron pkgs;
      }
    ))
    // {
      overlays.default = final: _: {
        positron = mkPositron final;
      };
    };
}
