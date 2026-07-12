# M185: PrintConfig min object distance API

## Goal
Port OrcaSlicer's `min_object_distance(const ConfigBase&)` helper into `ares-core` as a small `SliceOptions` API for future UI/arrange callers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.hpp:602-603` and `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8305-8329`, with enum/context anchors at `PrintConfig.hpp:148-152` and the existing option metadata for `printer_technology`, `extruder_clearance_radius`, and `print_sequence`. No object arranging, model placement, UI runtime behavior, variant expansion, normalization, slicing behavior, extrusion behavior, G-code behavior, new crate, or dependency is added.

## Exit checklist
- `SliceOptions` exposes `min_object_distance()` aligned with Orca's helper semantics.
- SLA printer technology returns `6.0`.
- Non-SLA missing `extruder_clearance_radius` or `print_sequence` returns `0.0`, matching Orca's null option branch.
- Non-SLA `print_sequence == by object` returns `max(6.0, extruder_clearance_radius)`; other print sequences return `6.0` when both options are present.
- Invalid user-provided boundary values return `SliceError::InvalidInput` rather than panicking.
- Implementation lives outside `options.rs` so the existing near-400 LOC file remains under the module split threshold.
- `DynamicPrintConfig::normalize_fdm` and later behavior from `PrintConfig.cpp:8332+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
