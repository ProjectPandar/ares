# M95: PrintConfig resonance avoidance and input shaping option registry

## Goal
Port the adjacent resonance avoidance and input shaping option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4516-4589` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:365-379,544,1276-1287`, `PrintConfig.cpp:503-518,4516-4589`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, resonance speed behavior, input-shaping G-code behavior, firmware override behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `resonance_avoidance`, `min_resonance_avoidance_speed`, `max_resonance_avoidance_speed`, `input_shaping_emit`, `input_shaping_type`, `input_shaping_freq_x`, `input_shaping_freq_y`, `input_shaping_damp_x`, and `input_shaping_damp_y` with exact kinds, defaults, and source line ranges.
- `input_shaping_type` source cites the `InputShaperType` enum declaration and static enum map while staying metadata-only.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI full-label/tooltip/category/sidetext/min/max/mode/readonly/enum-label metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for resonance avoidance, input-shaping emission, firmware override commands, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following fan/layer-height/extrusion-rate/nozzle options from `PrintConfig.cpp:4591+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
