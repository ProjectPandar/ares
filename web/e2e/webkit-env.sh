#!/usr/bin/env bash
# Runs a command with Playwright's bundled WebKit able to start on NixOS.
# Builds e2e/webkit-libs.nix (from the nixpkgs in ARES_E2E_NIXPKGS) and points
# the iphone project at e2e/webkit-run.sh, which starts the bundled headless
# (WPE) MiniBrowser with those libraries. Hosts with Playwright's WebKit
# dependencies installed run the command directly instead.
#
#   ARES_E2E_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
#     e2e/webkit-env.sh npx playwright test --project=iphone
set -euo pipefail
: "${ARES_E2E_NIXPKGS:?set ARES_E2E_NIXPKGS to a nixpkgs source path}"
here=$(cd "$(dirname "$0")" && pwd -P)
libs=$(nix build --impure --no-link --print-out-paths --file "$here/webkit-libs.nix")
# pw_run.sh of the WebKit revision this @playwright/test pins.
pw_run=$(cd "$here" && node -e 'console.log(require("playwright-core").webkit.executablePath())')
export ARES_E2E_WEBKIT_LIBS="$libs/lib"
export ARES_E2E_WEBKIT_DIR="$(dirname "$pw_run")/minibrowser-wpe"
export ARES_E2E_WEBKIT="$here/webkit-run.sh"
exec "$@"
