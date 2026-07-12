# M99: PrintConfig loading move, start/end points, ooze, and filename registry

## Goal
Port the adjacent extra-loading, start/end point, infill-retraction, ooze-prevention, and filename-format option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4812-4848` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1432,1544-1546,1614`, `PrintConfig.cpp:4812-4848`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, MMU loading behavior, cutter/start-end point behavior, retraction suppression behavior, ooze-prevention temperature behavior, filename rendering behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `extra_loading_move`, `start_end_points`, `reduce_infill_retraction`, `ooze_prevention`, and `filename_format` with exact kinds, defaults, and source line ranges.
- `start_end_points` uses existing points registry metadata without adding geometry/runtime behavior.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The near-limit known-count fixture is split so modified Rust files remain under 400 LOC.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for MMU loading, cutter start/end points, infill retraction suppression, ooze-prevention temperature control, filename formatting, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following `make_overhang_printable` and later options from `PrintConfig.cpp:4850+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
