{ pkgs, ... }:

{
  packages = with pkgs; [
    pkg-config
    librsvg
    webkitgtk_4_1
    playwright-test
    playwright-driver
  ];

  android = {
    enable = true;
    platforms.version = [
      "35"
      "36"
    ];
    platformTools.version = "35.0.2";
    buildTools.version = [
      "35.0.0"
      "36.0.0"
    ];
  };

  enterShell = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS"
    export __NV_DISABLE_EXPLICIT_SYNC=1
    export RUSTC_WRAPPER=""
  '';
}
