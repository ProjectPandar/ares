#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=scripts/ci/toolchain-image.env
source "${SCRIPT_DIR}/toolchain-image.env"

compute_hash() {
  (
    cd "${REPO_ROOT}"
    {
      cat scripts/ci/toolchain-image.env
      cat scripts/ci/ci-toolchain.Dockerfile
      cat rust-toolchain.toml
    } | sha256sum | cut -c1-12
  )
}

usage() {
  cat >&2 <<'EOF'
Usage:
  toolchain-image.sh hash
  toolchain-image.sh tag <base|browsers>
  toolchain-image.sh ref <base|browsers>
  toolchain-image.sh verify-pins
EOF
  exit 2
}

cmd="${1:-}"
case "$cmd" in
  hash)
    compute_hash
    ;;
  tag)
    variant="${2:-}"
    case "$variant" in
      base|browsers) echo "${variant}-$(compute_hash)" ;;
      *) usage ;;
    esac
    ;;
  ref)
    variant="${2:-}"
    case "$variant" in
      base|browsers) echo "${TOOLCHAIN_IMAGE_REPO}:${variant}-$(compute_hash)" ;;
      *) usage ;;
    esac
    ;;
  verify-pins)
    lock_wbg="$(grep -A1 'name = "wasm-bindgen"' "${REPO_ROOT}/Cargo.lock" | grep -m1 'version' | cut -d'"' -f2)"
    test "${lock_wbg}" = "${WASM_BINDGEN_VERSION}" || {
      echo "wasm-bindgen pin mismatch: Cargo.lock=${lock_wbg} toolchain-image.env=${WASM_BINDGEN_VERSION}" >&2
      exit 1
    }
    tc_rust="$(grep -m1 'channel' "${REPO_ROOT}/rust-toolchain.toml" | cut -d'"' -f2)"
    test "${tc_rust}" = "${RUST_VERSION}" || {
      echo "Rust pin mismatch: rust-toolchain.toml=${tc_rust} toolchain-image.env=${RUST_VERSION}" >&2
      exit 1
    }
    pw_ver="$(grep -m1 '"@playwright/test"' "${REPO_ROOT}/web/package.json" | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^[^0-9]*//')"
    test "${pw_ver}" = "${PLAYWRIGHT_VERSION}" || {
      echo "Playwright pin mismatch: web/package.json=${pw_ver} toolchain-image.env=${PLAYWRIGHT_VERSION}" >&2
      exit 1
    }
    echo "All toolchain pins match repo manifests (rust=${RUST_VERSION}, wasm-bindgen=${WASM_BINDGEN_VERSION}, playwright=${PLAYWRIGHT_VERSION}, bun=${BUN_CANARY_REVISION})."
    ;;
  *)
    usage
    ;;
esac
