# M157 Spec: PrintConfig SLA foot and gamma correction registry slice

## Goal
Port the SLA minimum elephant-foot width and gamma correction options from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1843-1844`: `SLAPrinterConfig` `elefant_foot_min_width` and `gamma_correction` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7351-7367`: `elefant_foot_min_width` and `gamma_correction` option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max metadata beyond the current registry metadata boundary.
- SLA elephant-foot and gamma correction runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7370+`: SLA material settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `elefant_foot_min_width` after `elefant_foot_layers_density` and before `emit_machine_limits_to_gcode`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add `gamma_correction` after `fuzzy_skin_thickness` and before `gap_fill_flow_ratio`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add both covered keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_foot_gamma.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_foot_gamma.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 2.
- `docs/roadmap.md` and `docs/milestones/m157-print-config-sla-foot-gamma-registry.md`: milestone sequencing docs.

## Included option definitions

- `elefant_foot_min_width` (`coFloat`, default `0.2`, field at `PrintConfig.hpp:1843`, definition lines 7351-7358, Ares kind `Float`)
- `gamma_correction` (`coFloat`, default `1.0`, field at `PrintConfig.hpp:1844`, definition lines 7360-7367, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `elefant_foot_compensation` at `PrintConfig.hpp:1842` is not redefined in this milestone because `ares-core` already has the shared `elefant_foot_compensation` option metadata from `PrintConfig.cpp:717-724`.
- SLA material settings beginning at `PrintConfig.cpp:7370` are deferred to later source-cited milestones.

## Functional requirements

1. Add the 2 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA foot/gamma behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add SLA material settings from `PrintConfig.cpp:7370+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA runtime behavior and `PrintConfig.cpp:7370+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
