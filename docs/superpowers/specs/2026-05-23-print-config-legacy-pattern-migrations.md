# M177 Spec: PrintConfig legacy pattern migration slice

## Goal
Port the legacy `zig-zag` pattern normalization branch from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8013-8019`: migrate `zig-zag` to `rectilinear` for six legacy pattern option keys.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8020+`: `filament_map_mode`, `filament_type`, prime-tower rib migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M177 pattern branch.
- `crates/ares-core/src/options/tests/legacy_pattern_migrations.rs`: add focused M177 tests proving all covered keys, non-matching preservation, non-string preservation, and unknown-key preservation without growing existing test modules past the 400 LOC limit.
- `crates/ares-core/src/options/tests.rs`: register the M177 test module.
- `docs/roadmap.md` and `docs/milestones/m177-print-config-legacy-pattern-migrations.md`: milestone sequencing docs.

## Included legacy rewrites

For these keys, string value `zig-zag` becomes string `rectilinear` (`PrintConfig.cpp:8013-8019`):

- `sparse_infill_pattern`
- `top_surface_pattern`
- `bottom_surface_pattern`
- `internal_solid_infill_pattern`
- `ironing_pattern`
- `support_ironing_pattern`

For covered keys:

- other string values remain unchanged
- non-string values remain unchanged

## Functional requirements

1. Apply the included rewrite when `SliceOptions` is deserialized from JSON.
2. Rewrite only exact string value `zig-zag`; do not rewrite case variants or substrings.
3. Preserve non-matching strings for covered keys unchanged.
4. Preserve non-string values for covered keys unchanged.
5. Preserve non-legacy unknown options exactly as today.
6. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored with migrated values according to the source-cited branch.
7. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
8. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:8020+` in this milestone.

## Acceptance checks

- Tests prove all six covered pattern keys rewrite string `zig-zag` to `rectilinear`.
- Tests prove non-matching string values remain unchanged for covered keys.
- Tests prove non-string values remain unchanged for covered keys.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8020+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
