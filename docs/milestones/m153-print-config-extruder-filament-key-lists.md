# M153: PrintConfig extruder/filament option key lists

## Goal
Port the `PrintConfigDef` extruder and filament option-key list accessors from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7227` and `PrintConfig.hpp:569-593` into `ares-core` as read-only, source-cited option registry API data.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:569-593`, `PrintConfig.cpp:7164-7227`, and the current `ares-core` option registry API boundary. No new Ares pipeline, crate, dependency, option parsing behavior, extruder/filament override resolution behavior, retraction behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `ares-core` exposes read-only key-list functions for Orca's `extruder_option_keys`, `extruder_retract_keys`, `filament_option_keys`, and `filament_retract_keys`.
- Each exposed list matches the upstream order exactly; retract key lists remain sorted as upstream asserts.
- Every key in each exposed list is already covered by `option_definition()` metadata.
- Public API coverage exists for all four lists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for extruder count expansion, filament override following, retraction planning, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `PrintConfigDef::init_sla_params` from `PrintConfig.cpp:7229+` remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
