#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
: "${PLUGIN_SPIKE_OUT:?set an absolute artifact directory}"
: "${CARGO_TARGET_DIR:?set an absolute cargo target directory}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
mkdir -p "$PLUGIN_SPIKE_OUT"
for item in staggered subdivision; do
    features=()
    if [[ "$item" == subdivision ]]; then features=(--features subdivide); fi
    cargo build --locked --release --target wasm32-unknown-unknown -p plugin-spike-guest "${features[@]}"
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/plugin_spike_guest.wasm" "$PLUGIN_SPIKE_OUT/$item.wasm"
    cargo build --locked --release --target wasm32-unknown-unknown -p plugin-spike-guest "${features[@]}" --features alternate
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/plugin_spike_guest.wasm" "$PLUGIN_SPIKE_OUT/$item-alternate.wasm"
done
cargo build --locked --release --target wasm32-wasip1 -p plugin-spike-guest --features boa
cp "$CARGO_TARGET_DIR/wasm32-wasip1/release/plugin_spike_guest.wasm" "$PLUGIN_SPIKE_OUT/boa.wasm"
export CC_wasm32_unknown_unknown="${CC_wasm32_unknown_unknown:-clang}"
cargo build --locked --release --target wasm32-unknown-unknown -p plugin-spike-guest --features quickjs
cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/plugin_spike_guest.wasm" "$PLUGIN_SPIKE_OUT/quickjs.wasm"
cargo build --locked --release --target wasm32-wasip1 -p plugin-spike-files-guest --features wasi
cp "$CARGO_TARGET_DIR/wasm32-wasip1/release/plugin_spike_files_guest.wasm" "$PLUGIN_SPIKE_OUT/files-wasi.wasm"
cargo build --locked --release --target wasm32-unknown-unknown -p plugin-spike-files-guest
cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/plugin_spike_files_guest.wasm" "$PLUGIN_SPIKE_OUT/files-custom.wasm"
cargo build --locked --release -p plugin-spike-host --features jit,direct-boa,direct-quickjs,files
"$CARGO_TARGET_DIR/release/bench-plugin" fixture "$PLUGIN_SPIKE_OUT"
cp plugin.js "$PLUGIN_SPIKE_OUT/staggered.js"
sed 's/const oddAdjustment = 0.0;/const oddAdjustment = 0.25;/' plugin.js > "$PLUGIN_SPIKE_OUT/staggered-alternate.js"
sed 's/const subdivide = false;/const subdivide = true;/' plugin.js > "$PLUGIN_SPIKE_OUT/subdivision.js"
sed 's/const lowerScale = 1.0;/const lowerScale = 0.5;/' "$PLUGIN_SPIKE_OUT/subdivision.js" > "$PLUGIN_SPIKE_OUT/subdivision-alternate.js"
{ printf '%s\n' '// Built-in math'; sed '/if (Number.isNaN(value)) return/d' plugin.js; } > "$PLUGIN_SPIKE_OUT/raw-math.js"
