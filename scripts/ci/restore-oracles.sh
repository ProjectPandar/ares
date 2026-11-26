#!/usr/bin/env bash
# Restores pinned OrcaSlicer v2.5.0 CI oracles from a versioned GHCR OCI image
# (zero Nix installation required), verifies every SHA-256 in manifest.json,
# writes ORCA_LIB_CACHE, and exports mode-scoped ARES_*/ORCA_* env vars.
set -euo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=scripts/ci/oracle-image.env
source "$repo/scripts/ci/oracle-image.env"

mode=""
image=""
from_root=""
appdir=""
job_appdir=""
env_file="${GITHUB_ENV:-}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode) mode="$2"; shift 2 ;;
    --image) image="$2"; shift 2 ;;
    --from-root) from_root="$2"; shift 2 ;;
    --appdir) appdir="$2"; shift 2 ;;
    --job-appdir) job_appdir="$2"; shift 2 ;;
    --env-file) env_file="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

case "$mode" in
  official|seeded|web) ;;
  *) echo "usage: restore-oracles.sh --mode <official|seeded|web> [--image <ref> | --from-root <dir>] [--appdir <dir>] [--job-appdir <dir>] [--env-file <path>]" >&2; exit 2 ;;
esac

root_prefix=""
if [ -n "$from_root" ]; then
  root_prefix="$(realpath "$from_root")"
else
  if [ -z "$image" ]; then
    image="$(bash "$repo/scripts/ci/oracle-image.sh" ref)"
  fi
  echo "Pulling CI oracle image: $image"
  docker pull "$image"
  cid="$(docker create "$image" /nonexistent)"
  trap 'docker rm -f "$cid" >/dev/null 2>&1 || true' EXIT
  sudo mkdir -p /nix/store /opt/ares-oracles
  docker export "$cid" | sudo tar -x -C / --no-overwrite-dir nix/store opt/ares-oracles
  docker rm -f "$cid" >/dev/null 2>&1 || true
  trap - EXIT
fi

manifest="${root_prefix}/opt/ares-oracles/manifest.json"
[ -f "$manifest" ] || { echo "missing oracle manifest: $manifest" >&2; exit 1; }

lib_cache="${RUNNER_TEMP:-${TMPDIR:-$repo/target}}/orca-parity-libs.cache"
mkdir -p "$(dirname "$lib_cache")"

python3 - "$manifest" "$root_prefix" "$repo" "$mode" "$appdir" "$job_appdir" "$lib_cache" "$ORCA_REF" <<'PYEOF' > "${env_file:-/dev/stdout}"
import hashlib, json, os, sys
from pathlib import Path

manifest_path, root_prefix, repo, mode, appdir, job_appdir, lib_cache, expected_ref = sys.argv[1:9]
data = json.loads(Path(manifest_path).read_text())

assert data.get("schema") == "ares-ci-oracles-v1", f"unexpected schema: {data.get('schema')}"
assert data.get("orca_ref") == expected_ref, f"orca_ref mismatch: {data.get('orca_ref')} != {expected_ref}"

def resolve(p: str) -> Path:
    return Path(root_prefix + p) if root_prefix else Path(p)

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

# Fail-closed verification of every checksum recorded in manifest.json.
checksums = data.get("sha256", {})
assert checksums, "manifest contains no checksums"
for target, expected in sorted(checksums.items()):
    actual_path = resolve(target)
    if not actual_path.is_file():
        raise SystemExit(f"missing verified oracle file: {actual_path}")
    actual = sha256_file(actual_path)
    if actual != expected:
        raise SystemExit(f"SHA-256 mismatch for {target}: expected {expected}, got {actual}")

official_libs = data["official_libs"]
if root_prefix:
    official_libs = ":".join(str(resolve(d)) for d in official_libs.split(":") if d)
Path(lib_cache).write_text(official_libs)

v = data["variants"]
def vpath(name: str, sub: str = "") -> str:
    base = v[name]
    full = f"{base}/{sub}" if sub else base
    return str(resolve(full))

merged_wrapped_key = f"{v['merged-oracle']}/bin/.orca-slicer-wrapped"
seed_wrapped_key = f"{v['seed-replay']}/bin/.orca-slicer-wrapped"
catalog_path = str(resolve(data["preset_catalog"]["catalog"]))
locale_archive = str(resolve(data["locale_archive"]))

env = {
    "LC_ALL": "C",
    "LOCALE_ARCHIVE": locale_archive,
    "ORCA_LIB_CACHE": lib_cache,
    "ORCA_PATCHELF": vpath("patchelf", "bin/patchelf"),
    "ORCA_REPLAY_SEED": "0",
    "ORCA_REPLAY_BIN": vpath("seed-replay", "bin/orca-slicer"),
    "ARES_PARITY_PROJECT_PRESET_CATALOG": catalog_path,
    "ARES_PROCGCODE_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_PROCGCODE_STABLE_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_PROCGCODE_DYNAMICS_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_PROCGCODE_DYNAMICS_STABLE_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_PROCGCODE_NATIVE_BIN": vpath("merged-oracle", "bin/.orca-slicer-wrapped"),
    "ARES_PROCGCODE_NATIVE_SHA256": checksums[merged_wrapped_key],
    "ARES_PROCGCODE_UPSTREAM_REV": expected_ref,
    "ARES_MARKER_TIME_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_MARKER_TIME_STABLE_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_PRINT_STATISTICS_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_ACTUAL_SPEED_PROFILE_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_CUSTOM_GCODE_OVERVIEW_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_ROLE_TIMES_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_SEGMENT_HW_BIN": vpath("merged-oracle", "bin/orca-slicer"),
    "ARES_PLATE_CUSTOM_GCODES_BIN": vpath("plate-custom-gcodes", "bin/orca-slicer"),
    "ARES_KSR_INITIALIZED_BIN": vpath("seed-replay", "bin/orca-slicer"),
    "ARES_KSR_INITIALIZED_BIN_SHA256": checksums[f"{v['seed-replay']}/bin/orca-slicer"],
    "ARES_KSR_INITIALIZED_NATIVE_BIN": vpath("seed-replay", "bin/.orca-slicer-wrapped"),
    "ARES_KSR_INITIALIZED_NATIVE_SHA256": checksums[seed_wrapped_key],
    "ARES_KSR_INITIALIZED_RESOURCES_DIR": vpath("seed-replay", "share/OrcaSlicer"),
    "ARES_ORCA_SEEDED_BIN": f"{repo}/scripts/orca-seeded-parity.sh",
    "ARES_ORCA_SEEDED_NATIVE_BIN": vpath("seed-replay", "bin/.orca-slicer-wrapped"),
    "ARES_ORCA_SEEDED_NATIVE_SHA256": checksums[seed_wrapped_key],
    "ARES_ORCA_SEEDED_RESOURCES_DIR": vpath("seed-replay", "share/OrcaSlicer"),
}

if mode in ("official", "seeded"):
    env["ARES_GDB_BIN"] = vpath("gdb", "bin/gdb")
    env["ARES_PREHEAT_MOVE_DURATIONS_BIN"] = vpath("preheat-move-durations", "bin/orca-slicer")

if mode == "official":
    env["ARES_PARITY_LANE"] = "official"
    env["ARES_ORCA_BIN"] = f"{repo}/scripts/orca-parity.sh"
    if appdir:
        env["ORCA_APPDIR"] = appdir
        env["ORCA_JOB_APPDIR"] = job_appdir or f"{appdir}-job"
        env["ARES_ORCA_NATIVE_BIN"] = f"{appdir}/bin/orca-slicer"
        env["ARES_ORCA_REFERENCE_BINARY"] = f"{appdir}/bin/orca-slicer"
        env["ARES_ORCA_RESOURCES_DIR"] = f"{appdir}/resources"
elif mode == "seeded":
    env["ARES_PARITY_LANE"] = "seeded"
    env["ARES_PARITY_SEEDED_REFERENCE"] = "1"
    env["ARES_ORCA_BIN"] = f"{repo}/scripts/orca-seeded-parity.sh"
    env["ARES_ORCA_NATIVE_BIN"] = vpath("seed-replay", "bin/.orca-slicer-wrapped")
    env["ARES_ORCA_REFERENCE_BINARY"] = vpath("seed-replay", "bin/.orca-slicer-wrapped")
    env["ARES_ORCA_RESOURCES_DIR"] = vpath("seed-replay", "share/OrcaSlicer")
elif mode == "web":
    env["ARES_REQUIRE_ORCA"] = "1"
    env["ARES_ORCA_BIN"] = f"{repo}/scripts/orca-parity.sh"
    env["ARES_SEGMENT_COLOR_BIN"] = vpath("segment-colors", "bin/orca-slicer")
    env["ARES_ORCA_PRESET_SELECT_BIN"] = vpath("preset-select", "bin/orca-slicer")
    env["ARES_ORCA_FILAMENT_REFRESH_BIN"] = vpath("preset-select", "bin/orca-slicer")
    if appdir:
        env["ORCA_APPDIR"] = appdir
        env["ORCA_JOB_APPDIR"] = job_appdir or f"{appdir}-job"
        env["ARES_ORCA_NATIVE_BIN"] = f"{appdir}/bin/orca-slicer"
        env["ARES_ORCA_REFERENCE_BINARY"] = f"{appdir}/bin/orca-slicer"
        env["ARES_ORCA_RESOURCES_DIR"] = f"{appdir}/resources"

for k, val in sorted(env.items()):
    print(f"{k}={val}")
PYEOF
