#!/usr/bin/env bash
set -euo pipefail

# Runs one timing-balanced web-desktop Playwright shard and records per-file timings.
SHARD="${1:?usage: run-web-desktop-shard.sh <shard> <total>}"
TOTAL="${2:?missing total}"
TMP_ROOT="${RUNNER_TEMP:-/tmp}"

mkdir -p "${TMP_ROOT}/web-results" "${TMP_ROOT}/web-timings"
mapfile -t specs < <(bun scripts/ci/balance-web-shards.mjs select \
  --web-dir web --project desktop --shard "$SHARD" --total "$TOTAL" \
  --timings "${TMP_ROOT}/web-timings")

# Anchor each spec path so e.g. slice.spec.ts does not substring-match presetSlice.spec.ts.
spec_filters=()
for spec in "${specs[@]}"; do
  spec_filters+=("e2e/${spec//./\\.}$")
done

PLAYWRIGHT_JSON_OUTPUT_NAME="${TMP_ROOT}/web-results/desktop-${SHARD}.json" \
bunx --cwd web playwright test --config playwright.config.ts --project desktop \
  --workers 1 --retries 0 --reporter=line,json \
  --output "${TMP_ROOT}/web-results/desktop-${SHARD}" "${spec_filters[@]}"

bun scripts/ci/balance-web-shards.mjs record \
  --report "${TMP_ROOT}/web-results/desktop-${SHARD}.json" \
  --out "${TMP_ROOT}/web-timings/shard-${SHARD}.json"
