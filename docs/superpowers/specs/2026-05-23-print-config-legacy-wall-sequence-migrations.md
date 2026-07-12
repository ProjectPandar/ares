# M172 Spec: PrintConfig legacy wall sequence migration slice

## Goal
Port the `overhang_fan_threshold` and `wall_infill_order` legacy normalization branches from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7944-7958`: rewrite `overhang_fan_threshold == "5%"` to `"10%"`, and rename `wall_infill_order` to `wall_sequence` with value migrations for legacy wall/infill ordering strings.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7959+`: nozzle/extruder variant value replacements, extruder type migration, power-loss recovery enum migration, shell-thickness migration, infill-anchor aliases, thumbnail/chamber aliases, top-one-wall migration, pattern migrations, filament type migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M172 `overhang_fan_threshold` and `wall_infill_order` branches.
- `crates/ares-core/src/options/tests/legacy.rs`: add tests proving value rewrite, key rename, covered wall sequence value migrations, fallback key-only wall sequence rename, and unknown-key preservation.
- `docs/roadmap.md` and `docs/milestones/m172-print-config-legacy-wall-sequence-migrations.md`: milestone sequencing docs.

## Included legacy rewrites

Value rewrite:

- `overhang_fan_threshold` with value `5%` becomes `10%` (`PrintConfig.cpp:7944-7945`).

Key/value rewrites for `wall_infill_order` (`PrintConfig.cpp:7946-7958`):

- `inner wall/outer wall/infill` -> key `wall_sequence`, value `inner wall/outer wall`
- `infill/inner wall/outer wall` -> key `wall_sequence`, value `inner wall/outer wall`
- `outer wall/inner wall/infill` -> key `wall_sequence`, value `outer wall/inner wall`
- `infill/outer wall/inner wall` -> key `wall_sequence`, value `outer wall/inner wall`
- `inner-outer-inner wall/infill` -> key `wall_sequence`, value `inner-outer-inner wall`
- any other `wall_infill_order` value -> key `wall_sequence`, value preserved unchanged

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve non-legacy unknown options exactly as today.
3. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored under modern keys/values according to the source-cited branch.
4. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
5. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:7959+` in this milestone.

## Acceptance checks

- Tests prove `overhang_fan_threshold: "5%"` deserializes as `"10%"`.
- Tests prove other `overhang_fan_threshold` values remain preserved unchanged.
- Tests prove all covered `wall_infill_order` values deserialize under `wall_sequence` with the expected values.
- Tests prove an unlisted `wall_infill_order` value still deserializes under `wall_sequence` with value preserved.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:7959+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
