# M122: PrintConfig support pattern spacing, speed, expansion, and style registry

## Goal
Port the adjacent support base-pattern spacing, normal support expansion, support speed, and support style option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6178-6230` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:972-975`, `PrintConfig.hpp:179-181`, `PrintConfig.cpp:322-331`, `PrintConfig.cpp:6178-6230`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, support spacing behavior, support expansion behavior, support speed behavior, support style selection behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_base_pattern_spacing`, `support_expansion`, `support_speed`, and `support_style` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support spacing, expansion, speed assignment, support style selection, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `independent_support_layer_height` and following support options from `PrintConfig.cpp:6232+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
