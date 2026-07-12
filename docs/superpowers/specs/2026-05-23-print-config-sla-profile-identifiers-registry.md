# M161 Spec: PrintConfig SLA profile identifiers registry slice

## Goal
Port the next SLA material/print profile identifier settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7507-7535`: `material_vendor`, default SLA material/print profile identifiers, and SLA settings-id option definitions.

Related upstream behavior explicitly deferred:

- CLI visibility (`nocli`) and UI label/tooltip metadata beyond the current registry metadata boundary.
- SLA profile selection, settings-id resolution, material vendor behavior, and runtime material/profile lookup behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7537+`: `supports_enable`, SLA support/pad settings, and later SLA settings.
- `material_print_speed` from `PrintConfig.cpp:7855-7864`, which is not adjacent to this source slice.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_defaults.rs`: add sorted `default_sla_material_profile` and `default_sla_print_profile` after `default_print_profile` and before `deretraction_speed` in the merged definition order.
- `crates/ares-core/src/options/registry/definitions/table/late_tail_material.rs`: add sorted `material_vendor` after `material_type` and before `max_bridge_length` in the merged definition order.
- `crates/ares-core/src/options/registry/definitions/table/tail_raft_suffix.rs`: add sorted `sla_material_settings_id` and `sla_print_settings_id` after `skirt_type` and before `slice_closing_radius`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add default SLA profile keys in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `material_vendor`, `sla_material_settings_id`, and `sla_print_settings_id` in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_profile_identifiers.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_profile_identifiers.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 5.
- `docs/roadmap.md` and `docs/milestones/m161-print-config-sla-profile-identifiers-registry.md`: milestone sequencing docs.

## Included option definitions

- `material_vendor` (`coString`, default empty string, definition lines 7507-7511, Ares kind `String`)
- `default_sla_material_profile` (`coString`, default empty string, definition lines 7513-7517, Ares kind `String`)
- `sla_material_settings_id` (`coString`, default empty string, definition lines 7519-7523, Ares kind `String`)
- `default_sla_print_profile` (`coString`, default empty string, definition lines 7525-7529, Ares kind `String`)
- `sla_print_settings_id` (`coString`, default empty string, definition lines 7531-7535, Ares kind `String`)

## Explicit non-included adjacent behavior

- `supports_enable` beginning at `PrintConfig.cpp:7537` is deferred to a later source-cited SLA support milestone.
- SLA support-head/support-pad settings following `supports_enable` are deferred.
- `material_print_speed` from `PrintConfig.cpp:7855-7864` is not adjacent to this source slice and remains deferred.
- Runtime profile selection and settings-id resolution behavior is deferred.

## Functional requirements

1. Add the 5 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA profile-selection behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `supports_enable` or later SLA support settings from `PrintConfig.cpp:7537+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA profile behavior and `PrintConfig.cpp:7537+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
