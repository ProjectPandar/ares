# Native content cache for direct Slice and Export commands

This document defines a test-adapter infrastructure slice. It ports the existing content-addressed Native cache convention to direct command invocations rather than changing slicer behavior, comparators, assertions, or test identities.

## Scope and source boundary

The implementation boundary is the `ares-cli` parity runner:

- `crates/ares-cli/tests/orca_parity/runner/command.rs::run` and `run_exact` own direct Native command capture, lifecycle classification, and captured output.
- `crates/ares-cli/tests/orca_parity/runner/stages.rs::run` and its stage wrappers route Export and Slice commands to that boundary.
- `crates/ares-cli/tests/orca_parity/runner/cache.rs` owns the existing `native-cache-v2` storage, locking, atomic publication, and cache receipts for authored export/slice cases.
- `crates/ares-cli/tests/orca_parity/runner/cache_identity.rs` owns producer and byte-digest conventions.

The existing cache remains authoritative for `cached_slice` and related paths. This slice adds a command-result cache for direct `Stage::Export` and `Stage::Slice` calls that currently bypass it. Replay and deliberate rerun paths remain live. Reuse of replay pairs is deferred.

## Required behavior

For a direct Export or Slice invocation, construct a request key from:

1. A schema/version discriminator.
2. The existing producer digest from `cache_identity::identify`, including Native binary, launcher, replay endpoint, resources tree, policy, catalog, and effective producer identity already represented by the producer receipt.
3. The effective command arguments in order, normalized as follows:
   - ordinary scalar arguments remain literal;
   - every file argument is represented by its content digest and basename, never its absolute or temporary path;
   - the output directory is represented by a stable output-directory marker, not its path;
   - `--datadir` is represented by a deterministic digest of its complete directory tree, including relative names, file bytes, and directory markers;
   - path-bearing values are classified from the command contract, not by an unsafe global string replacement.
4. Only the environment explicitly supplied to the child command through `Command::env`/`Command::envs`, together with any explicit `env_clear` policy. Represent the supplied set as sorted key/value data; unset and empty values remain distinct. Do not include inherited parent variables such as `NEXTEST_*`, `ORCA_REPLAY_CPU`, or `PATH` merely because they are visible to the test harness. If an inherited variable is shown to affect Native output, add it to the producer identity only through an explicit, documented producer contract; it is not part of this request key.
5. The stage.

The request digest must use the length-delimited SHA-256 helper already used by `cache_identity.rs`. No host path, temporary directory, or machine-specific output path may enter the key.

## Cache value and publication

A cache entry uses the existing `native-cache-v2` root and lock/temporary-directory/rename conventions from `cache.rs`. The entry stores:

- `result.json`, containing the schema, producer digest, request digest, normalized request components, stage, exit status/code, and output digests;
- the complete captured `stdout` bytes;
- the complete captured `stderr` bytes;
- the command result needed by the caller, including the exit status and code;
- the output directory contents produced by the command, preserving relative names and bytes.

Publication is staged and atomic under the per-entry lock. A hit is valid only when the receipt, result, stdout, stderr, and every declared output file are present and internally digest-consistent. Missing or malformed entries are cache misses and fall back to a live command. A corrupt entry must never produce a successful result without reproducing the existing captured-result classification.

The materialized output directory for a hit is created under the caller's requested work directory, then populated from the cached relative files before the caller reads it. The materialization path is never part of the key.

Only these results may be published:

- successful exits;
- deterministic input rejections whose complete status and diagnostics are stable under the existing stage classification.

Never publish timeouts, wait failures, spawn failures, capture I/O failures, crashes, signals, or other unavailable-status results. A deterministic rejection must retain its original non-success code, stdout, stderr, and `StageError` classification when replayed; the cache must not convert a failure into a pass.

## Live and forced paths

`stages::run_with_timeout` remains the live execution API. Direct commands opt into the content cache only for Export and Slice stages and only where the caller has not requested a replay or forced rerun. `rerun_reference` and all replay/forced paths bypass lookup and publication. Existing command capture, timeout, shutdown, diagnostics, and comparator behavior remain unchanged.

The producer digest is computed once per `OrcaRunner` process through the existing `OnceLock` path. Each nextest test has its own process, so this only avoids repeated identification among multiple runners in one test. Persisting or sharing producer keys across test processes is a separate follow-up and is not part of this slice.

## Determinism and safety rules

- Cache lookup is pure with respect to Native binary, launcher, resources, input bytes, effective arguments, datadir bytes, and explicitly supplied child environment.
- Cache entries never include path strings in the key or normalized request components except basenames used to disambiguate content-addressed file arguments.
- Directory traversal is sorted by relative encoded name and records directory/file type markers.
- A cache hit must reconstruct the same captured bytes, exit status, and declared output bytes as the original clean result.
- Existing comparators and assertions consume the same captured result and output files they receive on a live run.
- No public API, dependency, core slicing behavior, G-code formatting, Native comparator, or test ID changes are in scope.
- Keep modified files at or below 400 lines; use the existing small-module structure rather than growing a large command module.

## Failure-mode checklist before implementation

Before writing implementation code, enumerate and bind a failing harness-level test or reproducible check for each failure mode:

1. A normalized file argument accidentally retains its directory path or fails to distinguish different file bytes with the same basename.
2. An output-directory path enters the key and prevents reuse across equivalent work directories.
3. A `--datadir` tree hash omits a nested file, relative name, or directory/file type.
4. Two explicit child-environment sets that differ in an output-relevant value incorrectly share an entry, or an inherited harness variable incorrectly fragments the key.
5. A missing, malformed, or digest-inconsistent entry is treated as a hit instead of falling back live.
6. A cached deterministic rejection changes its exit code, diagnostics, or failure classification.
7. A timeout or crash is published and later replayed as a clean result.
8. A replay or forced rerun accidentally reads or writes the ordinary command cache.

The failing checks must fail against the pre-implementation behavior where the new contract is absent, then pass after the implementation. The checks are harness-level and must not alter production comparators or test identities.

## Engineering verification

The implementation must provide:

1. Harness-level coverage for normalized file arguments, output-directory normalization, datadir tree hashing, explicit-environment separation, corrupt-entry fallback, deterministic-rejection replay, timeout/crash exclusion, and replay/forced-path bypass.
2. Approximately 100 `orca_parity` tests run twice with the same inputs: cold cache, then warm cache.
3. Exact equality of PASS/FAIL test-ID sets between cold and warm runs.
4. Byte-identical compared outputs for corresponding cold and warm cases.
5. Measured lower Native time on the warm run, with cache hit/miss counts and timing measurements recorded.
6. A dual official/seeded FULL comparison against the current main tip, with inherited failures preserved and transitions analyzed by exact test ID.
7. `cargo fmt --all -- --check`, workspace clippy with warnings denied, and a scope audit.

## Deferred behavior

- Replay-pair caching remains explicitly deferred.
- Persisting producer keys across separate test processes remains explicitly deferred.
- Automatic cache invalidation beyond the content key remains unnecessary; changing any keyed producer, input, datadir, argument, or explicit child-environment content creates a new entry.
- Cross-process cache garbage collection is not part of this slice.
- Cache compression is not required.
