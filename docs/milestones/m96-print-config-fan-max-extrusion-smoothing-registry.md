# M96: PrintConfig fan max and extrusion-rate smoothing option registry

## Goal
Port the adjacent fan maximum, maximum layer height citation, and extrusion-rate smoothing option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4591-4648` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1362-1364,1535-1536`, `PrintConfig.cpp:4591-4648`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, cooling behavior, extrusion-rate smoothing behavior, arc-fitting behavior, speed planning behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `fan_max_speed`, `max_volumetric_extrusion_rate_slope`, `max_volumetric_extrusion_rate_slope_segment_length`, and `extrusion_rate_smoothing_external_perimeter_only` with exact kinds, defaults, and source line ranges.
- Existing `max_layer_height` source metadata includes `PrintConfig.hpp:1536` while preserving its existing kind/default and typed behavior.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI full-label/tooltip/category/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for part cooling, extrusion-rate smoothing, arc-fitting interactions, speed planning, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following fan-min/additional-cooling/min-layer/nozzle options from `PrintConfig.cpp:4651+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
