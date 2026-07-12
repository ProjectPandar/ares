# M145: PrintConfig thumbnails registry

## Goal
Port the adjacent G-code thumbnail option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6956-6978` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:397-399`, `PrintConfig.hpp:1616`, `PrintConfig.cpp:542-549`, `PrintConfig.cpp:6956-6978`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, thumbnail generation/validation behavior, G-code embedding behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `thumbnails` and `thumbnails_format` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for thumbnail string validation, thumbnail image generation, thumbnail format handling, G-code thumbnail embedding, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `use_relative_e_distances`, `wall_generator`, and following options from `PrintConfig.cpp:6980+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
