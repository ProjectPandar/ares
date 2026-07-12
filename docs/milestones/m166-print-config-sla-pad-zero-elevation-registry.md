# M166: PrintConfig SLA zero-elevation pad registry

## Goal
Port the SLA zero-elevation object-pad settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7768-7817` into `ares-core` registry metadata, including mechanical registry/key shard splits required to keep Rust files below 400 LOC.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1757-1780`, `PrintConfig.cpp:7768-7817`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, zero-elevation pad behavior, object-pad connector geometry, SLA pad generation behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `pad_around_object`, `pad_around_object_everywhere`, `pad_object_gap`, `pad_object_connector_stride`, `pad_object_connector_width`, and `pad_object_connector_penetration` with exact kinds, defaults, and source line ranges.
- `late_tail_after_material.rs` remains below 400 LOC by mechanically moving `parking_pos_retraction` and following definitions into a new sorted registry shard.
- Expected-key shards remain below 400 LOC and preserve sorted concatenated order.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Runtime behavior for zero-elevation pad mode, object-pad connector geometry, SLA pad generation, hollowing, material print speed, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `hollowing_enable`, hollowing settings, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7819+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
