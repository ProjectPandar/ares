#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SHIPPED_REL="web/packages/ares-wasm-js/src/wasm"
PROBE_REL="target/web-e2e/panic-probe"
RETENTION_DAYS=14
ARTIFACT_PREFIX="web-build-wasm"
PRODUCER_WORKFLOW_NAME="Web Build Producer"
PRODUCER_WORKFLOW_REL=".github/workflows/web-build-producer.yml"

usage() {
  cat >&2 <<'EOF'
usage:
  web-build-artifact.sh should-produce [--base <commit>] [--github-output <file>]
  web-build-artifact.sh create --output <directory> [--github-output <file>]
  web-build-artifact.sh restore --workspace <directory> [--artifact-dir <directory>]
EOF
  exit 2
}

emit() {
  local key="$1" value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    printf '%s=%s\n' "$key" "$value" >> "$GITHUB_OUTPUT"
  fi
  if [ -n "${OUTPUT_FILE:-}" ]; then
    printf '%s=%s\n' "$key" "$value" >> "$OUTPUT_FILE"
  fi
}

miss() {
  emit "wasm-hit" false
  emit "panic-probe-hit" false
  echo "web-build artifact: miss ($1)" >&2
  exit 0
}

hash_file() {
  sha256sum "$1" | cut -d' ' -f1
}

manifest_value() {
  local key="$1" file="$2"
  awk -F= -v wanted="$key" '$1 == wanted { print substr($0, index($0, "=") + 1); exit }' "$file"
}

require_manifest_value() {
  local key="$1" expected="$2" file="$3" actual
  actual="$(manifest_value "$key" "$file")"
  [ "$actual" = "$expected" ] || miss "manifest-${key}-mismatch"
}

changed_files() {
  local base="$1"
  git -C "$ROOT" diff --name-only "$base" HEAD
}

should_produce() {
  local base="HEAD^1" output_file=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --base) base="$2"; shift 2 ;;
      --github-output) output_file="$2"; shift 2 ;;
      *) usage ;;
    esac
  done
  OUTPUT_FILE="$output_file"
  local file count=0
  while IFS= read -r file; do
    [ -n "$file" ] || continue
    count=$((count + 1))
    case "$file" in
      docs/*.md|README.md|AGENTS.md|CONTRIBUTING.md|CHANGELOG.md) ;;
      *) emit produce true; return 0 ;;
    esac
  done < <(changed_files "$base")
  if [ "$count" -eq 0 ]; then
    emit produce true
  else
    emit produce false
  fi
}

write_recipe_identity() {
  local output="$1"
  source "$ROOT/scripts/ci/toolchain-image.env"
  cat > "$output/manifest" <<EOF
manifest_version=1
tree_sha=$(git -C "$ROOT" rev-parse 'HEAD^{tree}')
repository=${GITHUB_REPOSITORY:-}
retention_days=${RETENTION_DAYS}
workflow_sha256=$(hash_file "$ROOT/.github/workflows/tier1.yml")
producer_workflow_sha256=$(hash_file "$ROOT/$PRODUCER_WORKFLOW_REL")
run_web_build_sha256=$(hash_file "$ROOT/scripts/ci/run-web-build.sh")
build_wasm_sha256=$(hash_file "$ROOT/web/packages/ares-wasm-js/scripts/build-wasm.mjs")
rust_toolchain_sha256=$(hash_file "$ROOT/rust-toolchain.toml")
toolchain_image_sha256=$(hash_file "$ROOT/scripts/ci/toolchain-image.env")
cargo_lock_sha256=$(hash_file "$ROOT/Cargo.lock")
bun_lock_sha256=$(hash_file "$ROOT/web/bun.lock")
rust_version=${RUST_VERSION}
wasm_bindgen_version=${WASM_BINDGEN_VERSION}
bun_revision=${BUN_CANARY_REVISION}
playwright_version=${PLAYWRIGHT_VERSION}
EOF
}

write_file_hashes() {
  local output="$1" directory="$2" prefix="$3" relative file
  while IFS= read -r -d '' file; do
    relative="${file#"$directory/"}"
    printf '%s  %s\n' "$(hash_file "$file")" "$prefix/$relative" >> "$output/files.sha256"
  done < <(find "$directory" -type f -print0 | sort -z)
}

create_artifact() {
  local output="" output_file=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --output) output="$2"; shift 2 ;;
      --github-output) output_file="$2"; shift 2 ;;
      *) usage ;;
    esac
  done
  [ -n "$output" ] || usage
  OUTPUT_FILE="$output_file"
  [ ! -e "$output" ] || { echo "artifact output already exists: $output" >&2; exit 1; }
  mkdir -p "$output"
  [ -d "$ROOT/$SHIPPED_REL" ] || { echo "missing shipped WASM output" >&2; exit 1; }
  [ -d "$ROOT/$PROBE_REL" ] || { echo "missing panic-probe output" >&2; exit 1; }
  cp -a "$ROOT/$SHIPPED_REL" "$output/shipped-wasm"
  cp -a "$ROOT/$PROBE_REL" "$output/panic-probe"
  write_recipe_identity "$output"
  : > "$output/files.sha256"
  write_file_hashes "$output" "$output/shipped-wasm" shipped-wasm
  write_file_hashes "$output" "$output/panic-probe" panic-probe
  {
    echo "artifact_name=${ARTIFACT_PREFIX}-$(manifest_value tree_sha "$output/manifest")"
    echo "tree_sha=$(manifest_value tree_sha "$output/manifest")"
  } >> "$output/manifest"
  emit tree-sha "$(manifest_value tree_sha "$output/manifest")"
  emit artifact-name "${ARTIFACT_PREFIX}-$(manifest_value tree_sha "$output/manifest")"
}

find_artifact_root() {
  local extracted="$1" manifest
  manifest="$(find "$extracted" -type f -name manifest -print -quit)"
  [ -n "$manifest" ] || return 1
  dirname "$manifest"
}

verify_file_set() {
  local artifact_root="$1" expected actual
  expected="$(sed -E 's/^[0-9a-f]+  //' "$artifact_root/files.sha256" | sort)"
  actual="$(
    {
      find "$artifact_root/shipped-wasm" -type f -printf 'shipped-wasm/%P\n'
      find "$artifact_root/panic-probe" -type f -printf 'panic-probe/%P\n'
    } | sort
  )"
  [ "$expected" = "$actual" ] || miss "file-set-mismatch"
  (cd "$artifact_root" && sha256sum -c files.sha256 >/dev/null) || miss "file-hash-mismatch"
}

verify_recipe() {
  local artifact_root="$1"
  source "$ROOT/scripts/ci/toolchain-image.env"
  if [ -n "${GITHUB_REPOSITORY:-}" ]; then
    require_manifest_value repository "$GITHUB_REPOSITORY" "$artifact_root/manifest"
  fi
  require_manifest_value manifest_version 1 "$artifact_root/manifest"
  require_manifest_value retention_days "$RETENTION_DAYS" "$artifact_root/manifest"
  require_manifest_value workflow_sha256 "$(hash_file "$ROOT/.github/workflows/tier1.yml")" "$artifact_root/manifest"
  require_manifest_value producer_workflow_sha256 "$(hash_file "$ROOT/$PRODUCER_WORKFLOW_REL")" "$artifact_root/manifest"
  require_manifest_value run_web_build_sha256 "$(hash_file "$ROOT/scripts/ci/run-web-build.sh")" "$artifact_root/manifest"
  require_manifest_value build_wasm_sha256 "$(hash_file "$ROOT/web/packages/ares-wasm-js/scripts/build-wasm.mjs")" "$artifact_root/manifest"
  require_manifest_value rust_toolchain_sha256 "$(hash_file "$ROOT/rust-toolchain.toml")" "$artifact_root/manifest"
  require_manifest_value toolchain_image_sha256 "$(hash_file "$ROOT/scripts/ci/toolchain-image.env")" "$artifact_root/manifest"
  require_manifest_value cargo_lock_sha256 "$(hash_file "$ROOT/Cargo.lock")" "$artifact_root/manifest"
  require_manifest_value bun_lock_sha256 "$(hash_file "$ROOT/web/bun.lock")" "$artifact_root/manifest"
  require_manifest_value rust_version "$RUST_VERSION" "$artifact_root/manifest"
  require_manifest_value wasm_bindgen_version "$WASM_BINDGEN_VERSION" "$artifact_root/manifest"
  require_manifest_value bun_revision "$BUN_CANARY_REVISION" "$artifact_root/manifest"
  require_manifest_value playwright_version "$PLAYWRIGHT_VERSION" "$artifact_root/manifest"
}

restore_to_workspace() {
  local artifact_root="$1" workspace="$2"
  mkdir -p "$workspace/$SHIPPED_REL" "$workspace/$PROBE_REL"
  find "$workspace/$SHIPPED_REL" -mindepth 1 -delete
  find "$workspace/$PROBE_REL" -mindepth 1 -delete
  cp -a "$artifact_root/shipped-wasm/." "$workspace/$SHIPPED_REL/"
  cp -a "$artifact_root/panic-probe/." "$workspace/$PROBE_REL/"
}

restore_artifact() {
  local workspace="" direct_artifact="" event_name="${GITHUB_EVENT_NAME:-}" repository="${GITHUB_REPOSITORY:-}"
  while [ $# -gt 0 ]; do
    case "$1" in
      --workspace) workspace="$2"; shift 2 ;;
      --artifact-dir) direct_artifact="$2"; shift 2 ;;
      *) usage ;;
    esac
  done
  [ -n "$workspace" ] || usage
  local artifact_root=""
  if [ -n "$direct_artifact" ]; then
    artifact_root="$direct_artifact"
  else
    [ "$event_name" = merge_group ] || miss "not-merge-group"
    command -v gh >/dev/null 2>&1 || miss "gh-unavailable"
    command -v jq >/dev/null 2>&1 || miss "jq-unavailable"
    command -v unzip >/dev/null 2>&1 || miss "unzip-unavailable"
    [ -n "${GH_TOKEN:-}" ] || miss "missing-token"
    [[ "${GITHUB_REF_NAME:-}" =~ ^gh-readonly-queue/main/pr-([0-9]+)-[0-9a-f]{40}$ ]] || miss "not-single-pr-queue"
    local pr_number="${BASH_REMATCH[1]}" base_sha="${GITHUB_EVENT_MERGE_GROUP_BASE_SHA:-}" queue_tree pr_json pr_head_sha pr_tree pr_repo
    queue_tree="$(git -C "$workspace" rev-parse 'HEAD^{tree}')" || miss "missing-queue-tree"
    pr_json="$(gh api "/repos/${repository}/pulls/${pr_number}")" || miss "pr-api-error"
    pr_head_sha="$(jq -r '.head.sha // empty' <<<"$pr_json")"
    pr_repo="$(jq -r '.head.repo.full_name // empty' <<<"$pr_json")"
    [ -n "$pr_head_sha" ] || miss "missing-pr-head"
    [ "$pr_repo" = "$repository" ] || miss "fork-pr"
    [ "$(jq -r '.base.sha // empty' <<<"$pr_json")" = "$base_sha" ] || miss "base-sha-mismatch"
    pr_tree="$(gh api "/repos/${repository}/git/commits/${pr_head_sha}" --jq '.tree.sha // empty')" || miss "commit-api-error"
    [ "$pr_tree" = "$queue_tree" ] || miss "tree-sha-mismatch"
    local runs run_id jobs artifact_id artifact_name="${ARTIFACT_PREFIX}-${queue_tree}" zip extracted
    runs="$(gh api --paginate --slurp "/repos/${repository}/pulls/${pr_number}/runs?per_page=100")" || miss "runs-api-error"
    while IFS= read -r run_id; do
      [ -n "$run_id" ] || continue
      jobs="$(gh api --paginate --slurp "/repos/${repository}/actions/runs/${run_id}/jobs?per_page=100")" || continue
      if ! jq -e --arg name web-build-producer '[.[] | .jobs[] | select(.name == $name and .status == "completed" and .conclusion == "success")] | length > 0' <<<"$jobs" >/dev/null; then
        continue
      fi
      local artifacts_json
      artifacts_json="$(gh api "/repos/${repository}/actions/runs/${run_id}/artifacts?per_page=100")" || continue
      artifact_id="$(jq -r --arg name "$artifact_name" '.artifacts[] | select(.name == $name and .expired == false) | .id' <<<"$artifacts_json" | head -n1)"
      [ -n "$artifact_id" ] || continue
      zip="${RUNNER_TEMP:-/tmp}/web-build-artifact-${run_id}.zip"
      extracted="${RUNNER_TEMP:-/tmp}/web-build-artifact-${run_id}"
      gh api "/repos/${repository}/actions/artifacts/${artifact_id}/zip" > "$zip" || continue
      mkdir -p "$extracted"
      unzip -q "$zip" -d "$extracted" || continue
      artifact_root="$(find_artifact_root "$extracted")" || continue
      break
    done < <(jq -r --arg workflow "$PRODUCER_WORKFLOW_NAME" '[.[] | .workflow_runs[] | select(.name == $workflow and .status == "completed" and .conclusion == "success") | .id] | .[]' <<<"$runs")
    [ -n "$artifact_root" ] || miss "producer-not-found"
  fi
  [ -f "$artifact_root/manifest" ] || miss "manifest-missing"
  verify_recipe "$artifact_root"
  local expected_tree="$(git -C "$workspace" rev-parse 'HEAD^{tree}')"
  require_manifest_value tree_sha "$expected_tree" "$artifact_root/manifest"
  verify_file_set "$artifact_root"
  restore_to_workspace "$artifact_root" "$workspace"
  emit "wasm-hit" true
  emit "panic-probe-hit" true
  echo "web-build artifact: hit (tree=${expected_tree:0:12})"
}

[ $# -gt 0 ] || usage
mode="$1"
shift
case "$mode" in
  should-produce) should_produce "$@" ;;
  create) create_artifact "$@" ;;
  restore) restore_artifact "$@" ;;
  *) usage ;;
esac
