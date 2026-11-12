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

Publication is staged and atomic under the per-entry lock. A hit is valid only when the receipt, result, stdout, stderr, and declared output snapshots are present or absent exactly as recorded and internally digest-consistent. The corruption policy is **cache miss, rerun Native, then replace the damaged entry with the newly captured eligible result**. Missing, malformed, digest-inconsistent, or unsupported filesystem entries never return cached bytes or a partial result. An E2E check must tamper with a rejection record to claim success and prove that the caller still receives the original Native process failure after a live rerun.

All cached output roots are first copied into private staging under the caller's work directory and revalidated against the record before any requested output is populated. Source snapshot/read failures discard the staged copies and become a cache miss; ordinary caller-output I/O errors remain explicit I/O failures, not successful cache results. Non-regular cache file types, including symlinks, are ineligible. The materialization path is never part of the key.

Only these results may be published:

- successful exits;
- deterministic input rejections whose complete status and diagnostics are stable under the existing stage classification.

Never publish timeouts, wait failures, spawn failures, capture I/O failures, crashes, signals, or other unavailable-status results. A deterministic rejection must retain its original non-success code, stdout, stderr, and `StageError` classification when replayed; the cache must not convert a failure into a pass.

## Live and forced paths

`stages::run_with_timeout` remains the live execution API. Direct commands opt into the content cache only for Export and Slice stages and only where the caller has not requested a replay or forced rerun. `rerun_reference` and all replay/forced paths bypass lookup and publication. Existing command capture, timeout, shutdown, diagnostics, and comparator behavior remain unchanged.

The existing `OrcaRunner` cache retains its `OnceLock` producer path. Direct commands persist producer identities under `native-cache-v2/producer-identity`. Before each use, the record must match the Native executable's canonical path plus `(device, inode, size, mtime_ns)`, with nanoseconds including both seconds and the subsecond component. A mismatch forces recomputation and atomic replacement under the existing lock/temp/rename convention. The record also validates its checksum and declared launcher, resource location, policy, catalog location, and parallelism context. This is safe for the pinned oracle because the Nix store executable and its declared producer inputs remain immutable within that context; replacement or any Native stat change invalidates the record. A subprocess E2E verifies first computation, cross-process reuse, and recomputation after executable replacement.

The filesystem identity is only the lookup key for this producer record. The producer digest itself remains content-based, as do normalized command keys. Datadir content is still read and hashed on every eligible request; no mtime-only datadir digest memo is used. Timings separate normalization, input/datadir hashing, producer lookup, receipt/output validation, materialization, and publication.

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
9. A successful cached export restores the declared archive but drops Native's implicit cwd `result.json` or default logs.

The failing checks must fail against the pre-implementation behavior where the new contract is absent, then pass after the implementation. The checks are harness-level and must not alter production comparators or test identities.

## Engineering verification

The implementation must provide:

1. Harness-level coverage for normalized file arguments, output-directory normalization, datadir tree hashing, explicit-environment separation, corrupt-entry fallback, deterministic-rejection replay, timeout/crash exclusion, and replay/forced-path bypass.
2. Approximately 100 `orca_parity` tests run twice with the same inputs: cold cache, then warm cache. Export the existing Native comparison gates and report every ID's executed/skipped status. Skipped Native gates must be zero in the counted execution sample; any excluded skips are listed separately and never represented as successful Native comparisons.
3. Exact equality of PASS/FAIL test-ID sets between cold and warm runs.
4. Byte-identical compared outputs for corresponding cold and warm cases.
5. Measured lower Native time on the warm run, with cache hit/miss counts and timing measurements recorded.
6. A dual official/seeded full-suite comparison against the current main tip remains integration qualification, with inherited failures preserved and transitions analyzed by exact test ID; focused validation does not substitute for it.
7. Warm wall-time comparisons use three repetitions per mode and revision, exclude compilation and warm-up, and report medians and spread. Both candidate mode medians must be no slower than their baseline medians, with a clear improvement in at least one mode.
8. `cargo fmt --all -- --check`, workspace clippy with warnings denied, and a scope audit.

## Implemented command boundary

`stages::run` and `run_with_env` admit the direct command cache for Export and Slice. The recognized contract includes positional input files, semicolon-separated `--load-settings` and `--load-filaments` files, `--datadir`, `--outputdir`, named `--export-3mf` outputs, and scalar `--slice`, `--arrange`, and `--debug` values. Named export outputs retain their basename; output directory paths use only a marker. Unsupported argument shapes and already-populated output targets remain live, rather than guessing whether a path is an input.

Only the configured base Native launcher uses this cache. Instrumented helper binaries retain their own producer boundaries. `run_live`, `run_live_with_env`, and `run_with_timeout` never enter lookup or publication. Existing cache-owned subprocesses, `NativeCase::slice`, explicit self-replay helpers, and environment-controlled fuzzy replays route through those live APIs. Replay qualification therefore cannot accidentally become a comparison against cached bytes.

The result receipt hashes the entire result record. The record binds the producer, normalized request, code/status, capture hashes, and output-root tree hashes. Tree hashes include relative names, file bytes, and empty directories. Stored result records, captures, and trees must all validate before materialization. Only success and input rejections recognized by `stages::attributable` can be published; dynamically extracted option names must exist in the option-definition registry. A diagnostic header alone or a generic nonzero exit is not sufficient.

## Implicit Native cwd products

Declared output arguments are not the whole Native CLI output contract. At pinned source `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `OrcaSlicer.cpp:535–586`, `record_exit_reson`, writes `outputdir/result.json` on Linux when outputdir is nonempty, otherwise the relative path `result.json` in the command cwd. The same function writes success and failure results.
- `OrcaSlicer.cpp:1524–1535` initializes logging independently of outputdir. `PrintConfig.cpp:13196–13200` supplies an empty default logfile; `libslic3r/utils.cpp:197–200` delegates to Boost.Log. The pinned Boost1.86 `text_file_backend.hpp:402–408` defaults to `%5N.log`: zero-filled numeric counters with minimum width5.
- `OrcaSlicer.cpp:7182–7207` uses `PartPlate` temporary G-code paths when outputdir is empty. These paths are not an output namespace the command cache can fully restore.

Every eligible request therefore also declares a scoped cwd snapshot containing only `result.json` and numeric default log names, not inputs, command-capture directories, cache-owned directories or arbitrary cwd contents. Its stable `native-cli-diagnostics-v1` marker participates in the request digest, so old incomplete snapshots cannot hit the new request. The existing checksummed directory snapshot and private staging protocol validate and restore this output root alongside explicit outputs, including recorded absence of files. Cwd logs are captured even when an explicit outputdir exists because logging is independent of that flag.

A populated cwd diagnostic namespace remains live: existing files may be appended to or replaced by Native and are not a clean replay destination. Any invocation containing `--slice` without `--outputdir` also remains live, even if it declares an export archive. Unsupported output/log flags continue to bypass the cache rather than extending this contract by guesswork.

The Native subprocess regression calls the public stage entry twice with identical arguments and cwd, using an isolated cache. After the first live call it retains reference result/log bytes and removes generated outputs. The second call must be an actual cache hit and restore byte-identical `result.json` and `00000.log`; neither bypass nor a live rerun satisfies that assertion. The unmodified failing baseline reaches the second hit and then fails with ENOENT reading `result.json`.

The runner audit distinguishes the affected `vendor_profile_compatibility::compare` export (ten printer cases; no outputdir) from `preset_include_controls::check` (cwd log reader using live `command::run`/unsupported export-settings arguments), explicit-outputdir result readers, and cache-owned/live result pipelines. Cache metadata and Ares-authored `result.json` reports are not Native cwd consumers. No comparator or incumbent consumer assertion is weakened.

## Deferred behavior

- Replay-pair caching remains explicitly deferred.
- Producer identity persistence is limited to the canonical Native executable stat key described above; resource-tree changes outside that key remain governed by the pinned immutable oracle contract.
- Automatic cache invalidation beyond the content key remains unnecessary; changing any keyed producer, input, datadir, argument, or explicit child-environment content creates a new entry.
- Cross-process cache garbage collection is not part of this slice.
- Cache compression is not required.
