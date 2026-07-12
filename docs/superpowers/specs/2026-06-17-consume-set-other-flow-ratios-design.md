# Consume Set Other Flow Ratios Design

## Goal

Consume OrcaSlicer `set_other_flow_ratios` in Ares extrusion generation so already parsed "other flow ratio" options are gated by the same upstream boolean before changing G-code E output.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:978` declares `set_other_flow_ratios` as a `ConfigOptionBool` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1307-1312` registers `set_other_flow_ratios` with default `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1214-1221` groups `first_layer_flow_ratio`, `outer_wall_flow_ratio`, `inner_wall_flow_ratio`, `overhang_flow_ratio`, `sparse_infill_flow_ratio`, `internal_solid_infill_flow_ratio`, and `gap_fill_flow_ratio` as "other flow ratios" available for overriding when `set_other_flow_ratios` is enabled.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6415-6431` applies wall, overhang, sparse infill, internal solid infill, gap fill, and support flow ratio multipliers only inside `if (m_config.set_other_flow_ratios)`.

## Ares Destination Boundary

- `crates/ares-core/src/options/flow_ratios.rs`: parse the existing `set_other_flow_ratios` bool and apply it as a gate for Ares' currently supported other-flow roles.
- `crates/ares-core/src/options.rs`: wire `SliceOptions::extrusion_options` through the existing `flow_ratios` helper without growing the file beyond 400 LOC.
- `crates/ares-core/src/options/tests/set_other_flow_ratios.rs`: add option-to-extrusion tests proving the gate default, explicit false, explicit true, and invalid type behavior without growing existing large test files.
- `crates/ares-core/src/options/tests.rs`: register the new option test module by editing the existing final `option_test_modules!(...)` line so the file remains at 400 LOC.
- `crates/ares-core/src/pipeline/tests/set_other_flow_ratios.rs`: add G-code-facing regressions proving first-layer perimeter, wall, and sparse infill flow ratios affect E deltas only when the gate is enabled.
- `crates/ares-core/src/pipeline/tests.rs`: register the focused pipeline test module.

## Included Behavior

- Omitted `set_other_flow_ratios` defaults to `false`, matching Orca.
- Explicit `set_other_flow_ratios: false` disables Ares' currently supported other-flow multipliers: `first_layer_flow_ratio`, `outer_wall_flow_ratio`, `inner_wall_flow_ratio`, and `sparse_infill_flow_ratio`.
- Explicit `set_other_flow_ratios: true` enables the existing first-layer, wall, and sparse infill role multipliers.
- Invalid non-boolean `set_other_flow_ratios` values are rejected through `SliceOptions::extrusion_options`.
- The gate does not affect `brim_flow_ratio`, `bridge_flow`, `internal_bridge_flow`, or `print_flow_ratio`, because those are outside Orca's `set_other_flow_ratios` branch in the cited G-code boundary.
- Existing numeric validation for gated ratios remains strict even when the gate is false, because parsing invalid user input at the option boundary stays unchanged.

## Docs Impact

No user-facing documentation changes are required. This slice changes internal option consumption semantics to match the cited Orca G-code boundary and records the design/plan under `docs/superpowers`.

## Deferred Behavior

- Ares still does not implement overhang, internal solid infill, gap fill, support, or support interface print roles in this slice.
- This slice does not add new flow ratio options or geometry generators.
- This slice does not change brim, bridge, print-wide, object, filament, UI, or viewer behavior.
- This slice does not model Orca's support-flow branch because Ares lacks support-generation paths today.

## Acceptance Criteria

- Red tests first demonstrate that `set_other_flow_ratios` is not consumed: omitted or false gate values still allow wall/sparse ratios to change extrusion before the fix.
- After implementation, option-level tests prove omitted/false gate disables first-layer, wall, and sparse multipliers; true gate enables them; invalid gate values are rejected; and non-gated ratios remain effective.
- Pipeline-level tests prove generated G-code first-layer perimeter, wall, and sparse infill E deltas are unchanged by configured gated ratios when the gate is omitted/false and scale when the gate is true.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and Rust file LOC checks pass.
