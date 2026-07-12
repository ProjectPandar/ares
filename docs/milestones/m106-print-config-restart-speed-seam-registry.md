# M106: PrintConfig restart, retraction speed, M73, and seam registry

## Goal
Port the adjacent restart-extra, retraction/deretraction speed, firmware retraction, Bambu calibration mark, M73 disable, seam-position, staggered-inner-seam, and seam-gap option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5306-5390` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:944-945`, `PrintConfig.hpp:1182`, `PrintConfig.hpp:1296`, `PrintConfig.hpp:1382-1384`, `PrintConfig.hpp:1417`, `PrintConfig.hpp:1424-1425`, `PrintConfig.cpp:5306-5390`, `PrintConfig.hpp:211-213`, `PrintConfig.cpp:350-357`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, restart/retraction planning, firmware-retraction G10/G11 generation, M73 generation control, calibration mark generation, seam planning, scarf seam behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `retract_restart_extra`, `retract_restart_extra_toolchange`, `retraction_speed`, `deretraction_speed`, `use_firmware_retraction`, `bbl_calib_mark_logo`, `disable_m73`, `seam_position`, `staggered_inner_seams`, and `seam_gap` with exact kinds, defaults, and source line ranges.
- `seam_position` uses the current registry enum metadata boundary with the upstream `SeamPosition` enum-map citation and default `aligned`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for restart compensation, retraction/deretraction motion speed planning, firmware retraction G-code, M73 suppression, calibration marks, seam planning, staggered seams, seam-gap geometry, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following scarf seam options from `PrintConfig.cpp:5392+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
