# M144: PrintConfig XY compensation and polyhole registry

## Goal
Port the adjacent XY compensation and polyhole option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6907-6954` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1001-1002`, `PrintConfig.hpp:1202-1204`, `PrintConfig.cpp:6907-6954`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, XY compensation behavior, polyhole conversion behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `hole_to_polyhole`, `hole_to_polyhole_threshold`, `hole_to_polyhole_twisted`, `xy_contour_compensation`, and `xy_hole_compensation` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for XY hole/contour compensation, polyhole detection/conversion/twist, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `thumbnails`, `thumbnails_format`, and following options from `PrintConfig.cpp:6956+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
