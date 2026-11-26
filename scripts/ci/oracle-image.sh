#!/usr/bin/env bash
# Builds, tags, and prunes the versioned GHCR OCI image carrying the pinned
# OrcaSlicer v2.5.0 Nix oracle closure + preset catalog so daily CI shards run
# without installing Nix.
set -euo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=scripts/ci/oracle-image.env
source "$repo/scripts/ci/oracle-image.env"

compute_tag() {
  local hash
  hash="$(
    {
      printf 'orca_ref=%s\nnixpkgs=%s\n' "$ORCA_REF" "$ORCA_NIXPKGS_REV"
      (
        cd "$repo"
        find scripts/ci/build-oracle.sh scripts/create-project-preset-catalog.mjs \
          tools/chain-probe/orca-release \
          tools/chain-probe/merged-oracle.nix \
          tools/chain-probe/merged-oracle-extra.patch \
          tools/chain-probe/seed-replay.nix \
          tools/chain-probe/plate-custom-gcodes.nix \
          tools/chain-probe/plate-custom-gcodes.patch \
          tools/chain-probe/preheat-move-durations.nix \
          tools/chain-probe/preheat-duration-oracle.patch \
          tools/chain-probe/segment-colors.nix \
          tools/chain-probe/segment-colors.patch \
          tools/chain-probe/segment-colors-oracle.hpp \
          tools/chain-probe/segment-colors \
          tools/chain-probe/preset-select.nix \
          tools/chain-probe/preset-select.patch \
          tools/chain-probe/procgcode.patch \
          tools/chain-probe/procgcode-dynamics.patch \
          tools/chain-probe/marker-time.patch \
          tools/chain-probe/marker-time-oracle.hpp \
          tools/chain-probe/marker-time \
          tools/chain-probe/print-statistics.patch \
          tools/chain-probe/actual-speed-profile-oracle.hpp \
          tools/chain-probe/actual-speed-profile \
          tools/chain-probe/custom-gcode-overview-oracle.hpp \
          tools/chain-probe/custom-gcode-overview \
          tools/chain-probe/role-times-oracle.hpp \
          tools/chain-probe/role-times \
          tools/chain-probe/segment-hw-oracle.hpp \
          tools/chain-probe/segment-hw \
          -type f 2>/dev/null | LC_ALL=C sort | while IFS= read -r rel; do
            printf '%s  %s\n' "$(sha256sum "$rel" | awk '{print $1}')" "$rel"
          done
      )
    } | sha256sum | cut -c1-12
  )"
  printf 'v2.5.0-%s\n' "$hash"
}

image_ref() {
  local tag
  tag="${ORACLE_IMAGE_TAG:-$(compute_tag)}"
  if [ -n "${ORACLE_IMAGE_DIGEST:-}" ]; then
    printf '%s:%s@%s\n' "$ORACLE_IMAGE_REPO" "$tag" "$ORACLE_IMAGE_DIGEST"
  else
    printf '%s:%s\n' "$ORACLE_IMAGE_REPO" "$tag"
  fi
}

stage_rootfs() {
  local variants_json="" profiles_dir="" out_dir="" skip_closure="0"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --variants-json) variants_json="$2"; shift 2 ;;
      --profiles) profiles_dir="$2"; shift 2 ;;
      --out) out_dir="$2"; shift 2 ;;
      --skip-nix-closure) skip_closure="1"; shift ;;
      *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
  done
  [ -f "$variants_json" ] || { echo "missing --variants-json" >&2; exit 2; }
  [ -d "$profiles_dir" ] || { echo "missing --profiles" >&2; exit 2; }
  [ -n "$out_dir" ] || { echo "missing --out" >&2; exit 2; }

  rm -rf "$out_dir"
  mkdir -p "$out_dir/nix/store" "$out_dir/opt/ares-oracles"

  # Generate read-only immutable preset catalog (source-r3).
  node "$repo/scripts/create-project-preset-catalog.mjs" \
    --profiles "$profiles_dir" \
    --out "$out_dir/opt/ares-oracles/source-r3"
  chmod -R a-w "$out_dir/opt/ares-oracles/source-r3/native"

  # Copy deduplicated Nix runtime closure and hardlink identical share/OrcaSlicer files.
  python3 - "$variants_json" "$out_dir" "$ORCA_REF" "$ORCA_NIXPKGS_REV" "$skip_closure" <<'PYEOF'
import hashlib, json, os, shutil, subprocess, sys
from pathlib import Path

variants_path, out_root, orca_ref, nixpkgs_rev, skip_closure = sys.argv[1:6]
out_root = Path(out_root)
spec = json.loads(Path(variants_path).read_text())

roots = []
for key in ("merged-oracle", "seed-replay", "plate-custom-gcodes", "preheat-move-durations",
            "segment-colors", "preset-select", "gdb", "patchelf", "glibcLocales"):
    if key in spec["variants"]:
        roots.append(spec["variants"][key])
roots.extend(spec.get("official_libs", []))

if skip_closure == "1":
    closure = sorted(set(roots))
else:
    closure = subprocess.check_output(["nix", "path-info", "-r", *sorted(set(roots))], text=True).splitlines()

for store_path in closure:
    src = Path(store_path)
    if not src.exists():
        raise SystemExit(f"missing store path: {store_path}")
    rel = store_path.lstrip("/")
    dst = out_root / rel
    if dst.exists() or dst.is_symlink():
        continue
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.is_symlink():
        os.symlink(os.readlink(src), dst)
    elif src.is_dir():
        shutil.copytree(src, dst, symlinks=True)
    else:
        shutil.copy2(src, dst)

# Hardlink byte-identical files across share/OrcaSlicer trees to cut OCI layer & disk size.
seen_resources = {}
dedup_bytes = 0
for key in ("merged-oracle", "seed-replay", "plate-custom-gcodes", "preheat-move-durations",
            "segment-colors", "preset-select"):
    root = spec["variants"].get(key)
    if not root:
        continue
    share_dir = out_root / root.lstrip("/") / "share" / "OrcaSlicer"
    if not share_dir.is_dir():
        continue
    for dirpath, _, filenames in os.walk(share_dir):
        for fname in filenames:
            fpath = Path(dirpath) / fname
            if fpath.is_symlink() or not fpath.is_file():
                continue
            data = fpath.read_bytes()
            key_hash = (len(data), hashlib.sha256(data).hexdigest())
            prev = seen_resources.get(key_hash)
            if prev is None:
                seen_resources[key_hash] = fpath
            else:
                parent_mode = fpath.parent.stat().st_mode
                os.chmod(fpath.parent, parent_mode | 0o200)
                try:
                    os.unlink(fpath)
                    os.link(prev, fpath)
                    dedup_bytes += len(data)
                finally:
                    os.chmod(fpath.parent, parent_mode)

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

checksums = {}
for key, store_path in sorted(spec["variants"].items()):
    base = out_root / store_path.lstrip("/")
    for rel in ("bin/orca-slicer", "bin/.orca-slicer-wrapped", "bin/gdb", "bin/patchelf"):
        candidate = base / rel
        if candidate.is_file() and not candidate.is_symlink():
            checksums[f"{store_path}/{rel}"] = sha256_file(candidate)

catalog_file = out_root / "opt/ares-oracles/source-r3/catalog.json"
catalog_sha = sha256_file(catalog_file)
checksums["/opt/ares-oracles/source-r3/catalog.json"] = catalog_sha

locale_archive = f"{spec['variants']['glibcLocales']}/lib/locale/locale-archive"
official_lib_dirs = [f"{p}/lib" for p in spec.get("official_libs", [])]

manifest = {
    "schema": "ares-ci-oracles-v1",
    "orca_ref": orca_ref,
    "nixpkgs_rev": nixpkgs_rev,
    "variants": spec["variants"],
    "official_libs": ":".join(official_lib_dirs),
    "locale_archive": locale_archive,
    "preset_catalog": {
        "dir": "/opt/ares-oracles/source-r3",
        "catalog": "/opt/ares-oracles/source-r3/catalog.json",
        "catalog_sha256": catalog_sha,
    },
    "sha256": checksums,
    "closure_paths": len(closure),
    "hardlink_dedup_bytes": dedup_bytes,
}
(out_root / "opt/ares-oracles/manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
print(f"staged {len(closure)} store paths, deduped {dedup_bytes / 1048576:.1f} MiB, {len(checksums)} verified files")
PYEOF
}

build_image() {
  local staging="" image="" source_url="https://github.com/ProjectPandar/ares"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --staging) staging="$2"; shift 2 ;;
      --image) image="$2"; shift 2 ;;
      --source-url) source_url="$2"; shift 2 ;;
      *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
  done
  [ -d "$staging/opt/ares-oracles" ] || { echo "invalid --staging directory" >&2; exit 2; }
  [ -n "$image" ] || { echo "missing --image" >&2; exit 2; }

  cat > "$staging/Dockerfile" <<EOF
FROM scratch
LABEL org.opencontainers.image.source="$source_url"
LABEL org.opencontainers.image.description="Pinned OrcaSlicer v2.5.0 CI oracle closure for Ares"
COPY nix /nix
COPY opt /opt
EOF
  docker buildx build --provenance=false --sbom=false --load -f "$staging/Dockerfile" -t "$image" "$staging"
}

prune_ghcr() {
  local owner="" package="ares/ci-oracles" keep="3" keep_tag="" keep_digest=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --owner) owner="$2"; shift 2 ;;
      --package) package="$2"; shift 2 ;;
      --keep) keep="$2"; shift 2 ;;
      --keep-tag) keep_tag="$2"; shift 2 ;;
      --keep-digest) keep_digest="$2"; shift 2 ;;
      *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
  done
  [ -n "$owner" ] || { echo "missing --owner" >&2; exit 2; }
  local encoded
  encoded="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$package")"
  local endpoint="/orgs/$owner/packages/container/$encoded/versions"
  if ! versions="$(gh api --paginate "$endpoint" 2>/dev/null)"; then
    endpoint="/users/$owner/packages/container/$encoded/versions"
    versions="$(gh api --paginate "$endpoint")"
  fi
  VERSIONS_JSON="$versions" python3 - "$keep" "$keep_tag" "$keep_digest" <<'PYEOF' | while IFS= read -r vid; do
import json, os, sys
keep = int(sys.argv[1])
keep_tag = sys.argv[2]
keep_digest = sys.argv[3]
raw = os.environ.get("VERSIONS_JSON", "").strip()
items = sorted(json.loads(raw) if raw else [], key=lambda x: x["created_at"], reverse=True)
kept = 0
for item in items:
    tags = item.get("metadata", {}).get("container", {}).get("tags", [])
    if keep_digest and item.get("name") == keep_digest:
        kept += 1
        continue
    if keep_tag and keep_tag in tags:
        kept += 1
        continue
    if not tags:
        # Never blindly delete untagged manifests: multi-manifest OCI indexes
        # reference untagged child platform/attestation manifests by digest.
        continue
    if kept < keep:
        kept += 1
        continue
    print(item["id"])
PYEOF
    [ -n "$vid" ] || continue
    echo "Deleting stale GHCR version $vid"
    gh api --method DELETE "$endpoint/$vid" || true
  done
}

cmd="${1:-}"
shift || true
case "$cmd" in
  tag) compute_tag ;;
  ref) image_ref ;;
  stage) stage_rootfs "$@" ;;
  build) build_image "$@" ;;
  prune) prune_ghcr "$@" ;;
  *) echo "usage: oracle-image.sh <tag|ref|stage|build|prune> ..." >&2; exit 2 ;;
esac
