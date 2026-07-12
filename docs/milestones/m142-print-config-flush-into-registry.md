# M142: PrintConfig flush-into registry

## Goal
Port the adjacent flush-into option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6847-6870` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1003-1006`, `PrintConfig.cpp:6847-6870`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, purge-routing behavior, object/infill/support flush behavior, prime-tower behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `flush_into_infill`, `flush_into_objects`, and `flush_into_support` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for purging into object infill/support/object volumes, prime-tower dependency checks, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wipe_tower_bridging`, `wipe_tower_extra_spacing`, `wipe_tower_extra_flow`, `idle_temperature`, and following options from `PrintConfig.cpp:6872+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
