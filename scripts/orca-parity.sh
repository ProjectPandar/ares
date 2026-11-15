#!/usr/bin/env bash
# Runs the explicitly selected OrcaSlicer AppImage binary for the parity harness
# by assembling a host LD_LIBRARY_PATH from nixpkgs runtime libraries.
#
# Set ORCA_APPDIR to the explicitly selected extraction directory.
# Never discover arbitrary store builds: these may contain diagnostic patches
# or use a different toolchain. Version equality does not prove byte parity.
# The parity test suite invokes this script through ARES_ORCA_BIN.
set -euo pipefail

if [ -z "${ORCA_APPDIR:-}" ] && [ -n "${ARES_PARITY_ENV:-}" ]; then
    set -a
    source "$ARES_PARITY_ENV"
    set +a
fi
if [ -z "${ORCA_APPDIR:-}" ]; then
    echo "ORCA_APPDIR must name an explicitly selected reference extraction (or set ARES_PARITY_ENV)" >&2
    exit 127
fi
APPDIR="$ORCA_APPDIR"

if [ ! -x "$APPDIR/bin/orca-slicer" ]; then
    echo "orca-slicer not found under $APPDIR (set ORCA_APPDIR)" >&2
    exit 127
fi

# The nix eval is expensive and flaky under repeated invocations; cache the
# resolved library paths across runs (force a refresh with ORCA_LIB_REFRESH=1).
CACHE="${ORCA_LIB_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/ares/orca-parity-250b-libs.cache}"
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
    mkdir -p "$(dirname "$CACHE")"
    printf '%s' "$extra" > "$CACHE"
fi

native_library_path="$APPDIR/lib/orca-runtime:$APPDIR/bin:$extra${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# Nix runtime libraries require their matching loader, but invoking ld.so
# directly breaks upstream program_location() and resource lookup.
ld=""
IFS=: read -r -a libdirs <<< "$extra"
for dir in "${libdirs[@]}"; do
    if [ -x "$dir/ld-linux-x86-64.so.2" ]; then
        ld="$dir/ld-linux-x86-64.so.2"
        break
    fi
done

if [ -n "$ld" ]; then
    if [ -z "${ORCA_PATCHELF:-}" ]; then
        echo "ORCA_PATCHELF is required when Nix runtime libraries need a Nix loader; ld.so execution is forbidden" >&2
        exit 127
    fi
    if [ -z "${ORCA_JOB_APPDIR:-}" ]; then
        echo "ORCA_JOB_APPDIR must name a per-job copy destination; the reference AppDir is read-only" >&2
        exit 127
    fi
    source_dir="$(realpath "$APPDIR")"
    job_dir="$(realpath -m "$ORCA_JOB_APPDIR")"
    case "$job_dir/" in
        "$source_dir/"*)
            echo "ORCA_JOB_APPDIR must be outside the reference AppDir" >&2
            exit 127
            ;;
    esac
    mkdir -p "$(dirname "$job_dir")"
    (
        flock 9
        if [ ! -f "$job_dir/.orca-interp-patched" ]; then
            mkdir -p "$job_dir"
            cp -a --reflink=auto "$source_dir/." "$job_dir/"
            "$ORCA_PATCHELF" --set-interpreter "$ld" "$job_dir/bin/orca-slicer"
            touch "$job_dir/.orca-interp-patched"
        fi
    ) 9>"$job_dir.lock"
    APPDIR="$job_dir"
    native_library_path="$APPDIR/lib/orca-runtime:$APPDIR/bin:$extra${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi

if ! command -v setarch >/dev/null 2>&1; then
    echo "setarch (util-linux) is required to disable ASLR for the Native oracle; install util-linux" >&2
    exit 127
fi

# setarch itself must load system libraries, so it runs with LD_LIBRARY_PATH
# stripped; the inner env restores the AppImage path for orca-slicer only.
exec env -u LD_LIBRARY_PATH setarch -R \
    env LD_LIBRARY_PATH="$native_library_path" LC_ALL=C APPDIR="$APPDIR" \
    "$APPDIR/bin/orca-slicer" "$@"
