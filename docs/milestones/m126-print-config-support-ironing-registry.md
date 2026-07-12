# M126: PrintConfig support ironing registry

## Goal
Port the adjacent support interface ironing option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6446` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:997-1000`, `PrintConfig.hpp:87-98`, `PrintConfig.cpp:225-255`, `PrintConfig.cpp:6406-6446`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, support-ironing behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, and `support_ironing_spacing` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for support interface ironing, ironing pattern application, flow/spacing behavior, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `activate_chamber_temp_control`, `chamber_temperature`, and following chamber-temperature options from `PrintConfig.cpp:6448+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
