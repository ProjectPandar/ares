#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT/scripts/ci/web-build-artifact.sh"
RUN="$(mktemp -d "$ROOT/target/web-build-artifact-test.XXXXXX")"
SHIPPED="$ROOT/web/packages/ares-wasm-js/src/wasm"
PROBE="$ROOT/target/web-e2e/panic-probe"
cleanup() {
  if [ -d "$SHIPPED" ]; then find "$SHIPPED" -mindepth 1 -delete; fi
  if [ -d "$PROBE" ]; then find "$PROBE" -mindepth 1 -delete; fi
  rmdir "$SHIPPED" "$PROBE" 2>/dev/null || true
  find "$RUN" -mindepth 1 -delete 2>/dev/null || true
  rmdir "$RUN" 2>/dev/null || true
}
trap cleanup EXIT

mkdir -p "$SHIPPED" "$PROBE"
printf 'synthetic shipped output\n' > "$SHIPPED/shipped.js"
printf 'synthetic probe output\n' > "$PROBE/probe.js"
GITHUB_OUTPUT="$RUN/create.out" bash "$SCRIPT" create --output "$RUN/artifact"
find "$SHIPPED" -mindepth 1 -delete
find "$PROBE" -mindepth 1 -delete
GITHUB_OUTPUT="$RUN/hit.out" bash "$SCRIPT" restore --workspace "$ROOT" --artifact-dir "$RUN/artifact"
grep -qx 'wasm-hit=true' "$RUN/hit.out"
grep -qx 'panic-probe-hit=true' "$RUN/hit.out"
test -f "$SHIPPED/shipped.js"
test -f "$PROBE/probe.js"

cp -a "$RUN/artifact" "$RUN/tree-mismatch"
sed -i 's/^tree_sha=.*/tree_sha=0000000000000000000000000000000000000000/' "$RUN/tree-mismatch/manifest"
find "$SHIPPED" -mindepth 1 -delete
find "$PROBE" -mindepth 1 -delete
GITHUB_OUTPUT="$RUN/tree.out" bash "$SCRIPT" restore --workspace "$ROOT" --artifact-dir "$RUN/tree-mismatch"
grep -qx 'wasm-hit=false' "$RUN/tree.out"
grep -qx 'panic-probe-hit=false' "$RUN/tree.out"
test ! -e "$SHIPPED/shipped.js"
test ! -e "$PROBE/probe.js"

cp -a "$RUN/artifact" "$RUN/recipe-mismatch"
sed -i 's/^run_web_build_sha256=.*/run_web_build_sha256=0000000000000000000000000000000000000000000000000000000000000000/' "$RUN/recipe-mismatch/manifest"
GITHUB_OUTPUT="$RUN/recipe.out" bash "$SCRIPT" restore --workspace "$ROOT" --artifact-dir "$RUN/recipe-mismatch"
grep -qx 'wasm-hit=false' "$RUN/recipe.out"
grep -qx 'panic-probe-hit=false' "$RUN/recipe.out"
test ! -e "$SHIPPED/shipped.js"
test ! -e "$PROBE/probe.js"

MOCK_BIN="$RUN/mock-bin"
mkdir -p "$MOCK_BIN"
TREE="$(git -C "$ROOT" rev-parse 'HEAD^{tree}')"
BASE=1111111111111111111111111111111111111111
QUEUE_REF=gh-readonly-queue/main/pr-42-2222222222222222222222222222222222222222
cat > "$MOCK_BIN/gh" <<'EOF'
#!/usr/bin/env bash
set -eu
endpoint=""
for arg in "$@"; do
  case "$arg" in
    /repos/*) endpoint="$arg" ;;
  esac
done
case "$endpoint" in
  */pulls/42)
    printf '%s\n' '{"base":{"sha":"1111111111111111111111111111111111111111"},"head":{"sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","repo":{"full_name":"example/repo"}}}'
    ;;
  */git/commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa)
    printf '%s\n' "$MOCK_TREE"
    ;;
  */pulls/42/runs\?per_page=100)
    printf '%s\n' '[{"workflow_runs":[{"id":99,"name":"Tier 1","status":"in_progress","conclusion":null}]}]'
    ;;
  *)
    printf 'unexpected endpoint: %s\n' "$endpoint" >&2
    exit 1
    ;;
esac
EOF
chmod +x "$MOCK_BIN/gh"
find "$SHIPPED" -mindepth 1 -delete
find "$PROBE" -mindepth 1 -delete
PATH="$MOCK_BIN:$PATH" MOCK_TREE="$TREE" GITHUB_OUTPUT="$RUN/in-progress.out" GITHUB_EVENT_NAME=merge_group GITHUB_REF_NAME="$QUEUE_REF" GITHUB_REPOSITORY=example/repo GITHUB_EVENT_MERGE_GROUP_BASE_SHA="$BASE" GH_TOKEN=fixture RUNNER_TEMP="$RUN" bash "$SCRIPT" restore --workspace "$ROOT"
grep -qx 'wasm-hit=false' "$RUN/in-progress.out"
grep -qx 'panic-probe-hit=false' "$RUN/in-progress.out"
test ! -e "$SHIPPED/shipped.js"
test ! -e "$PROBE/probe.js"
printf 'web-build-artifact behavior checks passed\n'
