#!/usr/bin/env bash
# Runs the extracted OrcaSlicer 2.4.2 AppImage binary for the parity harness
# by assembling a host LD_LIBRARY_PATH from nixpkgs runtime libraries.
#
# Set ORCA_APPDIR to the explicitly selected extraction directory.
# Never discover arbitrary store builds: these may contain diagnostic patches
# or use a different toolchain. Version equality does not prove byte parity.
# The parity test suite invokes this script through ARES_ORCA_BIN.
set -euo pipefail

if [ -z "${ORCA_APPDIR:-}" ]; then
    echo "ORCA_APPDIR must name an explicitly selected reference extraction" >&2
    exit 127
fi
APPDIR="$ORCA_APPDIR"

if [ ! -x "$APPDIR/bin/orca-slicer" ]; then
    echo "orca-slicer not found under $APPDIR (set ORCA_APPDIR)" >&2
    exit 127
fi

# The nix eval is expensive and flaky under repeated invocations; cache the
# resolved library paths across runs (force a refresh with ORCA_LIB_REFRESH=1).
CACHE="${ORCA_LIB_CACHE:-/tmp/orca-parity-libs-v2.cache}"
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
    extra="$(nix build --no-link --print-out-paths --impure --expr 'with import <nixpkgs> {}; builtins.map lib.getLib [ gtk3 webkitgtk_4_1 libGLU pango glib glib-networking gst_all_1.gstreamer gst_all_1.gst-plugins-base libsoup_3 libx11 libxext libxkbcommon libSM libICE libglvnd stdenv.cc.cc.lib wayland harfbuzz atk cairo gdk-pixbuf fontconfig.lib dbus libsecret glibc ]' | sed 's#$#/lib#' | paste -sd:)"
    printf '%s' "$extra" > "$CACHE"
fi

native_library_path="$APPDIR/lib/orca-runtime:$APPDIR/bin:$extra${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# The nixpkgs runtime libraries may need a newer glibc than the host provides
# (Ubuntu CI image vs nixpkgs pin). Prefer the nix glibc's loader so the
# versioned glibc symbols resolve; fall back to the host loader when none is
# in the resolved set (e.g. wrapper tests with a synthetic library path).
ld=""
IFS=: read -r -a libdirs <<< "$extra"
for dir in "${libdirs[@]}"; do
    if [ -x "$dir/ld-linux-x86-64.so.2" ]; then
        ld="$dir/ld-linux-x86-64.so.2"
        break
    fi
done

# Exec'ing orca-slicer through the nix loader makes /proc/self/exe point at
# ld.so, so upstream's program_location()/resources_dir() resolves into the
# nix store and silently misses data files (resources/flush/flush_data_*.txt),
# changing computed outputs (e.g. flush_volumes_matrix). When ORCA_PATCHELF is
# provided, patch the AppImage interpreter once under a lock so the binary runs
# directly and keeps a correct exe path; fall back to the ld.so launch only
# when patchelf is unavailable.
if [ -n "$ld" ] && [ -n "${ORCA_PATCHELF:-}" ] && [ ! -f "$APPDIR/.orca-interp-patched" ]; then
    (
        flock 9
        if [ ! -f "$APPDIR/.orca-interp-patched" ]; then
            "$ORCA_PATCHELF" --set-interpreter "$ld" "$APPDIR/bin/orca-slicer"
            touch "$APPDIR/.orca-interp-patched"
        fi
    ) 9>"$APPDIR/.orca-interp.lock"
fi

if [ -f "$APPDIR/.orca-interp-patched" ]; then
    LD_LIBRARY_PATH="$native_library_path" LC_ALL=C APPDIR="$APPDIR" exec "$APPDIR/bin/orca-slicer" "$@"
elif [ -n "$ld" ]; then
    LD_LIBRARY_PATH="$native_library_path" LC_ALL=C APPDIR="$APPDIR" exec "$ld" "$APPDIR/bin/orca-slicer" "$@"
else
    LD_LIBRARY_PATH="$native_library_path" LC_ALL=C APPDIR="$APPDIR" exec "$APPDIR/bin/orca-slicer" "$@"
fi
