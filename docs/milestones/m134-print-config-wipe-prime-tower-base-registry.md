# M134: PrintConfig wipe and prime-tower base registry

## Goal
Port the adjacent wipe and prime-tower base option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6628-6657` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1569`, `PrintConfig.hpp:1573-1575`, `PrintConfig.cpp:6628-6657`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe movement behavior, prime tower generation behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `enable_prime_tower`, `prime_tower_enable_framework`, `wipe`, and `wipe_distance` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- A mechanical registry-table shard split is allowed only to keep modified Rust files below 400 LOC; moved definitions must remain unchanged.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for wipe movement, wipe distance, prime tower enablement/framework, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `flush_volumes_vector`, `flush_volumes_matrix`, `flush_multiplier`, `prime_volume`, and following options from `PrintConfig.cpp:6659+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
