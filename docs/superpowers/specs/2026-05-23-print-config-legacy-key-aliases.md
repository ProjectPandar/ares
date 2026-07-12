# M169 Spec: PrintConfig legacy key alias slice

## Goal
Port the first simple legacy-option normalization slice from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7867-7899`: simple legacy value/key rewrites at the start of `PrintConfigDef::handle_legacy`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7900+`: percentage-value erasure for now-absolute options, cumulative-key renames, cooling/timelapse/support enum value migrations, recursive `different_settings_to_system` normalization, and later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options.rs` and `crates/ares-core/src/options/legacy.rs`: replace derived `Deserialize` for `SliceOptions` with a manual deserialize path that loads the incoming JSON object into a `BTreeMap<String, Value>` and applies only the covered legacy value/key rewrites before storing `values`.
- `crates/ares-core/src/options/tests/legacy.rs`: add tests proving the covered aliases normalize during `SliceOptions` deserialization and unknown non-legacy keys remain preserved.
- `docs/roadmap.md` and `docs/milestones/m169-print-config-legacy-key-aliases.md`: milestone sequencing docs.

## Included legacy rewrites

Value rewrite:

- `curr_bed_type` with value `SuperTack Plate` becomes `Supertack Plate` (`PrintConfig.cpp:7870-7871`).

Key rewrites:

- `enable_wipe_tower` -> `enable_prime_tower` (`PrintConfig.cpp:7872-7873`)
- `wipe_tower_width` -> `prime_tower_width` (`PrintConfig.cpp:7874-7875`)
- `wiping_volume` -> `prime_volume` (`PrintConfig.cpp:7876-7877`)
- `wipe_tower_brim_width` -> `prime_tower_brim_width` (`PrintConfig.cpp:7878-7879`)
- `tool_change_gcode` -> `change_filament_gcode` (`PrintConfig.cpp:7880-7881`)
- `bridge_fan_speed` -> `overhang_fan_speed` (`PrintConfig.cpp:7882-7883`)
- `infill_extruder` -> `sparse_infill_filament` (`PrintConfig.cpp:7884-7885`)
- `solid_infill_extruder` -> `solid_infill_filament` (`PrintConfig.cpp:7886-7887`)
- `perimeter_extruder` -> `wall_filament` (`PrintConfig.cpp:7888-7889`)
- `wipe_tower_extruder` -> `wipe_tower_filament` (`PrintConfig.cpp:7890-7891`)
- `support_material_extruder` -> `support_filament` (`PrintConfig.cpp:7892-7893`)
- `support_material_interface_extruder` -> `support_interface_filament` (`PrintConfig.cpp:7894-7895`)
- `support_material_angle` -> `support_angle` (`PrintConfig.cpp:7896-7897`)
- `support_material_enforce_layers` -> `enforce_support_layers` (`PrintConfig.cpp:7898-7899`)

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve non-legacy unknown options exactly as today.
3. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored under modern keys/values.
4. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
5. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:7900+` in this milestone.

## Acceptance checks

- Tests prove `curr_bed_type: "SuperTack Plate"` deserializes as `curr_bed_type: "Supertack Plate"`.
- Tests prove all covered legacy keys deserialize under their modern names with values preserved.
- Tests prove unknown non-legacy keys remain preserved.
- Existing registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:7900+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
