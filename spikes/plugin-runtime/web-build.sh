#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/../.."
: "${PLUGIN_SPIKE_OUT:?set an absolute artifact directory}"
: "${CARGO_TARGET_DIR:?set an absolute cargo target directory}"
export CC_wasm32_unknown_unknown="${CC_wasm32_unknown_unknown:-clang}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
mkdir -p "$PLUGIN_SPIKE_OUT"
for engine in interpreter compiled-rust quickjs boa files baseline; do
    feature=(--features "spike-plugin-$engine")
    case $engine in
        interpreter) name=ares_wasm ;;
        compiled-rust) name=direct_rust ;;
        files) name=files_host ;;
        baseline) name=baseline; feature=() ;;
        *) name="direct_$engine" ;;
    esac
    cargo build --locked --release --target wasm32-unknown-unknown -p ares-wasm "${feature[@]}"
    wasm-bindgen --target web --omit-default-module-path --out-dir "$PLUGIN_SPIKE_OUT" --out-name "$name" \
        "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/ares_wasm.wasm"
done
