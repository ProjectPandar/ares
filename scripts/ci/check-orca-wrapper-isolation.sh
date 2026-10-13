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

# setarch determinism: the wrapper must disable ASLR at the Native exec site
# without leaking LD_LIBRARY_PATH into setarch itself.
command -v setarch >/dev/null 2>&1 || { echo 'setarch (util-linux) missing on this runner' >&2; exit 1; }
mkdir -p "$root/shim-bin"
printf '#!/bin/sh\nprintf "%%s\\n" "LDLP=${LD_LIBRARY_PATH:-UNSET}" "$*" >"$SETARCH_RECORD"\nwhile [ "$1" = "-R" ]; do shift; done\nexec "$@"\n' > "$root/shim-bin/setarch"
chmod +x "$root/shim-bin/setarch"
SETARCH_RECORD="$root/setarch.record" PATH="$root/shim-bin:$PATH" \
    env -u ORCA_PATCHELF -u ORCA_JOB_APPDIR ORCA_APPDIR="$root/source" ORCA_LIB_CACHE="$root/host-cache" \
    bash "$repo/scripts/orca-parity.sh" --help > "$root/setarch.out"
printf '%s\n' "$root/source" --help > "$root/setarch.expected"
cmp "$root/setarch.expected" "$root/setarch.out"
grep -q '^LDLP=UNSET$' "$root/setarch.record"
grep -q -- '-R env' "$root/setarch.record"

# A runner without setarch must fail loudly instead of running the oracle
# with ASLR enabled.
mkdir -p "$root/minimal-bin"
for tool in cat env; do ln -s "$(command -v "$tool")" "$root/minimal-bin/$tool"; done
set +e
env -u ORCA_PATCHELF -u ORCA_JOB_APPDIR ORCA_APPDIR="$root/source" ORCA_LIB_CACHE="$root/host-cache" \
    PATH="$root/minimal-bin" "$(command -v bash)" "$repo/scripts/orca-parity.sh" --help > "$root/nosetarch.out" 2> "$root/nosetarch.err"
code=$?
set -e
test "$code" != 0
grep -q 'setarch (util-linux) is required' "$root/nosetarch.err"
echo "PASS $root"
