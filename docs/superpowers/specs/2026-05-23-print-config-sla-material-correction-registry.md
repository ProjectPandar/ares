# M160 Spec: PrintConfig SLA material correction registry slice

## Goal
Port the next SLA material correction settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7479-7505`: `material_correction`, `material_correction_x`, `material_correction_y`, and `material_correction_z` option definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1817-1820`: `SLAMaterialConfig` material correction fields.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/mode/min metadata beyond the current registry metadata boundary.
- SLA material correction/scaling runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7507+`: `material_vendor`, default SLA profile identifiers, `material_print_speed`, SLA support/pad settings, and later SLA settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: mechanically keep only the sorted prefix through `master_extruder_id`, moving existing material definitions and following max/min definitions out to smaller shards so this file stays below 400 LOC.
- Create `crates/ares-core/src/options/registry/definitions/table/late_tail_material.rs`: move existing `material_colour`, `material_density`, and `material_type` here without changing metadata; add sorted `material_correction`, `material_correction_x`, `material_correction_y`, and `material_correction_z` after `material_colour` and before `material_density`.
- Create `crates/ares-core/src/options/registry/definitions/table/late_tail_after_material.rs`: move existing `max_bridge_length` and following `late_tail` definitions here without changing metadata.
- Modify `crates/ares-core/src/options/registry/definitions/table.rs`: merge `late_tail`, `late_tail_material`, and `late_tail_after_material` in that order before `late_tail_final`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add material correction keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_material_correction.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_material_correction.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 4.
- `docs/roadmap.md` and `docs/milestones/m160-print-config-sla-material-correction-registry.md`: milestone sequencing docs.

## Included option definitions

- `material_correction` (`coFloats`, default `{1., 1., 1.}`, field at `PrintConfig.hpp:1817`, definition lines 7479-7484, Ares kind `Floats`, default string `1` consistent with existing `relative_correction` metadata)
- `material_correction_x` (`coFloat`, default `1.`, field at `PrintConfig.hpp:1818`, definition lines 7486-7491, Ares kind `Float`)
- `material_correction_y` (`coFloat`, default `1.`, field at `PrintConfig.hpp:1819`, definition lines 7493-7498, Ares kind `Float`)
- `material_correction_z` (`coFloat`, default `1.`, field at `PrintConfig.hpp:1820`, definition lines 7500-7505, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `material_vendor`, `default_sla_material_profile`, `sla_material_settings_id`, `default_sla_print_profile`, and `sla_print_settings_id` beginning at `PrintConfig.cpp:7507` are deferred to later source-cited milestones.
- `material_print_speed` from `PrintConfig.cpp:7855-7864` is not adjacent to this source slice and remains deferred.
- SLA material correction/scaling runtime behavior is deferred.

## Functional requirements

1. Add the 4 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA material correction behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `material_vendor` or later SLA settings from `PrintConfig.cpp:7507+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- Existing material identity metadata remains unchanged after the mechanical shard split.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA material correction behavior and `PrintConfig.cpp:7507+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
