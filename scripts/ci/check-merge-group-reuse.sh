#!/usr/bin/env bash
set -euo pipefail

# Fail-closed check-reuse verifier for GitHub Merge Queue (merge_group).
#
# When a single same-repo PR enters the merge queue and its exact repository
# snapshot (HEAD^{tree}) and base SHA (merge_group.base_sha) match the PR head
# commit that already passed all required PR checks, merge_group does not need
# to re-run those identical checks. Any mismatch (main advanced / rebased,
# batched multi-PR group, fork PR, missing/failed/in-progress check, API error)
# fails closed to false.

WORKFLOW=""
BASE_SHA=""
REF_NAME="${GITHUB_REF_NAME:-}"
REPO="${GITHUB_REPOSITORY:-}"
HEAD_TREE=""
GITHUB_OUTPUT_FILE=""
MOCK_DIR=""

while [ $# -gt 0 ]; do
  case "$1" in
    --workflow) WORKFLOW="$2"; shift 2 ;;
    --base) BASE_SHA="$2"; shift 2 ;;
    --ref-name) REF_NAME="$2"; shift 2 ;;
    --repo) REPO="$2"; shift 2 ;;
    --head-tree) HEAD_TREE="$2"; shift 2 ;;
    --github-output) GITHUB_OUTPUT_FILE="$2"; shift 2 ;;
    --mock-dir) MOCK_DIR="$2"; shift 2 ;;
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

fail_closed() {
  local reason="$1"
  echo "merge-group-reuse: reuse_pr_checks=false reuse_complete_suite=false reason=${reason}" >&2
  emit reuse_pr_checks false
  emit reuse_complete_suite false
  emit reuse_reason "$reason"
  exit 0
}

case "$WORKFLOW" in
  tier1|pages) ;;
  *) echo "usage: check-merge-group-reuse.sh --workflow <tier1|pages> --base <sha> [--github-output <file>]" >&2; exit 2 ;;
esac

if [ -z "$BASE_SHA" ] || [ -z "$REPO" ]; then
  fail_closed "missing-base-or-repo"
fi

if [[ ! "$REF_NAME" =~ ^gh-readonly-queue/main/pr-([0-9]+)-([0-9a-f]{40})$ ]]; then
  fail_closed "non-single-pr-queue-ref"
fi
PR_NUM="${BASH_REMATCH[1]}"

if [ -z "$HEAD_TREE" ]; then
  HEAD_TREE="$(git rev-parse 'HEAD^{tree}' 2>/dev/null || true)"
fi
if [ -z "$HEAD_TREE" ]; then
  fail_closed "missing-head-tree"
fi

fetch_json() {
  local kind="$1" endpoint="$2"
  if [ -n "$MOCK_DIR" ]; then
    cat "${MOCK_DIR}/${kind}.json" 2>/dev/null || return 1
    return 0
  fi
  if [ -z "${GH_TOKEN:-}" ]; then
    return 1
  fi
  gh api --paginate "$endpoint" 2>/dev/null || return 1
}

PR_JSON="$(fetch_json pr "/repos/${REPO}/pulls/${PR_NUM}")" || fail_closed "pr-api-error"

PR_META="$(
  PR_JSON="$PR_JSON" python3 - "$REPO" "$BASE_SHA" <<'PYEOF'
import json, os, sys
repo, base_sha = sys.argv[1:3]
pr = json.loads(os.environ["PR_JSON"])
base = pr.get("base") or {}
head = pr.get("head") or {}
head_repo = (head.get("repo") or {}).get("full_name", "")
if base.get("ref") != "main":
    print("ERR:pr-base-not-main")
elif head_repo != repo:
    print("ERR:fork-pr")
elif base.get("sha") != base_sha:
    print("ERR:base-sha-mismatch")
elif not head.get("sha"):
    print("ERR:missing-pr-head-sha")
else:
    print("OK:" + head["sha"])
PYEOF
)" || fail_closed "pr-parse-error"

case "$PR_META" in
  OK:*) PR_HEAD_SHA="${PR_META#OK:}" ;;
  ERR:*) fail_closed "${PR_META#ERR:}" ;;
  *) fail_closed "pr-meta-invalid" ;;
esac

COMMIT_JSON="$(fetch_json commit "/repos/${REPO}/git/commits/${PR_HEAD_SHA}")" || fail_closed "commit-api-error"
PR_TREE="$(
  COMMIT_JSON="$COMMIT_JSON" python3 - <<'PYEOF'
import json, os
print((json.loads(os.environ["COMMIT_JSON"]).get("tree") or {}).get("sha", ""))
PYEOF
)" || fail_closed "commit-parse-error"

if [ "$PR_TREE" != "$HEAD_TREE" ]; then
  fail_closed "tree-sha-mismatch"
fi

CHECKS_JSON="$(fetch_json checks "/repos/${REPO}/commits/${PR_HEAD_SHA}/check-runs?per_page=100")" || fail_closed "checks-api-error"

RESULT="$(
  CHECKS_JSON="$CHECKS_JSON" python3 - "$WORKFLOW" <<'PYEOF'
import json, os, sys

workflow = sys.argv[1]
raw = os.environ["CHECKS_JSON"].strip()
# gh api --paginate concatenates JSON objects across pages.
dec = json.JSONDecoder()
pos = 0
runs = []
while pos < len(raw):
    while pos < len(raw) and raw[pos].isspace():
        pos += 1
    if pos >= len(raw):
        break
    obj, pos = dec.raw_decode(raw, pos)
    runs.extend(obj.get("check_runs", []))

latest = {}
for r in runs:
    name = r.get("name")
    rid = r.get("id", 0)
    if not name:
        continue
    prev = latest.get(name)
    if prev is None or rid > prev.get("id", 0):
        latest[name] = r

def passed(name: str) -> bool:
    r = latest.get(name)
    return bool(r and r.get("status") == "completed" and r.get("conclusion") == "success")

if workflow == "pages":
    if passed("pages-build"):
        print("true false pr-pages-build-passed")
    else:
        print("false false pr-pages-build-not-passed")
    sys.exit(0)

required_pr = ["fast-checks", "rust-wasm-check", "rust-clippy", "rust-nonparity", "tier1-gate"]
missing = [n for n in required_pr if not passed(n)]
if missing:
    print(f"false false pr-checks-missing:{','.join(missing)}")
    sys.exit(0)

# Also reuse complete suite if the PR explicitly ran the full suite (e.g. run-full-parity)
# and every complete-suite check succeeded with zero non-success runs.
complete_ok = (
    passed("build-parity")
    and passed("web-build")
    and passed("web-streaming")
    and any(n.startswith("parity-official") and passed(n) for n in latest)
    and any(n.startswith("parity-seeded") and passed(n) for n in latest)
    and any(n.startswith("web-desktop") and passed(n) for n in latest)
    and all(
        r.get("status") == "completed" and r.get("conclusion") in ("success", "skipped")
        for r in latest.values()
    )
    and all(
        passed(n)
        for n in latest
        if n.startswith(("parity-official", "parity-seeded", "web-desktop", "web-packages", "web-mobile"))
    )
)
if complete_ok:
    print("true true pr-full-suite-passed")
else:
    print("true false pr-checks-passed")
PYEOF
)" || fail_closed "checks-parse-error"

read -r REUSE_PR REUSE_COMPLETE REASON <<< "$RESULT"
echo "merge-group-reuse: pr=#${PR_NUM} head=${PR_HEAD_SHA:0:9} tree=${HEAD_TREE:0:9} reuse_pr_checks=${REUSE_PR} reuse_complete_suite=${REUSE_COMPLETE} reason=${REASON}" >&2
emit reuse_pr_checks "$REUSE_PR"
emit reuse_complete_suite "$REUSE_COMPLETE"
emit reuse_reason "$REASON"
