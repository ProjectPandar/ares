#!/usr/bin/env bash
# Runs the extracted OrcaSlicer 2.4.2 AppImage binary for the parity harness
# by assembling a host LD_LIBRARY_PATH from nixpkgs runtime libraries.
#
# Override the extracted AppImage location with ORCA_APPDIR (default
# /tmp/squashfs-root). The parity test suite invokes this script through
# ARES_ORCA_BIN.
set -euo pipefail

APPDIR="${ORCA_APPDIR:-/tmp/squashfs-root}"

if [ ! -x "$APPDIR/bin/orca-slicer" ]; then
    echo "orca-slicer not found under $APPDIR (set ORCA_APPDIR)" >&2
    exit 127
fi

# The nix eval is expensive and flaky under repeated invocations; cache the
# resolved library paths across runs (force a refresh with ORCA_LIB_REFRESH=1).
CACHE="${ORCA_LIB_CACHE:-/tmp/orca-parity-libs.cache}"
cache_valid=0
if [ -s "$CACHE" ] && [ -z "${ORCA_LIB_REFRESH:-}" ]; then
    cache_valid=1
    IFS=: read -r -a cached_paths <<< "$(cat "$CACHE")"
    for path in "${cached_paths[@]}"; do
        if [ ! -d "$path" ]; then
            cache_valid=0
            break
        fi
    done
fi

if [ "$cache_valid" -eq 1 ]; then
    extra="$(cat "$CACHE")"
else
    extra="$(nix build --no-link --print-out-paths --impure --expr 'with import <nixpkgs> {}; builtins.map lib.getLib [ gtk3 webkitgtk_4_1 libGLU pango glib glib-networking gst_all_1.gstreamer gst_all_1.gst-plugins-base libsoup_3 libx11 libxext libxkbcommon libSM libICE libglvnd stdenv.cc.cc.lib wayland harfbuzz atk cairo gdk-pixbuf fontconfig.lib dbus libsecret ]' | sed 's#$#/lib#' | paste -sd:)"
    printf '%s' "$extra" > "$CACHE"
fi

export LD_LIBRARY_PATH="$APPDIR/lib/orca-runtime:$APPDIR/bin:$extra${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LC_ALL=C
export APPDIR

exec "$APPDIR/bin/orca-slicer" "$@"
