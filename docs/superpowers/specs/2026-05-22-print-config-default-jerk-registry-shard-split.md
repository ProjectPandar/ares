# M77 Spec: PrintConfig default jerk registry with pre-middle shard split

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` default jerk and default junction deviation option-definition slice into `ares-core` option registry metadata by adding registry coverage for `default_jerk` and `default_junction_deviation`, and split the oversized pre-middle registry shard to keep modified Rust files under 400 LOC.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1052`: `default_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3169-3176`: `default_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1060`: `default_junction_deviation` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3178-3186`: `default_junction_deviation` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Default jerk runtime behavior and junction-deviation runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3188-3249`: wall/infill/travel jerk options already covered by M76.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3251+`: `initial_layer_line_width`, `initial_layer_print_height`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: add the new shard modules to the compile-time merge in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: keep only the leading `complete_*`, `cool_*`, `counterbore_*`, and `curr_*` definitions so it stays below 400 LOC.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_defaults.rs`: new shard containing sorted `default_*` definitions, including new `default_jerk` and `default_junction_deviation`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: new shard containing the existing `different_*`, `dont_*`, `during_*`, `elefant_*`, `enable_*`, `eng_*`, `ensure_*`, `extra_*`, `extruder_*`, `fan_*`, and `filament_*` definitions through `filament_printable`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: extend source metadata assertions for the two options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_speed.rs`: extend public lookup coverage for the two options.
- `docs/roadmap.md` and `docs/milestones/m77-print-config-default-jerk-registry-shard-split.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `default_jerk` (`coFloat`, default `0`, field at `PrintConfig.hpp:1052`, definition lines 3169-3176, Ares kind `Float`)
- `default_junction_deviation` (`coFloat`, default `0`, field at `PrintConfig.hpp:1060`, definition lines 3178-3186, Ares kind `Float`)

## Functional requirements

1. Add the missing options to the sorted registry stream using `Float`.
2. Split `pre_middle.rs` into smaller sorted shards without changing existing option definitions, kinds, defaults, or source citations.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, default jerk behavior, junction-deviation behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `initial_layer_line_width`, `initial_layer_print_height`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, and GUI metadata from `PrintConfig.cpp:3169-3186` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Default jerk behavior, junction-deviation behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- Wall/infill/top/travel jerk options from `PrintConfig.cpp:3188-3249` remain owned by M76.
- `initial_layer_line_width`, `initial_layer_print_height`, and following options from `PrintConfig.cpp:3251+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for both new keys.
- Existing moved definitions remain byte-for-byte equivalent except for file location.
- Plan/spec explicitly account for deferred UI/bounds metadata, default jerk/junction-deviation behavior, slicing/extrusion/G-code behavior, and following initial-layer line-width scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
