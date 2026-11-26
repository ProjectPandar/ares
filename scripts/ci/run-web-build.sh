#!/usr/bin/env bash
set -euo pipefail

# Build the host tools, verify PrintConfigDef metadata, and pack the browser
# test bundle. Fast checks handle format, lint, unit tests and metadata sync.

CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
  cargo build --release -p ares-cli --bin ares -p ares-core --example print_config_def

./target/release/examples/print_config_def > target/print-config-def.json
bun scripts/verify-configdef.mjs target/print-config-def.json --report target/configdef-verification.json

# The browser specs consume files written by these tests, even if another job runs them.
CARGO_PROFILE_DEV_DEBUG=0 cargo nextest run --cargo-profile dev \
  -p ares-wasm --test compatible_presets --test decoded_settings --test encode_roundtrip

if [[ ${WASM_CACHE_HIT:-} != true ]]; then
  rm -rf web/packages/ares-wasm-js/src/wasm
  bun run --cwd web build:wasm
fi
if [[ ${PANIC_PROBE_CACHE_HIT:-} != true ]]; then
  rm -rf target/web-e2e/panic-probe
  bun web/scripts/build-panic-probe.mjs
fi

bun run --cwd web build
bun run --cwd web typecheck

mkdir -p target/ci-web-bundle
tar --zstd -cf target/ci-web-bundle/web-dist.tar.zst \
  target/release/ares \
  target/release/examples/print_config_def \
  target/web-e2e \
  web/packages/ares-wasm-js/src/wasm \
  web/packages/ares-wasm-js/dist \
  web/packages/ares-profiles/dist \
  web/app/dist
ls -lh target/ci-web-bundle/web-dist.tar.zst
