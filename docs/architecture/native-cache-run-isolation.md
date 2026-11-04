# Native parity cache: content addressing and producer provenance

## Harness boundary and evidence

This is test-harness maintenance, not a production slicing/configuration rewrite. The boundary is `crates/ares-cli/tests/orca_parity/runner.rs::{from_env,orca_identity,export_case,slice_case,content_digest}` and cache-owned extracted helpers. Comparators, generator normalization, application verification, completeness and retry policy remain unchanged.

The current identity hashes launcher bytes and environment strings, not actual Native/resources content; slice hits depend on `plate_1.gcode` existence rather than actual exported 3MF bytes. Historical manual_snug/manual_strong shared references have uncertain producer provenance; do not retrospectively classify their motion differences as M73-only. The bounded correction is to bind producer and request content, not prohibit legitimate reuse or repair Native nondeterminism.

Resources are process inputs (`OrcaSlicer.cpp:7147-7174::CLI::setup`, including FHS roots; `:2053,2371,2412,2482,2874` reads packaged profiles/CLI config). The seed package's 20KB `bin/orca-slicer` ELF launcher is not its approximately68MB actual `bin/.orca-slicer-wrapped` image.

## Cache semantics

- Pure content addressing: `<storage>/native-cache-v2/<producer-sha256>/<stage>/<request-sha256>/…`. No runner/run/base namespace, git SHA, timestamp or PID in the key. Identical content must hit across runners, runs and bases.
- `ARES_ORCA_CACHE_DIR` remains a reusable storage root. With it unset, the existing private work directory remains storage. To require fresh production, specify a new cache directory; no new fresh switch.
- Never read/migrate v1 cache entries or fall back to weak keys. Preserve old files as evidence rather than deleting them.
- Preserve Native payloads, command semantics, logical filenames, order and existing reference retry behavior. Cache directories must not change filename-sensitive templates. Human labels/short filename stems are not authorization keys.

## Key components

Use full SHA256 and schema/domain-separated, length-delimited components or canonical records. Do not use Debug text/DefaultHasher as authoritative identity.

**Producer:** exact launcher/endpoint bytes, actual selected Native image bytes, deterministic whole-resource-tree digest, replay seed and execution recipe. Require `ARES_ORCA_NATIVE_BIN` and `ARES_ORCA_RESOURCES_DIR` for cached production, identifying the actual producer and its actual resource root, not changing Native discovery. Official callers declare `$ORCA_APPDIR/bin/orca-slicer` and `$ORCA_APPDIR/resources`; this seed package declares `.orca-slicer-wrapped` and `share/OrcaSlicer`. Custom/FHS builds declare their true selection; do not discover arbitrary builds. Paths/version/mtime alone are not content identity. Include seed and output-affecting mode/locale/effective parallelism policy; physical CPU IDs are audit metadata, not a content-key namespace (the existing pin runner chooses different IDs per run while keeping parallelism1). Record runtime-library/HOME/XDG selection for audit without claiming complete runtime reproducibility.

Resource hashing orders logical relative paths and file types and hashes bytes, following resource symlinks to their contents. Changes to any file invalidate producer identity. Hash lazily once per runner, stream large files, and hold the resulting identity for its fixed producer lifetime; no per-case tree rehash, persistent mtime fingerprint cache or resource copies. Direct uncached Native commands/offline provisioning remain unchanged. Seeded replay addresses Native randomness; neither cache addressing nor forced fresh claims to fix all Native nondeterminism.

**Export request:** exact effective serialized machine/process/ordered filament presets after existing overrides/compatibility/write-preset metadata; model bytes and logical basename; ordered export command recipe. No empty-byte read fallback. Preserve payload naming independently of content key.

**Slice request:** exact actual exported 3MF bytes passed to Native plus ordered slice command recipe/selected plate. Different ZIP container bytes miss even when uncompressed contents match. No archive normalization or export-request-only slice hit.

## Publication and audit

Use existing portable per-entry locks. Produce into a staging directory, then publish the complete successful entry via atomic rename. Incomplete entries are misses; Native process/capture/read errors remain StageError and are not cached successes. Existing forced rerun still performs a fresh slice.

Receipts contain key components, producer/resource paths, command evidence and output hashes **for audit only**. Do not add tamper detection, rehash cached output on hit, or make receipt hash validation a new acceptance layer. Record hit/miss/forced-rerun; complete publication, not a loose output filename, defines a hit. The complete-stream comparator remains the sole existing parity acceptance mechanism.

## Existing usage changes

- V1 directory layouts and weak-key/file-exists hit assumptions are invalid; no fallback. V2 content-identical cross-base/run hits remain valid and required.
- Cached producers need explicit actual-image/resource declarations. Callers of existing full-control scripts can add these environment values without changing those helpers; mock command fixtures declare their own image and fixture resource tree.
- A new runner is **not** an invalidation event. Shared-root benchmarks keep cross-run amortization. Fresh verification continues to use a new `ARES_ORCA_CACHE_DIR`.
- Comparators, payloads, offline/direct-uncached calls and retry counts are unaffected.

## Prewritten failure cases and acceptance

Confirmed seam: real `OrcaRunner` export/slice entrypoints and their captured subprocess outputs, not a private hash function. Before implementation prewrite: (1) identical producer/request across two runners hits; (2) unchanged wrapper but changed actual Native image misses; (3) any resource-file content change misses; (4) changed actual 3MF bytes with unchanged export request misses; (5) changed replay seed misses. Retain observable Native-call counts, complete output bytes and audit artifacts. No new-runner-must-miss or tamper tests.

For real manual_snug/manual_strong acceptance, use the same storage root: run old base first, then new base. With identical producer/request keys the second run must hit and return byte-identical complete references; changed keys must miss. Use official/seeded producers distinctly, keep seed in identity, and report any genuine fresh parity RED. Run workspace nextest, strict all-target/all-feature Clippy, fmt/WASM, reference-enabled profile_entry and full official/seeded controls. Extract only cache-owned logic to keep changed modules below400LOC.
