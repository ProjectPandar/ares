# M156 Spec: PrintConfig SLA axis and absolute correction registry slice

## Goal
Port the axis-specific and absolute SLA correction options from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1838-1841`: `SLAPrinterConfig` correction fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7320-7349`: `relative_correction_x`, `relative_correction_y`, `relative_correction_z`, and `absolute_correction` option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min metadata beyond the current registry metadata boundary.
- SLA correction/scaling runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7351+`: `elefant_foot_min_width`, `gamma_correction`, and later SLA material settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add `absolute_correction` before `accel_to_decel_enable`.
- `crates/ares-core/src/options/registry/definitions/table/tail_raft.rs`: keep entries through `relative_correction`, then add `relative_correction_x`, `relative_correction_y`, and `relative_correction_z` after `relative_correction`.
- `crates/ares-core/src/options/registry/definitions/table/tail_raft_suffix.rs`: move existing `required_nozzle_HRC` and later `tail_raft` entries into this new suffix shard so `tail_raft.rs` stays below 400 LOC.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add `absolute_correction` in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add relative correction axis keys in sorted order.
- `crates/ares-core/src/options/registry/definitions/table.rs`: wire `tail_raft_suffix` immediately after `tail_raft` in the merge order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and a new metadata core shard: move existing root metadata tests out of `metadata.rs` so the module list can accept another module without reaching 400 LOC.
- `crates/ares-core/src/options/registry/tests/metadata/sla_axis_absolute_correction.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_axis_absolute_correction.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 4.
- `docs/roadmap.md` and `docs/milestones/m156-print-config-sla-axis-absolute-correction-registry.md`: milestone sequencing docs.

## Included option definitions

- `relative_correction_x` (`coFloat`, default `1.`, field at `PrintConfig.hpp:1838`, definition lines 7320-7326, Ares kind `Float`)
- `relative_correction_y` (`coFloat`, default `1.`, field at `PrintConfig.hpp:1839`, definition lines 7328-7334, Ares kind `Float`)
- `relative_correction_z` (`coFloat`, default `1.`, field at `PrintConfig.hpp:1840`, definition lines 7336-7342, Ares kind `Float`)
- `absolute_correction` (`coFloat`, default `0.0`, field at `PrintConfig.hpp:1841`, definition lines 7344-7349, Ares kind `Float`)

## Functional requirements

1. Add the 4 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA correction behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `elefant_foot_min_width` or later SLA settings from `PrintConfig.cpp:7351+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; split registry and metadata fixtures as needed.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA runtime behavior and `PrintConfig.cpp:7351+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
