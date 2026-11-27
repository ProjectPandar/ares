#!/usr/bin/env bash
set -euo pipefail
: "${PLUGIN_SPIKE_OUT:?set an absolute artifact directory}"
: "${CARGO_TARGET_DIR:?set an absolute cargo target directory}"
cd -- "$PLUGIN_SPIKE_OUT"
bench="$CARGO_TARGET_DIR/release/bench-plugin"
failed=0
capture() {
    local name=$1
    shift
    printf '%s\n' "$name"
    if ! "$bench" "$@" > "$name"; then
        printf '%s failed\n' "$name" >&2
        failed=1
    fi
}
for kind in staggered subdivision; do
    export PLUGIN_SPIKE_KIND="$kind"
    capture "native-direct-rust-$kind.jsonl" rust - - -
    for guest in quickjs boa; do
        capture "native-direct-$guest-$kind.jsonl" "$guest" - "$kind-alternate.js" "$kind.js"
    done
    for host in interpreter jit; do
        for guest in rust quickjs boa; do
            if [[ "$guest" == rust ]]; then
                args=("$kind.wasm" "$kind-alternate.wasm" -)
            else
                args=("$guest.wasm" "$kind-alternate.js" "$kind.js")
            fi
            capture "native-$host-$guest-$kind.jsonl" "$host" "${args[@]}"
        done
    done
done
export PLUGIN_SPIKE_KIND=staggered
for host in interpreter jit; do
    PLUGIN_SPIKE_METERED=1 capture "native-metered-$host-rust-staggered.jsonl" "$host" staggered.wasm staggered-alternate.wasm -
done
if ! "$bench" files "$PLUGIN_SPIKE_OUT"; then failed=1; fi
capture native-direct-limits.json direct-limits
export PLUGIN_SPIKE_LAYERS=1 PLUGIN_SPIKE_TRIALS=1
for guest in quickjs boa; do
    capture "native-raw-math-jit-$guest.jsonl" jit "$guest.wasm" staggered-alternate.js raw-math.js
    capture "native-raw-math-direct-$guest.jsonl" "$guest" - staggered-alternate.js raw-math.js
done
exit "$failed"
