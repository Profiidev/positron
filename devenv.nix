{ pkgs, ... }:

let
  browsers =
    (builtins.fromJSON (builtins.readFile "${pkgs.playwright-driver}/browsers.json")).browsers;
  chromium-rev = (builtins.head (builtins.filter (x: x.name == "chromium") browsers)).revision;
in
{
  packages = with pkgs; [
    pkg-config
    librsvg
    webkitgtk_4_1
    nodejs
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

  languages.javascript.enable = true;

  scripts.intro.exec = ''
    playwrightNpmVersion=$(node -p "require('@playwright/test/package.json').version" 2>/dev/null)
    nixPlaywrightBaseVersion=$(echo "${pkgs.playwright.version}" | cut -d. -f1,2)
    npmPlaywrightBaseVersion=$(echo "$playwrightNpmVersion" | cut -d. -f1,2)

    if [ "$nixPlaywrightBaseVersion" != "$npmPlaywrightBaseVersion" ]; then
        echo "❄️ Playwright nix version: ${pkgs.playwright.version}"
        echo "📦 Playwright npm version: $playwrightNpmVersion"
        echo "❌ Playwright versions (major, minor) in nix ($nixPlaywrightBaseVersion in devenv.yaml) and npm ($npmPlaywrightBaseVersion in package.json) are not the same! Please adapt the configuration."
    fi
  '';

  enterShell = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS"
    intro
  '';

  env = {
    RUSTC_WRAPPER = "";
    __NV_DISABLE_EXPLICIT_SYNC = "1";
    PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright.browsers}";
    PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
    PLAYWRIGHT_NODEJS_PATH = "${pkgs.nodejs}/bin/node";
    PLAYWRIGHT_LAUNCH_OPTIONS_EXECUTABLE_PATH = "${pkgs.playwright.browsers}/chromium-${chromium-rev}/chrome-linux/chrome";
  };
}
