#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
root=$(mktemp -d /tmp/orca-wrapper-isolation-XXXX)
mkdir -p "$root/source/bin" "$root/libs"
printf '#!/bin/sh\nprintf "%%s\\n" "$APPDIR" "$@"\n' > "$root/source/bin/orca-slicer"
printf '#!/bin/sh\necho loader-must-not-run >&2\nexit 99\n' > "$root/libs/ld-linux-x86-64.so.2"
chmod +x "$root/source/bin/orca-slicer" "$root/libs/ld-linux-x86-64.so.2"
printf '%s' "$root/libs" > "$root/cache"
cp "$root/source/bin/orca-slicer" "$root/original"
printf '%s' "$root" > "$root/host-cache"
env -u ORCA_PATCHELF -u ORCA_JOB_APPDIR ORCA_APPDIR="$root/source" ORCA_LIB_CACHE="$root/host-cache" bash "$repo/scripts/orca-parity.sh" --help > "$root/host.out"
printf '%s\n' "$root/source" --help > "$root/host.expected"
cmp "$root/host.expected" "$root/host.out"
set +e
env -u ORCA_PATCHELF -u ORCA_JOB_APPDIR ORCA_APPDIR="$root/source" ORCA_LIB_CACHE="$root/cache" bash "$repo/scripts/orca-parity.sh" --help > "$root/missing.out" 2> "$root/missing.err"
code=$?
set -e
test "$code" != 0
grep -q 'ORCA_PATCHELF is required' "$root/missing.err"
! grep -q loader-must-not-run "$root/missing.err"
printf '#!/bin/sh\nprintf "# patched\\n" >> "$3"\n' > "$root/patchelf"
chmod +x "$root/patchelf"
env ORCA_PATCHELF="$root/patchelf" ORCA_JOB_APPDIR="$root/job" ORCA_APPDIR="$root/source" ORCA_LIB_CACHE="$root/cache" bash "$repo/scripts/orca-parity.sh" --slice 0 'model with spaces.3mf' > "$root/patched.out"
printf '%s\n' "$root/job" --slice 0 'model with spaces.3mf' > "$root/expected"
cmp "$root/expected" "$root/patched.out"
cmp "$root/original" "$root/source/bin/orca-slicer"
grep -q '# patched' "$root/job/bin/orca-slicer"
test ! -e "$root/source/.orca-interp-patched"
set +e
env ORCA_PATCHELF="$root/patchelf" ORCA_JOB_APPDIR="$root/source" ORCA_APPDIR="$root/source" ORCA_LIB_CACHE="$root/cache" bash "$repo/scripts/orca-parity.sh" --help > "$root/shared.out" 2> "$root/shared.err"
code=$?
set -e
test "$code" != 0
grep -q 'ORCA_JOB_APPDIR must be outside' "$root/shared.err"
cmp "$root/original" "$root/source/bin/orca-slicer"
echo "PASS $root"
