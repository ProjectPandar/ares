# M80: PrintConfig filament ironing override registry

## Goal
Port the adjacent FFF filament-specific ironing override option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3372-3418` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1148-1151`, `PrintConfig.cpp:3372-3418`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, ironing runtime behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OptionValueKind` includes metadata-only `PercentsNullable` for Orca `coPercents` plus `nullable = true` registry metadata.
- `OPTION_DEFINITIONS` includes `filament_ironing_flow`, `filament_ironing_spacing`, `filament_ironing_inset`, and `filament_ironing_speed` with exact kinds, nil defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary, except preserving nullable type identity through `PercentsNullable`/`FloatsNullable` kinds.
- Ironing runtime behavior, filament override resolution, typed accessors, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `fuzzy_skin` and following options from `PrintConfig.cpp:3420+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
