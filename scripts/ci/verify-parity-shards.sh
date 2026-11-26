#!/usr/bin/env bash
set -euo pipefail

# Proves that cargo nextest hash partitioning across 1..N shards covers 100% of
# the profile's tests with zero missing and zero duplicate entries.

ARCHIVE_FILE="${1:?usage: verify-parity-shards.sh <archive.tar.zst> <profile> <total_shards>}"
PROFILE="${2:?missing profile}"
TOTAL="${3:?missing total shards}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

list_tests() {
  cargo nextest list \
    --archive-file "$ARCHIVE_FILE" \
    --workspace-remap "$PWD" \
    --profile "$PROFILE" \
    "$@" \
    --message-format oneline | LC_ALL=C sort
}

list_tests > "$tmp_dir/full.txt"
full_count="$(wc -l < "$tmp_dir/full.txt")"
if [ "$full_count" -eq 0 ]; then
  echo "FAIL: profile $PROFILE listed 0 tests" >&2
  exit 1
fi

: > "$tmp_dir/all_shards.txt"
for ((i = 1; i <= TOTAL; i++)); do
  list_tests --partition "hash:${i}/${TOTAL}" > "$tmp_dir/shard_${i}.txt"
  shard_count="$(wc -l < "$tmp_dir/shard_${i}.txt")"
  echo "  ${PROFILE} shard ${i}/${TOTAL}: ${shard_count} tests"
  cat "$tmp_dir/shard_${i}.txt" >> "$tmp_dir/all_shards.txt"
done

LC_ALL=C sort "$tmp_dir/all_shards.txt" > "$tmp_dir/union_with_dups.txt"
LC_ALL=C sort -u "$tmp_dir/all_shards.txt" > "$tmp_dir/union_unique.txt"

dup_count="$(($(wc -l < "$tmp_dir/union_with_dups.txt") - $(wc -l < "$tmp_dir/union_unique.txt")))"
if [ "$dup_count" -ne 0 ]; then
  echo "FAIL: ${PROFILE} has ${dup_count} duplicate test assignments across ${TOTAL} shards" >&2
  exit 1
fi

if ! diff -u "$tmp_dir/full.txt" "$tmp_dir/union_unique.txt" > "$tmp_dir/diff.txt"; then
  echo "FAIL: ${PROFILE} shard union does not equal full test inventory:" >&2
  head -n 40 "$tmp_dir/diff.txt" >&2
  exit 1
fi

echo "PROVEN: ${PROFILE} union of ${TOTAL} disjoint shards == ${full_count}/${full_count} tests (0 missing, 0 duplicate)"
