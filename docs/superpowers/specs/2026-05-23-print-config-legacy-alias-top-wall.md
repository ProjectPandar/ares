# M175 Spec: PrintConfig legacy alias and top-wall slice

## Goal
Port the next legacy alias and conditional top-wall branches from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7992-8004`: migrate legacy infill-anchor, chamber, thumbnail, top-one-wall, and initial-layer-flow key/value handling.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8005+`: `ironing_direction` alias, negative `ironing_angle` migration, `counterbole_hole_bridging` spelling alias, `draft_shield` value migration, pattern migrations, `filament_map_mode`, `filament_type`, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M175 branches.
- `crates/ares-core/src/options/tests/legacy_alias_top_wall.rs`: add focused M175 tests proving covered key renames, conditional `top_one_wall_type` behavior, value preservation, non-string preservation, and unknown-key preservation without growing existing test modules past the 400 LOC limit.
- `crates/ares-core/src/options/tests.rs`: register the M175 test module.
- `docs/roadmap.md` and `docs/milestones/m175-print-config-legacy-alias-top-wall.md`: milestone sequencing docs.

## Included legacy rewrites

Simple key aliases (`PrintConfig.cpp:7992-7999`, `8003-8004`):

- `sparse_infill_anchor` -> `infill_anchor`
- `sparse_infill_anchor_max` -> `infill_anchor_max`
- `chamber_temperatures` -> `chamber_temperature`
- `thumbnail_size` -> `thumbnails`
- `initial_layer_flow_ratio` -> `bottom_solid_infill_flow_ratio`

Conditional top-one-wall migration (`PrintConfig.cpp:8002-8003`):

- if key is `top_one_wall_type` and the value is a string other than `none`, rename the key to `only_one_wall_top` and store string value `1`
- if key is `top_one_wall_type` and the value is string `none`, leave the original key/value unchanged
- if key is `top_one_wall_type` and the value is non-string, leave the original key/value unchanged because this Rust ingestion layer only applies string-value legacy comparisons when the upstream branch compares string content; tests cover boolean, number, null, array, and object representatives

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve values unchanged for all simple aliases.
3. Preserve non-string values for simple aliases under their modern keys.
4. Apply `top_one_wall_type` only for matching string values other than `none`; preserve string `none`, unmatched non-string values, and the original key when the condition does not match.
5. Preserve non-legacy unknown options exactly as today.
6. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored under modern keys/values according to the source-cited branch.
7. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
8. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:8005+` in this milestone.

## Acceptance checks

- Tests prove all covered simple aliases are renamed and preserve their values.
- Tests prove simple aliases preserve non-string values under the renamed keys.
- Tests prove `top_one_wall_type` with a string value other than `none` becomes `only_one_wall_top: "1"` and removes the legacy key.
- Tests prove `top_one_wall_type: "none"` remains unchanged under the legacy key.
- Tests prove non-string `top_one_wall_type` remains unchanged under the legacy key.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8005+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
