# M179 Spec: PrintConfig legacy prime-tower rib and hardware slice

## Goal
Port the prime-tower rib aliases, clearance/tool-change aliases, and `wall_direction` legacy value branch from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8046-8067`: migrate prime-tower rib keys, `extruder_clearance_max_radius`, `machine_switch_extruder_time`, and `wall_direction: auto`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8069+`: obsolete-key ignore set, final `print_config_def.has(opt_key)` validation, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M179 branches.
- `crates/ares-core/src/options/tests/legacy_prime_tower_rib_hardware.rs`: add focused M179 tests proving included key renames, conditional drop, value migrations, non-matching preservation, non-string preservation, and unknown-key preservation without growing existing test modules past the 400 LOC limit.
- `crates/ares-core/src/options/tests.rs`: register the M179 test module.
- `docs/roadmap.md` and `docs/milestones/m179-print-config-legacy-prime-tower-rib-hardware.md`: milestone sequencing docs.

## Included legacy rewrites

`prime_tower_rib_wall` (`PrintConfig.cpp:8047-8053`):

- string `1` becomes key `wipe_tower_wall_type` with string value `rib`
- every other value is dropped because upstream sets `opt_key` to an empty string

Simple key aliases (`PrintConfig.cpp:8054-8064`):

- `prime_tower_extra_rib_length` -> `wipe_tower_extra_rib_length`
- `prime_tower_rib_width` -> `wipe_tower_rib_width`
- `prime_tower_fillet_wall` -> `wipe_tower_fillet_wall`
- `extruder_clearance_max_radius` -> `extruder_clearance_radius`
- `machine_switch_extruder_time` -> `machine_tool_change_time`

`wall_direction` (`PrintConfig.cpp:8065-8067`):

- string `auto` becomes string `ccw`
- other strings remain unchanged
- non-string values remain unchanged

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Drop `prime_tower_rib_wall` unless its value is exactly string `1`.
3. Preserve values unchanged for simple aliases.
4. Preserve non-string values for simple aliases under their modern keys.
5. Apply `wall_direction` migration only to exact string value `auto`.
6. Preserve non-legacy unknown options exactly as today.
7. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored with migrated keys/values according to the source-cited branch.
8. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
9. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:8069+` in this milestone.

## Acceptance checks

- Tests prove `prime_tower_rib_wall: "1"` becomes `wipe_tower_wall_type: "rib"` and removes the legacy key.
- Tests prove `prime_tower_rib_wall` values other than string `1`, including non-string values, are dropped.
- Tests prove all five simple aliases are renamed and preserve values.
- Tests prove simple aliases preserve non-string values under modern keys.
- Tests prove `wall_direction: "auto"` becomes `ccw` while other strings and non-string values remain unchanged.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8069+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
