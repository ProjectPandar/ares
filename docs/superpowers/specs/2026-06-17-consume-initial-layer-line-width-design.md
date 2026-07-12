# Consume Initial Layer Line Width Design

## Goal

Port the OrcaSlicer `initial_layer_line_width` flow-width behavior into Ares slicing output so an already-registered option changes first-layer extrusion amounts instead of remaining registry-only metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1527` declares `ConfigOptionFloatOrPercent initial_layer_line_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3251-3261` registers `initial_layer_line_width` as `coFloatOrPercent`, ratio-over `nozzle_diameter`, minimum `0`, maximum `1000`, literal maximum `10`, default `0`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:42-44` maps `initial_layer_line_width` to `frPerimeter`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:62-105` resolves configured flow width: a zero value falls back through `line_width`, percent values are resolved over nozzle diameter, and automatic width remains role/nozzle based.
- `OrcaSlicer/src/libslic3r/PrintRegion.cpp:27-49` uses `print_config.initial_layer_line_width` for first-layer region flow when its value is greater than zero, then calls `Flow::new_from_config_width`.
- `OrcaSlicer/src/libslic3r/Print.cpp:1960-1973` and `1981-1992` apply the same first-layer width selection to skirt and brim flow generation.

## Ares Boundary

Implement the runtime slice in `crates/ares-core` only:

- `crates/ares-core/src/options.rs` parses `initial_layer_line_width` with the existing extrusion-width parser, including percent-over-nozzle handling and non-negative validation.
- `crates/ares-core/src/extrusions.rs` stores `initial_layer_line_width` in `ExtrusionOptions`.
- `ExtrusionOptions::extrusion_per_mm_for_layer(role, layer_height, is_first_layer)` resolves first-layer extrusion width from `initial_layer_line_width` when `is_first_layer` and the parsed value is greater than zero.
- A zero or omitted `initial_layer_line_width` falls back to the existing role width path, including `line_width` and automatic nozzle-derived width.
- The behavior applies to Ares roles that currently become real first-layer extrusion moves: `Skirt`, `Brim`, `ExternalPerimeter`, `InternalPerimeter`, and `SparseInfill`.

This slice does not change Ares perimeter/skirt/brim geometry spacing. Orca also uses the first-layer width for brim/skirt flow construction, but Ares currently separates generated path geometry width from extrusion E/mm and has no layer-aware width API for geometry generation. The required concrete output for this slice is first-layer G-code extrusion amount changes.

## Out Of Scope

- No new option registration or registry metadata.
- No support, raft, wipe tower, or bridge-specific first-layer width behavior beyond any existing Ares path roles that already emit first-layer moves.
- No changes to path geometry offsets, skirt distance, brim path count, or perimeter spacing.
- No Ares-owned pipeline redesign.

## Acceptance Criteria

- A parsed numeric `initial_layer_line_width` changes first-layer perimeter G-code extrusion delta compared with the same model and options without that setting.
- The same setting does not change second-layer perimeter G-code extrusion delta.
- A string percent such as `"150%"` resolves against the nozzle diameter and changes first-layer E/mm accordingly.
- `initial_layer_line_width: 0` and omitted `initial_layer_line_width` produce identical extrusion width behavior.
- Invalid values below zero, non-numeric strings, NaN, and non-number JSON values are rejected through `SliceOptions::extrusion_options`.
- Existing flow-ratio behavior remains multiplicative: first-layer line width changes the base extrusion area and existing flow ratios still multiply that result.
- All touched Rust source files remain at or below 400 LOC.
- Verification must include focused red/green tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate.
