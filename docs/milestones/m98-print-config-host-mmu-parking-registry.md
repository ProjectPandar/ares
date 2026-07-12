# M98: PrintConfig notes, host, nozzle-volume, and MMU parking registry

## Goal
Port the adjacent notes, printer-host type, nozzle-volume, cooling-tube, high-current filament swap, and parking-position option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4723-4810` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:79-81,1428-1431,1613,1633`, `PrintConfig.cpp:137-153,4723-4810`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, printer-host upload behavior, MMU loading/unloading behavior, G-code behavior, UI behavior, slicing behavior, or extrusion behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `notes`, `host_type`, `nozzle_volume`, `cooling_tube_retraction`, `cooling_tube_length`, `high_current_on_filament_swap`, and `parking_pos_retraction` with exact kinds, defaults, and source line ranges.
- `host_type` cites both the `PrintHostType` enum map and the option-definition lines while preserving metadata-only enum behavior.
- `nozzle_volume` uses existing nullable float-vector registry metadata without adding typed parsing/accessors.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for printer-host upload, MMU cooling tube moves, high-current filament swap, parking-position use, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following `extra_loading_move`, `start_end_points`, and later options from `PrintConfig.cpp:4812+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
