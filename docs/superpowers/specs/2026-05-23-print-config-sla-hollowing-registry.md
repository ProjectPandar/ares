# M167 Spec: PrintConfig SLA hollowing registry slice

## Goal
Port the SLA model-hollowing settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1791-1802`, `PrintConfig.cpp:7819-7853`: SLA hollowing enable, minimum wall thickness, hollowing quality, and closing-distance option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/sidetext/min/max/mode metadata beyond the current registry metadata boundary.
- SLA hollowing runtime behavior, cavity generation, wall-thickness enforcement, drain-hole behavior, OpenVDB/voxel behavior, and runtime geometry changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7855+`: `material_print_speed` and later SLA material settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add the 4 covered `hollowing_*` definitions in lexicographic order after `hole_to_polyhole_twisted` and before `host_type`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add the 4 covered `hollowing_*` keys in lexicographic order after `hole_to_polyhole_twisted` and before `host_type`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_hollowing.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_hollowing.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add fixture values for the covered keys near the existing `hole_to_polyhole*` / `host_type` values.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 4.
- `docs/roadmap.md` and `docs/milestones/m167-print-config-sla-hollowing-registry.md`: milestone sequencing docs.

## Included option definitions

- `hollowing_enable` (`coBool`, default `false`, field at `PrintConfig.hpp:1791`, definition lines 7819-7824, Ares kind `Bool`)
- `hollowing_min_thickness` (`coFloat`, default `3.`, field at `PrintConfig.hpp:1796`, definition lines 7826-7834, Ares kind `Float`, registry default string `3`)
- `hollowing_quality` (`coFloat`, default `0.5`, field at `PrintConfig.hpp:1799`, definition lines 7836-7843, Ares kind `Float`)
- `hollowing_closing_distance` (`coFloat`, default `2.0`, field at `PrintConfig.hpp:1802`, definition lines 7845-7853, Ares kind `Float`, registry default string `2`)

## Explicit non-included adjacent behavior

- `material_print_speed` beginning at `PrintConfig.cpp:7855` is deferred to a later source-cited SLA material-speed milestone.
- Runtime hollowing behavior, cavity generation, wall-thickness enforcement, drain-hole behavior, OpenVDB/voxel behavior, and geometry modification are deferred.
- UI min/max/mode/sidetext metadata remains deferred until the registry metadata boundary expands.

## Functional requirements

1. Add the 4 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, hollowing runtime behavior, geometry changes, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `material_print_speed` or later SLA settings from `PrintConfig.cpp:7855+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; current destination shards have enough room and should not require a split.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred hollowing runtime behavior and `PrintConfig.cpp:7855+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
