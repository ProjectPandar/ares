# M170 Spec: PrintConfig legacy simple value migration slice

## Goal
Port the next simple legacy-option normalization slice from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7900-7932`: percentage-valued legacy key erasure, cumulative/cooling/timelapse key aliases, and simple timelapse/support value migrations.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7933+`: recursive `different_settings_to_system` normalization, `overhang_fan_threshold`, `wall_infill_order`, nozzle/extruder variant value replacements, power-loss recovery enum migration, shell-thickness migration, infill-anchor aliases, thumbnail/chamber aliases, pattern migrations, filament type migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing M169 deserialization normalization so each incoming key/value follows the same ordered single-branch semantics as Orca's `else if` chain.
- `crates/ares-core/src/options/tests/legacy.rs`: add tests proving the covered percentage erasures, key aliases, value migrations, ordered alias/value semantics, and unknown-key preservation.
- `docs/roadmap.md` and `docs/milestones/m170-print-config-legacy-simple-value-migrations.md`: milestone sequencing docs.

## Included legacy rewrites

Percentage-valued key erasure (`PrintConfig.cpp:7900-7909`): if the incoming value is a string containing `%`, omit the option instead of storing it under either legacy or modern key:

- `initial_layer_print_height`
- `initial_layer_speed`
- `internal_solid_infill_speed`
- `top_surface_speed`
- `support_interface_speed`
- `outer_wall_speed`
- `support_object_xy_distance`

Key rewrites:

- `inherits_cummulative` -> `inherits_group` (`PrintConfig.cpp:7911-7912`)
- `compatible_printers_condition_cummulative` -> `compatible_machine_expression_group` (`PrintConfig.cpp:7913-7914`)
- `compatible_prints_condition_cummulative` -> `compatible_process_expression_group` (`PrintConfig.cpp:7915-7916`)
- `cooling` -> `slow_down_for_layer_cooling` (`PrintConfig.cpp:7917-7918`)
- `timelapse_no_toolhead` -> `timelapse_type` (`PrintConfig.cpp:7919-7920`)

Value rewrites:

- `timelapse_type` with value `2` becomes `0` (`PrintConfig.cpp:7921-7924`)
- `support_type` with value `normal` becomes `normal(manual)` (`PrintConfig.cpp:7925-7926`)
- `support_type` with value `tree` becomes `tree(manual)` (`PrintConfig.cpp:7927-7928`)
- `support_type` with value `hybrid(auto)` becomes `tree(auto)` (`PrintConfig.cpp:7929-7930`)
- `support_base_pattern` with value `none` becomes `hollow` (`PrintConfig.cpp:7931-7932`)

Although the final `support_base_pattern` branch reaches line 7932, include it in M170 because it is the last contiguous simple support value branch before `different_settings_to_system`; defer the recursive branch beginning at `PrintConfig.cpp:7933`.

## Ordered branch semantics

Orca uses one `if`/`else if` chain. Therefore a key that matches an alias branch must not immediately trigger a later value branch in the same call. For this milestone, `timelapse_no_toolhead: "2"` must become `timelapse_type: "2"`, not `timelapse_type: "0"`.

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve non-legacy unknown options exactly as today.
3. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are omitted or stored under modern keys/values according to the source-cited branch.
4. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
5. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:7933+` in this milestone.

## Acceptance checks

- Tests prove all covered percentage-valued legacy keys are omitted when their JSON value is a string containing `%`.
- Tests prove those same keys are preserved unchanged when their JSON value is not a percentage string.
- Tests prove all covered key aliases deserialize under modern names with values preserved.
- Tests prove all covered value migrations rewrite only the listed values.
- Tests prove ordered branch semantics with `timelapse_no_toolhead: "2"` preserving value `2` after key aliasing.
- Tests prove unknown non-legacy keys remain preserved.
- Existing registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:7933+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
