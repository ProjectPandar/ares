# M105: PrintConfig extruder variant and ID registry

## Goal
Port the adjacent extruder variant list, AMS count, printer/print/filament extruder IDs, and printer/print/filament extruder variant option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239-5304` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1077-1078`, `PrintConfig.hpp:1338`, `PrintConfig.hpp:1410-1413`, `PrintConfig.cpp:5239-5304`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, AMS-count parser, extruder variant normalization, extruder mapping behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `extruder_variant_list`, `extruder_ams_count`, `printer_extruder_id`, `printer_extruder_variant`, `master_extruder_id`, `print_extruder_id`, `print_extruder_variant`, `filament_extruder_variant`, and `filament_self_index` with exact kinds, defaults, and source line ranges.
- The commented-out upstream `filament_extruder_id` block remains deferred and is not added as a registry definition.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for extruder variant normalization, AMS-count parsing, printer/print/filament extruder mapping, preset compatibility, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following restart/retraction speed options from `PrintConfig.cpp:5306+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
