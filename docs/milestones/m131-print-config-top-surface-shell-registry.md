# M131: PrintConfig top-surface and top-shell registry

## Goal
Port the adjacent top-surface line-width/speed and top-shell layers/thickness option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6543-6584` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1166-1169`, `PrintConfig.cpp:6543-6584`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, top-surface geometry behavior, top-shell layer adjustment behavior, line-width behavior, speed planning behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `top_shell_layers`, `top_shell_thickness`, `top_surface_line_width`, and `top_surface_speed` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for top-surface line width, top-surface speed, top-shell layer adjustment, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `top_surface_density`, `bottom_surface_density`, and following options from `PrintConfig.cpp:6586+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
