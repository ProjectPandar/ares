# M158 Spec: PrintConfig SLA material identity and cost registry slice

## Goal
Port the first SLA material identity/cost settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7370-7423`: SLA material colour/type, bottle volume/weight/cost, and material density option definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1811-1814`: `SLAMaterialConfig` bottle and density fields.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/gui type/gui flags/enum value metadata beyond the current registry metadata boundary.
- SLA material identity, density, and cost runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- Existing `initial_layer_height` behavior; it is already represented in `ares-core` from `PrintConfig.cpp:7390-7395`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7425+`: `faded_layers`, exposure settings, material correction, material print speed, and later SLA support/pad settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted `bottle_cost`, `bottle_volume`, and `bottle_weight` after `best_object_pos` and before `bottom_shell_layers`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add sorted `material_colour`, `material_density`, and `material_type` after `master_extruder_id` and before `max_bridge_length`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add bottle keys in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add material keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_material_identity_cost.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_material_identity_cost.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 6.
- `docs/roadmap.md` and `docs/milestones/m158-print-config-sla-material-identity-cost-registry.md`: milestone sequencing docs.

## Included option definitions

- `material_colour` (`coString`, default `#29B2B2`, definition lines 7372-7376, Ares kind `String`)
- `material_type` (`coString`, default `Tough`, definition lines 7378-7388, Ares kind `String`)
- `bottle_volume` (`coFloat`, default `1000.0`, field at `PrintConfig.hpp:1812`, definition lines 7397-7402, Ares kind `Float`)
- `bottle_weight` (`coFloat`, default `1.0`, field at `PrintConfig.hpp:1813`, definition lines 7404-7409, Ares kind `Float`)
- `material_density` (`coFloat`, default `1.0`, field at `PrintConfig.hpp:1814`, definition lines 7411-7416, Ares kind `Float`)
- `bottle_cost` (`coFloat`, default `0.0`, field at `PrintConfig.hpp:1811`, definition lines 7418-7423, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `initial_layer_height` at `PrintConfig.cpp:7390-7395` is not redefined in this milestone because `ares-core` already has that exact SLA material option metadata.
- `faded_layers` and exposure/material correction settings beginning at `PrintConfig.cpp:7425` are deferred to later source-cited milestones.

## Functional requirements

1. Add the 6 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA material behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `faded_layers` or later SLA settings from `PrintConfig.cpp:7425+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA runtime behavior and `PrintConfig.cpp:7425+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
