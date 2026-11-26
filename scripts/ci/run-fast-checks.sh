#!/usr/bin/env bash
set -euo pipefail

# Runs all small/fast checks in one job (overlapping rust-fmt with web/script checks),
# lists EVERY failure (never stops at the first failure), and enforces strict latency budgets.

BUDGET_WEB_FMT_MS="${BUDGET_WEB_FMT_MS:-1000}"
BUDGET_WEB_LINT_MS="${BUDGET_WEB_LINT_MS:-1000}"
BUDGET_WEB_CHECK_MS="${BUDGET_WEB_CHECK_MS:-5000}"
BUDGET_WEB_UNIT_MS="${BUDGET_WEB_UNIT_MS:-5000}"
BUDGET_CONFIGDEF_MS="${BUDGET_CONFIGDEF_MS:-2000}"
BUDGET_SCRIPTS_MS="${BUDGET_SCRIPTS_MS:-2000}"
BUDGET_RUST_FMT_MS="${BUDGET_RUST_FMT_MS:-15000}"
BUDGET_TOTAL_RUN_MS="${BUDGET_TOTAL_RUN_MS:-20000}"

now_ms() { date +%s%3N; }

run_start_ms="$(now_ms)"
failures=()
results=()

# Pin RUSTUP_TOOLCHAIN so rustup does not auto-download clippy/wasm32 components
# declared in rust-toolchain.toml during fmt/gen-configdef checks.
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-1.99.0}"

record_result() {
  local name="$1" budget_ms="$2" rc="$3" dt="$4" log_file="$5"
  if [ "$rc" -ne 0 ]; then
    echo "::error::[FAIL] ${name} exited ${rc} (${dt}ms)"
    cat "$log_file"
    failures+=("${name}: exit=${rc} (${dt}ms)")
    results+=("FAIL  ${name}  ${dt}ms (budget ${budget_ms}ms)")
  elif [ "$dt" -gt "$budget_ms" ]; then
    echo "::error::[BUDGET EXCEEDED] ${name} took ${dt}ms > budget ${budget_ms}ms"
    cat "$log_file"
    failures+=("${name}: ${dt}ms exceeded budget ${budget_ms}ms")
    results+=("OVER  ${name}  ${dt}ms (budget ${budget_ms}ms)")
  else
    echo "[PASS] ${name}: ${dt}ms (budget ${budget_ms}ms)"
    results+=("PASS  ${name}  ${dt}ms (budget ${budget_ms}ms)")
  fi
}

run_check() {
  local name="$1" budget_ms="$2"
  shift 2
  local t0 t1 dt rc log_file
  log_file="$(mktemp)"
  t0="$(now_ms)"
  set +e
  "$@" >"$log_file" 2>&1
  rc=$?
  set -e
  t1="$(now_ms)"
  dt=$((t1 - t0))
  record_result "$name" "$budget_ms" "$rc" "$dt" "$log_file"
  rm -f "$log_file"
}

# Run tightly budgeted web fmt, lint, and TS7 typecheck with dedicated CPU before
# spawning background cargo fmt (avoids 4-vCPU contention spiking lint/check over budget).
run_check "web-fmt" "$BUDGET_WEB_FMT_MS" bun --cwd web fmt --no-script --check packages app e2e pages
run_check "web-lint" "$BUDGET_WEB_LINT_MS" bun --cwd web lint --no-script --max-warnings=0 packages app e2e pages

if [ -f web/packages/ares-wasm-js/src/wasm/ares_wasm.d.ts ]; then
  run_check "web-typecheck-ts7" "$BUDGET_WEB_CHECK_MS" bash -c '
    set -euo pipefail
    web/node_modules/.bin/tsc -p web/packages/ares-wasm-js/tsconfig.build.json
    ORCA_SLICER_DIR="${ORCA_SLICER_DIR:-$PWD/OrcaSlicer}" bun run --cwd web --filter @ares-slicer/profiles build >/dev/null
    bun run --cwd web typecheck
  '
else
  echo "::warning::Skipping full web-typecheck-ts7 in fast-checks: cached src/wasm not yet warmed on main (checked in pages-build)"
  results+=("SKIP  web-typecheck-ts7  0ms (awaiting main wasm-pkg cache warm)")
fi

# Now overlap rust-fmt in the background with unit tests, configdef sync, and CI script checks.
rust_fmt_log="$(mktemp)"
rust_fmt_meta="$(mktemp)"
(
  t0="$(now_ms)"
  set +e
  cargo fmt --check >"$rust_fmt_log" 2>&1
  rc=$?
  set -e
  t1="$(now_ms)"
  printf '%s %s\n' "$rc" "$((t1 - t0))" > "$rust_fmt_meta"
) &
rust_fmt_pid=$!

if [ -d "${ORCA_SLICER_DIR:-OrcaSlicer}/resources/profiles" ]; then
  mkdir -p "${ARES_PROFILE_TEST_ARTIFACTS:-/tmp/ares-profile-tests}"
  run_check "web-unit-tests" "$BUDGET_WEB_UNIT_MS" env \
    ORCA_SLICER_DIR="${ORCA_SLICER_DIR:-$PWD/OrcaSlicer}" \
    ARES_PROFILE_TEST_ARTIFACTS="${ARES_PROFILE_TEST_ARTIFACTS:-/tmp/ares-profile-tests}" \
    bun run --cwd web test:unit
else
  failures+=("web-unit-tests: missing OrcaSlicer/resources/profiles")
  results+=("FAIL  web-unit-tests  missing OrcaSlicer checkout")
fi

if [ -f "OrcaSlicer/src/libslic3r/PrintConfig.cpp" ]; then
  run_check "configdef-sync" "$BUDGET_CONFIGDEF_MS" bun scripts/gen-configdef.mjs --check
elif [ -n "${ORCA_SLICER_DIR:-}" ] && [ -f "${ORCA_SLICER_DIR}/src/libslic3r/PrintConfig.cpp" ]; then
  echo "[SKIP] configdef-sync: local run without ./OrcaSlicer checkout"
  results+=("SKIP  configdef-sync  0ms (local ORCA_SLICER_DIR outside repo)")
else
  failures+=("configdef-sync: missing OrcaSlicer/src/libslic3r/PrintConfig.cpp")
  results+=("FAIL  configdef-sync  missing OrcaSlicer checkout")
fi

run_check "ci-scripts-syntax-and-pins" "$BUDGET_SCRIPTS_MS" bash -c '
  set -euo pipefail
  bash -n scripts/ci/*.sh
  bash scripts/ci/toolchain-image.sh verify-pins
  bun scripts/ci/balance-web-shards.mjs verify --web-dir web --project desktop --total 4
'

wait "$rust_fmt_pid" || true
read -r rust_rc rust_dt < "$rust_fmt_meta"
record_result "rust-fmt" "$BUDGET_RUST_FMT_MS" "$rust_rc" "$rust_dt" "$rust_fmt_log"
rm -f "$rust_fmt_log" "$rust_fmt_meta"

total_ms=$(($(now_ms) - run_start_ms))
echo ""
echo "=== fast-checks summary (${total_ms}ms / budget ${BUDGET_TOTAL_RUN_MS}ms) ==="
printf '  %s\n' "${results[@]}"

if [ "$total_ms" -gt "$BUDGET_TOTAL_RUN_MS" ]; then
  failures+=("total-run: ${total_ms}ms exceeded budget ${BUDGET_TOTAL_RUN_MS}ms")
fi

if [ "${#failures[@]}" -gt 0 ]; then
  echo ""
  echo "=== ALL FAILURES (${#failures[@]}) ===" >&2
  printf '  - %s\n' "${failures[@]}" >&2
  exit 1
fi
