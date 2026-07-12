# M107: PrintConfig scarf seam registry

## Goal
Port the adjacent scarf-seam option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5392-5500` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:216-220`, `PrintConfig.hpp:1224-1234`, `PrintConfig.cpp:360-365`, `PrintConfig.cpp:5392-5500`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, scarf seam planning, seam slope geometry, overhang estimation, scarf flow/speed planning, inner-wall scarf behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `seam_slope_type`, `seam_slope_conditional`, `scarf_angle_threshold`, `scarf_overhang_threshold`, `scarf_joint_speed`, `scarf_joint_flow_ratio`, `seam_slope_start_height`, `seam_slope_entire_loop`, `seam_slope_min_length`, `seam_slope_steps`, and `seam_slope_inner_walls` with exact kinds, defaults, and source line ranges.
- `seam_slope_type` uses the current registry enum metadata boundary with the upstream `SeamScarfType` enum-map citation and default `none`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for scarf seam planning, conditional scarf selection, overhang estimation, scarf speed/flow application, seam slope geometry, inner-wall scarf behavior, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following wipe-speed options from `PrintConfig.cpp:5502+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
