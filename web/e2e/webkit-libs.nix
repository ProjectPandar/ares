# Shared libraries that Playwright's bundled headless WebKit
# (webkit-<rev>/minibrowser-wpe under ~/.cache/ms-playwright) links beyond the
# ones it bundles in lib/ and sys/lib/. NixOS has no FHS library path, so
# e2e/webkit-run.sh puts this environment on LD_LIBRARY_PATH.
#
#   ARES_E2E_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
#   nix build --impure --no-link --print-out-paths --file web/e2e/webkit-libs.nix
let
  nixpkgs = builtins.getEnv "ARES_E2E_NIXPKGS";
  pkgs = import nixpkgs { };
in
assert nixpkgs != "";
pkgs.symlinkJoin {
  name = "ares-e2e-webkit-libs";
  paths = map pkgs.lib.getLib (with pkgs; [
    at-spi2-core
    stdenv.cc.cc
    libavif
    libdrm
    enchant
    libepoxy
    libevent
    expat
    flite
    fontconfig
    freetype
    libgbm
    libgcrypt
    glib
    libgpg-error
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-bad
    harfbuzz
    # libharfbuzz-icu only; its libharfbuzz.so.0 lives in harfbuzz.
    harfbuzzFull
    hyphen
    icu74
    libjpeg8
    lcms2
    libmanette
    libopus
    libpng
    libpsl
    nghttp2.lib
    sqlite
    systemd
    libtasn1
    wayland
    libwebp
    libxkbcommon
    # WebKit links the libxml2.so.2 soname; nixpkgs libxml2 is .so.16.
    libxml2_13
    libx11
    libxslt
    zlib
    libGL
  ]);
}
