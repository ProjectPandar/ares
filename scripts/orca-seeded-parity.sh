#!/usr/bin/env bash
set -euo pipefail

: "${ORCA_REPLAY_BIN:?select the separately built seed-only OrcaSlicer executable}"
: "${ORCA_REPLAY_SEED:?provide an explicit u32 replay seed}"
: "${ORCA_REPLAY_CPU:?select one CPU for the continuous-stream reference}"
case "$ORCA_REPLAY_SEED" in
    ''|*[!0-9]*) echo 'ORCA_REPLAY_SEED must be a u32' >&2; exit 2 ;;
esac
if [ "${#ORCA_REPLAY_SEED}" -gt 10 ] || [ "$ORCA_REPLAY_SEED" -gt 4294967295 ]; then
    echo 'ORCA_REPLAY_SEED must be a u32' >&2
    exit 2
fi
case "$ORCA_REPLAY_CPU" in
    ''|*[!0-9]*) echo 'ORCA_REPLAY_CPU must name one CPU' >&2; exit 2 ;;
esac
if ! command -v setarch >/dev/null 2>&1; then
    echo "setarch (util-linux) is required to disable ASLR for the Native oracle; install util-linux" >&2
    exit 127
fi
exec taskset -c "$ORCA_REPLAY_CPU" setarch -R "$ORCA_REPLAY_BIN" "$@"
