# M169: PrintConfig legacy key aliases

## Goal
Port the first simple legacy-option normalization slice from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7867-7899` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7867-7899` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization rewrites `curr_bed_type` value `SuperTack Plate` to `Supertack Plate`.
- `SliceOptions` deserialization rewrites the covered legacy keys to their modern Orca keys: `enable_wipe_tower`, `wipe_tower_width`, `wiping_volume`, `wipe_tower_brim_width`, `tool_change_gcode`, `bridge_fan_speed`, `infill_extruder`, `solid_infill_extruder`, `perimeter_extruder`, `wipe_tower_extruder`, `support_material_extruder`, `support_material_interface_extruder`, `support_material_angle`, and `support_material_enforce_layers`.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:7900+` remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
