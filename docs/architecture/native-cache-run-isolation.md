# Native parity cache: run isolation and producer provenance

## Scope and evidence

This is **test-harness maintenance**, not a slicing/configuration rewrite milestone. Owners are `crates/ares-cli/tests/orca_parity/runner.rs::{from_env,orca_identity,export_case,slice_case,content_digest}` and the cache-owned test helpers extracted from that module. No production crate, acceptance comparator, reference normalization, application verification, completeness rule or retry policy changes are included.

`runner.rs:182-194` currently places explicitly configured `ARES_ORCA_CACHE_DIR` under `orca_identity`. `orca_identity` hashes the launcher bytes and **environment strings**, not the selected Native executable/resources content. `content_digest` hashes model bytes but silently substitutes empty bytes on read failure; inputs use Debug formatting/DefaultHasher. `slice_case` trusts the existence of `plate_1.gcode`, without binding it to the actual exported 3MF digest or successful producer receipt. The runner is private when that environment variable is absent; the external full-control helper makes `/tmp/ares-orca-cache` its default, thus enabling cross-run sharing in normal validation commands.

The observed `manual_snug` historical shared reference had real ordered motion differences as well as M73 placement differences; its producer is unknown. `manual_strong` also failed against the old shared cache in a later control attempt, while independent fresh official baseline/candidate runs passed. Neither establishes that cache identity is the *cause* of all Native nondeterminism, nor that the difference is M73-only. This slice prevents inappropriate reference reuse and records provenance; it does not repair TreeSupport or excuse fresh differences.

Native resources are real process inputs: `OrcaSlicer.cpp:7147-7174::CLI::setup` resolves platform/package resources, including the compile-time FHS root, and `:2053,2371,2412,2482,2874` reads packaged profiles/CLI config. A wrapper/version/path is not a resource fingerprint. In the configured seed replay package, `bin/orca-slicer` is a **20KB ELF launcher**, and `bin/.orca-slicer-wrapped` is the actual approximately68MB Native image; hashing the `ORCA_REPLAY_BIN` endpoint alone is insufficient.

## Non-negotiable cache semantics

1. **No default reuse across independent runs/bases, even for identical content.** A new `OrcaRunner` allocates a fresh namespace using the existing unique temporary-directory allocation. `ARES_ORCA_CACHE_DIR` becomes a storage root, not permission to share an old reference. With it unset, the existing private work directory remains the storage root. The namespace is never derived only from a label, PID, timestamp, artifact-root path or git SHA; even repeated runs on the same base cannot collide.
2. Reuse is limited to the **same runner lifetime**, same producer and same request. This deliberately also removes cross-test-process cache amortization. No new opt-in cross-run legacy/reuse mode is introduced. Existing rerun-reference behavior still forces a fresh slice; its retry count/acceptance logic is unchanged.
3. Cache schema is `native-cache-v2`. No reading/migrating v1 directories, compatibility lookup, or silent old-key fallback. Old files remain intact as historical evidence; this change does not delete shared caches.
4. Keep Native execution arguments, filenames/logical input names and input bytes unchanged. Cache placement/identity must not alter a filename-sensitive template. Separate the semantic request name from the authoritative cache key; existing short filenames must never become hit authorization.

## Key composition

Layout: `<storage>/native-cache-v2/<fresh-runner-namespace>/<producer-sha256>/<stage>/<request-sha256>/…`.

Hashes use SHA256 with an explicit schema/domain tag and unambiguous length-delimited components (or canonical serialized records with a fixed field order); no DefaultHasher/Debug text for authoritative keys. Full digests, not abbreviated hashes, authorize hits.

### Producer digest

- Exact bytes of `ARES_ORCA_BIN` launcher; exact bytes of a distinct `ORCA_REPLAY_BIN` endpoint when present; exact bytes of the **actual selected Native image**. Record canonical paths for audit, but path/version/mtime is not a content fingerprint.
- Require explicit `ARES_ORCA_NATIVE_BIN` and `ARES_ORCA_RESOURCES_DIR` for cache-producing calls. The former identifies the actual executable image, not an intermediary script/ELF launcher; the latter identifies the resource root used by that selected package/build. These are producer declarations, not new flags overriding Native resource discovery. Official usage supplies `$ORCA_APPDIR/bin/orca-slicer` and `$ORCA_APPDIR/resources`; this seeded package supplies its `.orca-slicer-wrapped` image and `share/OrcaSlicer`. FHS/custom wrappers must declare their true root; never guess by scanning arbitrary store builds or accept a caller-provided digest without hashing the bytes.
- Resource digest covers the **whole declared resource tree**, deterministically sorted by relative path, file type and file bytes. Symlinked resource files/directories contribute their resolved contents at their logical relative paths. Add/remove/rename/content changes invalidate identity; canonical absolute root is audit metadata. Broken/unreadable resources or cycles fail at this external filesystem boundary, not as an empty-resource sentinel.
- Record output-affecting wrapper/environment selection: `ORCA_APPDIR`, `ORCA_REPLAY_BIN`, `ORCA_REPLAY_SEED`, `ORCA_REPLAY_CPU`, `ARES_PARITY_SEEDED_REFERENCE`, locale and inherited affinity/CPU policy. Explicitly record runtime library recipe/environment and HOME/XDG selection for audit. This slice does not claim complete runtime-library reproducibility or eliminate within-producer Native scheduling nondeterminism.
- Compute lazily only when a runner first uses cached export/slice; direct uncached `stages::run` tests and offline initialization retain their existing provisioning behavior. Stream file hashes and retain the producer identity within that runner rather than rehashing resources per case. Producer files are fixed for that runner lifetime; a changed producer requires a new runner. No persistent mtime-based fingerprint cache or defensive resource-tree copy.

### Export request digest

- Ordered, exact JSON bytes written for the effective machine/process/filament presets **after** existing override application, compatibility injection and write-preset metadata. Filament vector order remains significant.
- Exact model bytes, its logical basename/name semantics, and the export command recipe (`--load-settings`, `--load-filaments`, `--arrange 1`, `--export-3mf`). File argument slots bind to those content digests; incidental cache output directories are not request data.
- Label may be retained as audit metadata, never as a substitute for any input digest. Unreadable model/preset input is a StageError; no `unwrap_or_default` authorization.

### Slice request digest

- SHA256 of the **actual exported 3MF bytes passed to Native**, not only the presets/model request that once produced it. Different ZIP container bytes invalidate the slice reference even if uncompressed entries are identical; no new archive normalization.
- Ordered slice command recipe, including selected plate (`--slice 0` today) and any future output-affecting arguments. No implicit export-request-only slice hit.

The fresh namespace prevents cross-run contamination; the producer/request digests make within-run hits auditable. Content hashes alone would still reuse a nondeterministic old reference across runs, so they do **not** replace namespace isolation.

## Hit validity and failure behavior

- A successful export/slice writes a versioned receipt binding namespace, producer/request digests, stage, captured successful command evidence and output byte hashes. Publish output/receipt only after successful capture/read under the existing portable per-entry file lock. A hit requires that receipt and matching output hashes; filename existence alone is not success.
- Missing, inconsistent or tampered cache entries cannot produce PASS. Incomplete entries are cache misses, regenerated under the existing lock with miss reasons retained in evidence; an actual Native process/read/capture error remains StageError, not an accepted result or hidden retry. Do not cache Native failures as successful references.
- Manifests/artifacts distinguish `miss/produced`, `hit/same-runner` and `forced-rerun`, and expose component digests, resource/Native paths and command evidence. Never label an old unknown producer as verified after the fact.
- Preserve the current comparator and its existing allowed generator normalization verbatim. A fresh complete-stream difference remains RED.

## Existing uses invalidated

- `/tmp/ares-orca-cache` and other named roots can no longer provide references to a second base, second invocation or second `OrcaRunner`. Same-root benchmark/control scripts must budget fresh Native production; old caches can only be inspected as historical evidence.
- `ARES_ORCA_BIN` plus `ORCA_APPDIR`/`ORCA_REPLAY_BIN` alone no longer provisions **cached** Native production; callers must supply the actual-image/resources declarations above. Existing full-control scripts can remain unchanged when callers add those environment values. Fake producer fixtures must declare their own executable and deterministic fixture resource tree, not masquerade as a real Native package.
- Tests relying on a stable v1 path, short hasher key, file-exists hits, cross-run hit counts or stale output after in-place binary/resource replacement must migrate to v2 receipts and same-runner hit expectations.
- Existing exporter/slicer payload semantics, cache-free direct Native calls, reference retry policy, preset/application parity and complete-stream acceptance are **not** invalidated or relaxed.

## Prewritten failures and acceptance

Before implementation, prewrite observable harness tests for: same runner/same request hit; identical inputs in a new runner/base using the same root forcing a Native call; unchanged wrapper with changed actual executable; resource-content/rename/add/remove changes across runners; identical model path with changed bytes; preset/vector-order changes; changed exported 3MF with unchanged export request; missing producer declarations; unreadable inputs; receipt/output mismatch; failed/partial publication and concurrent same-entry access. Produce calls/receipts/output artifacts, not just tests of a private hash function.

Then repeat real `manual_snug` and `manual_strong` official/seeded controls on an independent current-main baseline and candidate, with a deliberately common storage root but distinct runner namespaces. Inspect provenance to prove there are no cross-run hits; compare complete bytes with the unchanged acceptance comparator. Fresh differences must be reported rather than attributed to cache by assumption. Run full workspace nextest, strict all-target/all-feature Clippy, fmt/WASM, reference-enabled profile_entry and full official/seeded controls with current-base complete-byte comparisons. Keep changed modules below400LOC by extracting cache-owned logic from the already oversized runner, not by unrelated refactoring.

**Approval gate:** this document is a proposed harness-only contract. No implementation or test changes begin until coordinator review approves its key composition, default isolation and breaking-use list.
