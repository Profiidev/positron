{ pkgs, ... }:

{
  packages = with pkgs; [
    pkg-config
    librsvg
    webkitgtk_4_1
  ];

  android.enable = true;

  enterShell = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS"
    export __NV_DISABLE_EXPLICIT_SYNC=1
    export RUSTC_WRAPPER=""
  '';
}
