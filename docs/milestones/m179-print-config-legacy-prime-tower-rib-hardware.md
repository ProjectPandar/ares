# M179: PrintConfig legacy prime-tower rib and hardware migrations

## Goal
Port the prime-tower rib aliases, clearance/tool-change aliases, and `wall_direction` legacy value branch from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8046-8067` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:8046-8067` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `prime_tower_rib_wall` string value `1` is rewritten to `wipe_tower_wall_type` with string value `rib`.
- `prime_tower_rib_wall` values other than string `1`, including non-string values, are dropped.
- `prime_tower_extra_rib_length` is renamed to `wipe_tower_extra_rib_length` while preserving the value.
- `prime_tower_rib_width` is renamed to `wipe_tower_rib_width` while preserving the value.
- `prime_tower_fillet_wall` is renamed to `wipe_tower_fillet_wall` while preserving the value.
- `extruder_clearance_max_radius` is renamed to `extruder_clearance_radius` while preserving the value.
- `machine_switch_extruder_time` is renamed to `machine_tool_change_time` while preserving the value.
- `wall_direction` string value `auto` is rewritten to `ccw`; other strings and non-string values remain unchanged.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:8069+`, including the obsolete-key ignore set and final unknown-key validation, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
