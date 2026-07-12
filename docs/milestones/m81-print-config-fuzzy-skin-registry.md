# M81: PrintConfig fuzzy-skin option registry

## Goal
Port the adjacent FFF fuzzy-skin option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3420-3576` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:50-72,1108-1119`, `PrintConfig.cpp:192-210,218-223,3420-3576`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, fuzzy-skin runtime behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `fuzzy_skin`, `fuzzy_skin_thickness`, `fuzzy_skin_point_distance`, `fuzzy_skin_first_layer`, `fuzzy_skin_mode`, `fuzzy_skin_noise_type`, `fuzzy_skin_scale`, `fuzzy_skin_octaves`, `fuzzy_skin_persistence`, `fuzzy_skin_ripples_per_layer`, `fuzzy_skin_ripple_offset`, and `fuzzy_skin_layers_between_ripple_offset` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream enum labels, UI label/category/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary, while enum key/default string identity is source-cited.
- Fuzzy-skin geometry generation, random/noise displacement, first-layer filtering, ripple behavior, validation, typed accessors, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filter_out_gap_fill` and following options from `PrintConfig.cpp:3578+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
