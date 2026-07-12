# M92: PrintConfig custom G-code, machine limit flag, and small-area flow option registry

## Goal
Port the adjacent FFF custom G-code, machine-limit emission flag, small-area infill flow compensation, and scarf-seam marker option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4295-4375` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1211,1247,1358-1360,1398-1400,1464,1466`, `PrintConfig.cpp:4295-4375`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, custom G-code emission behavior, machine-limit emission behavior, small-area flow compensation behavior, scarf-seam behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `layer_change_gcode`, `time_lapse_gcode`, `wrapping_detection_gcode`, `silent_mode`, `emit_machine_limits_to_gcode`, `machine_pause_gcode`, `template_custom_gcode`, `small_area_infill_flow_compensation`, `small_area_infill_flow_compensation_model`, and `has_scarf_joint_seam` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- `tail` remains below the 400 LOC threshold by splitting later sorted registry definitions into a focused `tail_final` shard without changing registry behavior.
- Upstream UI label/tooltip/category/sidetext/min/max/mode/gui-flags/multiline/full-width/height metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for custom G-code insertion, machine limit emission, small-area flow compensation, scarf-seam detection, slicing, extrusion, and downstream G-code behavior remains deferred.
- Machine axis limit loop options from `PrintConfig.cpp:4377+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
