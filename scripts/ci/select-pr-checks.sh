#!/usr/bin/env bash
set -euo pipefail

# Diff-based check selector.
# - On merge_group: runs the 100% complete suite unless EVERY changed file in
#   base_sha..HEAD is strictly a pure Markdown doc (docs/**/*.md, README.md,
#   AGENTS.md, CONTRIBUTING.md, CHANGELOG.md; never THIRD_PARTY_NOTICES.md).
#   Fails closed to full on missing/empty diff or any non-doc file.
# - On push / workflow_dispatch: ALWAYS runs the complete suite.
# - On pull_request: narrows PR feedback to targets reachable from the diff,
#   failing closed to a full rerun for trigger files or any unknown path.

EVENT_NAME=""
BASE_SHA=""
HEAD_SHA="HEAD"
FILES_LIST=""
GITHUB_OUTPUT_FILE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --event) EVENT_NAME="$2"; shift 2 ;;
    --base) BASE_SHA="$2"; shift 2 ;;
    --head) HEAD_SHA="$2"; shift 2 ;;
    --files) FILES_LIST="$2"; shift 2 ;;
    --github-output) GITHUB_OUTPUT_FILE="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

emit() {
  local key="$1" val="$2"
  echo "${key}=${val}"
  if [ -n "$GITHUB_OUTPUT_FILE" ]; then
    echo "${key}=${val}" >> "$GITHUB_OUTPUT_FILE"
  fi
}

emit_full() {
  local reason="$1"
  echo "selection: mode=full reason=${reason}" >&2
  emit mode full
  emit full_rerun true
  emit run_complete_suite true
  emit docs_only false
  emit run_rust_wasm true
  emit run_rust_clippy true
  emit run_rust_nonparity true
  emit rust_crates all
  emit run_web true
  emit wasm_rebuild true
  emit reason "$reason"
  exit 0
}

is_pure_doc_file() {
  case "$1" in
    docs/*.md|README.md|AGENTS.md|CONTRIBUTING.md|CHANGELOG.md) return 0 ;;
    *) return 1 ;;
  esac
}

if [ "$EVENT_NAME" != "pull_request" ] && [ "$EVENT_NAME" != "merge_group" ]; then
  emit_full "event:${EVENT_NAME:-non-pr}"
fi

changed_files=()
if [ -n "$FILES_LIST" ]; then
  while IFS= read -r line; do
    [ -n "$line" ] && changed_files+=("$line")
  done < "$FILES_LIST"
elif [ "$EVENT_NAME" = "merge_group" ]; then
  if [ -z "$BASE_SHA" ]; then
    emit_full "merge-group-no-base-sha"
  fi
  if ! git rev-parse --verify -q "$BASE_SHA" >/dev/null 2>&1; then
    git fetch --no-tags --filter=blob:none --depth=1 origin "$BASE_SHA" >/dev/null 2>&1 || true
  fi
  if git rev-parse --verify -q "$BASE_SHA" >/dev/null 2>&1; then
    mapfile -t changed_files < <(git diff --name-only "$BASE_SHA" "$HEAD_SHA")
  else
    emit_full "merge-group-missing-base-sha"
  fi
else
  # On GitHub pull_request events, HEAD is the synthetic merge commit refs/pull/N/merge
  # whose first parent (HEAD^1) is the exact target branch commit.
  if git rev-parse --verify -q "${HEAD_SHA}^1" >/dev/null 2>&1; then
    mapfile -t changed_files < <(git diff --name-only "${HEAD_SHA}^1" "$HEAD_SHA")
  elif [ -n "$BASE_SHA" ]; then
    if ! git rev-parse --verify -q "$BASE_SHA" >/dev/null 2>&1; then
      git fetch --no-tags --filter=blob:none --depth=1 origin "$BASE_SHA" >/dev/null 2>&1 || true
    fi
    if git rev-parse --verify -q "$BASE_SHA" >/dev/null 2>&1; then
      mapfile -t changed_files < <(git diff --name-only "$BASE_SHA" "$HEAD_SHA")
    else
      emit_full "missing-base-sha"
    fi
  else
    emit_full "no-diff-base"
  fi
fi

if [ "${#changed_files[@]}" -eq 0 ]; then
  emit_full "empty-diff"
fi

if [ "$EVENT_NAME" = "merge_group" ]; then
  for f in "${changed_files[@]}"; do
    if ! is_pure_doc_file "$f"; then
      emit_full "merge-group-non-doc:${f}"
    fi
  done
  echo "selection: mode=docs-only event=merge_group files=${#changed_files[@]}" >&2
  emit mode docs-only
  emit full_rerun false
  emit run_complete_suite false
  emit docs_only true
  emit run_rust_wasm false
  emit run_rust_clippy false
  emit run_rust_nonparity false
  emit rust_crates none
  emit run_web false
  emit wasm_rebuild false
  emit reason "merge-group-docs-only"
  exit 0
fi

run_rust_wasm=false
run_rust_clippy=false
run_rust_nonparity=false
run_web=false
wasm_rebuild=false

need_core=false
need_vgcode=false
need_wasm=false
need_cli=false
matched_reasons=()

# Explicit full-rerun trigger inputs: touching any of these forces all checks.
is_full_rerun_trigger() {
  case "$1" in
    Cargo.toml|Cargo.lock|rust-toolchain.toml|.config/nextest.toml|.cargo/*) return 0 ;;
    vendor/*) return 0 ;;
    web/package.json|web/bun.lock|web/tsconfig*.json|web/.oxlintrc.json|web/bunfig.toml) return 0 ;;
    web/packages/ares-wasm-js/scripts/build-wasm.mjs) return 0 ;;
    .github/workflows/tier1.yml|.github/workflows/cloudflare-pages.yml|.github/actions/*) return 0 ;;
    scripts/ci/*|scripts/gen-configdef*|scripts/verify-configdef.mjs) return 0 ;;
    *) return 1 ;;
  esac
}

for f in "${changed_files[@]}"; do
  if is_full_rerun_trigger "$f"; then
    emit_full "trigger:${f}"
  fi

  case "$f" in
    crates/ares-core/*)
      run_rust_wasm=true
      run_rust_clippy=true
      run_rust_nonparity=true
      run_web=true
      wasm_rebuild=true
      need_core=true
      need_vgcode=true
      need_wasm=true
      need_cli=true
      matched_reasons+=("ares-core")
      ;;
    crates/ares-vgcode/*)
      run_rust_wasm=true
      run_rust_clippy=true
      run_rust_nonparity=true
      run_web=true
      wasm_rebuild=true
      need_vgcode=true
      need_wasm=true
      need_cli=true
      matched_reasons+=("ares-vgcode")
      ;;
    crates/ares-wasm/*)
      run_rust_wasm=true
      run_rust_clippy=true
      run_rust_nonparity=true
      run_web=true
      wasm_rebuild=true
      need_wasm=true
      need_cli=true
      matched_reasons+=("ares-wasm")
      ;;
    crates/ares-cli/*|tests/*)
      run_rust_clippy=true
      run_rust_nonparity=true
      need_cli=true
      matched_reasons+=("ares-cli")
      ;;
    web/*)
      run_web=true
      matched_reasons+=("web")
      ;;
    THIRD_PARTY_NOTICES.md|LICENSES/*)
      run_web=true
      matched_reasons+=("licenses")
      ;;
    docs/*|*.md|AGENTS.md|hk.pkl|.gitignore|.github/workflows/oracles-build.yml|.github/workflows/toolchain-build.yml|tools/*)
      matched_reasons+=("leaf-docs-or-tooling")
      ;;
    *)
      # Fail-closed on any uncategorized path so new top-level files are never silently skipped.
      emit_full "uncategorized:${f}"
      ;;
  esac
done

crates=()
if [ "$need_core" = "true" ] && [ "$need_vgcode" = "true" ] && [ "$need_wasm" = "true" ] && [ "$need_cli" = "true" ]; then
  rust_crates="all"
else
  [ "$need_core" = "true" ] && crates+=("ares-core")
  [ "$need_vgcode" = "true" ] && crates+=("ares-vgcode")
  [ "$need_wasm" = "true" ] && crates+=("ares-wasm")
  [ "$need_cli" = "true" ] && crates+=("ares-cli")
  rust_crates="${crates[*]:-none}"
fi

unique_reasons="$(printf '%s\n' "${matched_reasons[@]}" | sort -u | paste -sd, -)"
echo "selection: mode=selected files=${#changed_files[@]} reasons=${unique_reasons} rust_crates=${rust_crates} run_web=${run_web} wasm_rebuild=${wasm_rebuild}" >&2

emit mode selected
emit full_rerun false
emit run_complete_suite false
emit docs_only false
emit run_rust_wasm "$run_rust_wasm"
emit run_rust_clippy "$run_rust_clippy"
emit run_rust_nonparity "$run_rust_nonparity"
emit rust_crates "$rust_crates"
emit run_web "$run_web"
emit wasm_rebuild "$wasm_rebuild"
emit reason "$unique_reasons"
