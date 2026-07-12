# M159 Spec: PrintConfig SLA exposure time registry slice

## Goal
Port the next SLA faded-layer and exposure-time settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7425-7477`: `faded_layers`, exposure bounds, exposure time, initial exposure bounds, and initial exposure time option definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1815-1816`: `SLAMaterialConfig` exposure fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1848-1851`: `SLAPrinterConfig` exposure bound fields.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/mode/min/max metadata beyond the current registry metadata boundary.
- SLA exposure timing, faded-layer blending, and runtime material/printer exposure behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7479+`: `material_correction`, material profile identifiers, SLA support/pad settings, and later SLA settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted `exposure_time` after `exclude_object` and before `extra_loading_move`; add sorted `faded_layers` after `extrusion_rate_smoothing_external_perimeter_only` and before `fan_cooling_layer_time`.
- `crates/ares-core/src/options/registry/definitions/table/middle_independent.rs`: add sorted `initial_exposure_time` after `inherits_group` and before `initial_layer_acceleration`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add sorted `max_exposure_time` and `max_initial_exposure_time` after `max_bridge_length` and before `max_layer_height`; add sorted `min_exposure_time` after `min_bead_width` and before `min_feature_size`; add sorted `min_initial_exposure_time` after `min_feature_size` and before `min_layer_height`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add `exposure_time` and `faded_layers` in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add exposure bound keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_exposure_time.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_exposure_time.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 7.
- `docs/roadmap.md` and `docs/milestones/m159-print-config-sla-exposure-time-registry.md`: milestone sequencing docs.

## Included option definitions

- `faded_layers` (`coInt`, default `10`, definition lines 7425-7431, Ares kind `Int`)
- `min_exposure_time` (`coFloat`, default `0`, field at `PrintConfig.hpp:1848`, definition lines 7433-7439, Ares kind `Float`)
- `max_exposure_time` (`coFloat`, default `100`, field at `PrintConfig.hpp:1849`, definition lines 7441-7447, Ares kind `Float`)
- `exposure_time` (`coFloat`, default `10`, field at `PrintConfig.hpp:1815`, definition lines 7449-7454, Ares kind `Float`)
- `min_initial_exposure_time` (`coFloat`, default `0`, field at `PrintConfig.hpp:1850`, definition lines 7456-7462, Ares kind `Float`)
- `max_initial_exposure_time` (`coFloat`, default `150`, field at `PrintConfig.hpp:1851`, definition lines 7464-7470, Ares kind `Float`)
- `initial_exposure_time` (`coFloat`, default `15`, field at `PrintConfig.hpp:1816`, definition lines 7472-7477, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `material_correction`, `material_correction_x`, `material_correction_y`, and `material_correction_z` beginning at `PrintConfig.cpp:7479` are deferred to a later source-cited milestone.
- `material_vendor`, default SLA profile identifiers, SLA support settings, and later SLA settings from `PrintConfig.cpp:7507+` are deferred.

## Functional requirements

1. Add the 7 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA exposure behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `material_correction` or later SLA settings from `PrintConfig.cpp:7479+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA exposure behavior, material correction behavior, and `PrintConfig.cpp:7479+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
