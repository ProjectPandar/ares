# M173 Spec: PrintConfig legacy extruder variant value slice

## Goal
Port the nozzle/extruder variant string replacement and `extruder_type` legacy normalization branches from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7959-7970`: for the covered nozzle/extruder variant keys, replace `Normal` with `Standard` and `Big Traffic` with `High Flow`; for `extruder_type`, replace `DirectDrive` with `Direct Drive`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7971+`: power-loss recovery enum migration, shell-thickness migration, rotate solid infill migration, infill-anchor aliases, thumbnail/chamber aliases, top-one-wall migration, pattern migrations, filament type migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M173 string replacement branches.
- `crates/ares-core/src/options/tests/legacy.rs`: add tests proving all covered keys perform the expected string replacements, non-string values are preserved, and unknown keys remain preserved.
- `docs/roadmap.md` and `docs/milestones/m173-print-config-legacy-extruder-variant-values.md`: milestone sequencing docs.

## Included legacy rewrites

For these keys (`PrintConfig.cpp:7959-7967`), replace all occurrences in string values:

- `nozzle_volume_type`
- `default_nozzle_volume_type`
- `printer_extruder_variant`
- `print_extruder_variant`
- `filament_extruder_variant`
- `extruder_variant_list`

Replacements:

- `Normal` -> `Standard`
- `Big Traffic` -> `High Flow`

For `extruder_type` (`PrintConfig.cpp:7968-7970`), replace all occurrences in string values:

- `DirectDrive` -> `Direct Drive`

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve non-string values for covered keys unchanged, because Ares stores JSON values and the upstream branch is string based.
3. Preserve non-legacy unknown options exactly as today.
4. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy string values are stored with source-cited replacements.
5. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
6. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:7971+` in this milestone.

## Acceptance checks

- Tests prove every covered nozzle/extruder variant key rewrites `Normal` to `Standard` and `Big Traffic` to `High Flow` in string values.
- Tests prove replacements apply to multiple occurrences inside a single string.
- Tests prove `extruder_type` rewrites `DirectDrive` to `Direct Drive` in string values.
- Tests prove non-string covered values remain preserved unchanged.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:7971+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
