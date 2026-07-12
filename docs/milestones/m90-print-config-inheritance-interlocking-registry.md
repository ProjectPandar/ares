# M90: PrintConfig inheritance, MMU interlocking, and calibration flag option registry

## Goal
Port the adjacent FFF profile inheritance, interface-shell, MMU segmented-region, interlocking, and flowrate-calibration flag option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4063-4159` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:935,937-938,1062-1067,1070`, `PrintConfig.cpp:4063-4159`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, inheritance resolution, MMU interlocking behavior, calibration behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `inherits`, `inherits_group`, `interface_shells`, `mmu_segmented_region_max_width`, `mmu_segmented_region_interlocking_depth`, `interlocking_beam`, `interlocking_beam_width`, `interlocking_orientation`, `interlocking_beam_layer_count`, `interlocking_depth`, `interlocking_boundary_avoidance`, and `calib_flowrate_topinfill_special_order` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/category/sidetext/min/max/mode/full-width/height/cli metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for inheritance resolution, interface shells, segmented-region interlocking, beam interlocking, calibration ordering, slicing, extrusion, and downstream G-code behavior remains deferred.
- `ironing_type` and following options from `PrintConfig.cpp:4161+` remain unchanged/deferred.
- `middle` remains below the 400 LOC threshold by splitting its interlocking/internal-tail registry definitions into a focused `middle_tail` shard without changing registry behavior.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
