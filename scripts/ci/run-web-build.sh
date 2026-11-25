#!/usr/bin/env bash
set -euo pipefail

# Builds release CLI, verifies PrintConfigDef metadata, builds WASM + web app +
# lint/fmt/typecheck/unit tests, and packs the prebuilt bundle for web E2E shards.

TMP_ROOT="${RUNNER_TEMP:-/tmp}"
cargo build --release -p ares-cli --bin ares -p ares-core --example print_config_def

bun scripts/gen-configdef.mjs --check
./target/release/examples/print_config_def > target/print-config-def.json
bun scripts/verify-configdef.mjs target/print-config-def.json --report target/configdef-verification.json

cargo nextest run -p ares-wasm --test compatible_presets --test decoded_settings --test encode_roundtrip
bun web/scripts/build-panic-probe.mjs

bun run --cwd web build:wasm
bun run --cwd web build
bun run --cwd web fmt:check
bun run --cwd web lint
bun run --cwd web typecheck
mkdir -p "${TMP_ROOT}/profile-tests"
ORCA_SLICER_DIR="$PWD/OrcaSlicer" \
ARES_PROFILE_TEST_ARTIFACTS="${TMP_ROOT}/profile-tests" \
  bun run --cwd web test:unit

mkdir -p "${TMP_ROOT}/web-bundle"
tar --zstd -cf "${TMP_ROOT}/web-bundle/web-dist.tar.zst" \
  target/release/ares \
  target/release/examples/print_config_def \
  target/web-e2e \
  web/packages/ares-wasm-js/dist \
  web/packages/ares-profiles/dist \
  web/app/dist
ls -lh "${TMP_ROOT}/web-bundle/web-dist.tar.zst"
