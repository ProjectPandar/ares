# M92 Spec: PrintConfig custom G-code, machine limit flag, and small-area flow registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` custom G-code, machine-limit emission flag, small-area infill flow compensation, and scarf-seam marker option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1358`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4295-4302`: `layer_change_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1359`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4304-4310`: `time_lapse_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1360`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4312-4318`: `wrapping_detection_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1398`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4320-4324`: `silent_mode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1247`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4326-4332`: `emit_machine_limits_to_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1399`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4334-4341`: `machine_pause_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1400`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4343-4350`: `template_custom_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1211`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4352-4357`: `small_area_infill_flow_compensation` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1464`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4359-4371`: `small_area_infill_flow_compensation_model` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1466`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4373-4375`: `has_scarf_joint_seam` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/category/sidetext/min/max/mode/gui-flags/multiline/full-width/height metadata beyond the current registry boundary.
- Custom G-code insertion or template execution behavior.
- Machine-limit emission behavior.
- Small-area infill flow compensation behavior.
- Scarf-seam detection or G-code processor behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4377+`: machine axis limit loop and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definition for `emit_machine_limits_to_gcode`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definition for `has_scarf_joint_seam`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for `layer_change_gcode` and `machine_pause_gcode`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `silent_mode`, `small_area_infill_flow_compensation`, and `small_area_infill_flow_compensation_model`, then keep the shard below the 400 LOC threshold.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: split later sorted tail definitions out of `tail.rs` and add `template_custom_gcode`, `time_lapse_gcode`, and `wrapping_detection_gcode` in sorted order without changing unrelated moved metadata.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merge the new `tail_final` shard after `tail` to preserve sorted lookup order.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod custom_gcode;`.
- `crates/ares-core/src/options/registry/tests/metadata/custom_gcode.rs`: source metadata assertions for all ten options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_custom_gcode;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_custom_gcode.rs`: public lookup coverage for all ten options.
- `docs/roadmap.md` and `docs/milestones/m92-print-config-custom-gcode-small-area-registry.md`: milestone sequencing docs.

## Included option definitions

- `layer_change_gcode` (`coString`, default `""`, field at `PrintConfig.hpp:1358`, definition lines 4295-4302, Ares kind `String`)
- `time_lapse_gcode` (`coString`, default `""`, field at `PrintConfig.hpp:1359`, definition lines 4304-4310, Ares kind `String`)
- `wrapping_detection_gcode` (`coString`, default `""`, field at `PrintConfig.hpp:1360`, definition lines 4312-4318, Ares kind `String`)
- `silent_mode` (`coBool`, default `false`, field at `PrintConfig.hpp:1398`, definition lines 4320-4324, Ares kind `Bool`)
- `emit_machine_limits_to_gcode` (`coBool`, default `true`, field at `PrintConfig.hpp:1247`, definition lines 4326-4332, Ares kind `Bool`)
- `machine_pause_gcode` (`coString`, default `""`, field at `PrintConfig.hpp:1399`, definition lines 4334-4341, Ares kind `String`)
- `template_custom_gcode` (`coString`, default `""`, field at `PrintConfig.hpp:1400`, definition lines 4343-4350, Ares kind `String`)
- `small_area_infill_flow_compensation` (`coBool`, default `false`, field at `PrintConfig.hpp:1211`, definition lines 4352-4357, Ares kind `Bool`)
- `small_area_infill_flow_compensation_model` (`coStrings`, default `0,0\n0.2,0.4444\n0.4,0.6145\n0.6,0.7059\n0.8,0.7619\n1.5,0.8571\n2,0.8889\n3,0.9231\n5,0.9520\n10,1`, field at `PrintConfig.hpp:1464`, definition lines 4359-4371, Ares kind `Strings`)
- `has_scarf_joint_seam` (`coBool`, default `false`, field at `PrintConfig.hpp:1466`, definition lines 4373-4375, Ares kind `Bool`)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, custom G-code insertion behavior, machine-limit emission behavior, small-area flow compensation behavior, scarf-seam behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter machine axis limit loop options from `PrintConfig.cpp:4377+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC by splitting `tail` into a focused `tail_final` shard when M92 pushes the existing file over the limit; create focused tests instead of growing unrelated near-limit files.

## Acceptance checks

- Registry tests prove all ten new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all ten new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following machine-axis-limit-loop scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
