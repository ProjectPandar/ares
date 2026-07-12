# M165: PrintConfig SLA pad base registry

## Goal
Port the first SLA pad/base-pool settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7712-7766` into `ares-core` registry metadata, including a mechanical key-list shard split required to keep Rust test files below 400 LOC.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1733-1755`, `PrintConfig.cpp:7712-7766`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA pad generation behavior, pad/base geometry behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `pad_enable`, `pad_wall_thickness`, `pad_wall_height`, `pad_brim_size`, `pad_max_merge_distance`, and `pad_wall_slope` with exact kinds, defaults, and source line ranges.
- `crates/ares-core/src/options/registry/tests/keys/second.rs` remains below 400 LOC by mechanically moving its `parking_pos_retraction` and following tail keys into `keys/third.rs`, before the existing `wall_*`/`wipe_*`/`z*` entries.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA pad generation, pad/base geometry, zero-elevation pad mode, hollowing, material print speed, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `pad_around_object`, zero-elevation pad settings, hollowing settings, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7768+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
