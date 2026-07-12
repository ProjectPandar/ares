# M154: PrintConfig SLA display and tilt registry

## Goal
Port the first SLA printer settings slice from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7235-7310` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:260-263`, `PrintConfig.hpp:1830-1836`, `PrintConfig.hpp:1845-1847`, `PrintConfig.cpp:400-404`, `PrintConfig.cpp:7235-7310`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA slicing behavior, display pixel/orientation behavior, tilt timing behavior, area fill behavior, UI behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `display_width`, `display_height`, `display_pixels_x`, `display_pixels_y`, `display_mirror_x`, `display_mirror_y`, `display_orientation`, `fast_tilt_time`, `slow_tilt_time`, and `area_fill` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA display orientation, mirroring, tilt timing, area fill, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `relative_correction` and later SLA settings from `PrintConfig.cpp:7312+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
