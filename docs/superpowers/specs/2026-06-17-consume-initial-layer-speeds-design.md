# Consume Initial Layer Speeds Design

## Goal

Port the OrcaSlicer first-layer print and travel speed selection into Ares G-code feedrates so the existing `initial_layer_speed`, `initial_layer_infill_speed`, and `initial_layer_travel_speed` options affect concrete first-layer moves.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1421` declares `ConfigOptionFloatOrPercent initial_layer_travel_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1529` declares `ConfigOptionFloat initial_layer_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1532` declares `ConfigOptionFloat initial_layer_infill_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3280-3286` registers `initial_layer_speed` as a float in mm/s, min `1`, default `30`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3288-3294` registers `initial_layer_infill_speed` as a float in mm/s, min `1`, default `60`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3296-3304` registers `initial_layer_travel_speed` as `coFloatOrPercent`, ratio-over `travel_speed`, min `1`, default `100%`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6495-6500` uses `initial_layer_speed` for first-layer perimeter paths and `initial_layer_infill_speed` for other first-layer print paths.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:613`, `698-699`, and `835-838` use `initial_layer_travel_speed` for first-layer travel.

## Ares Boundary

Implement the runtime slice in `crates/ares-core` only:

- `crates/ares-core/src/options.rs` parses `initial_layer_speed` and `initial_layer_infill_speed` as positive number-or-string mm/s values, defaulting to Orca defaults `30` and `60`.
- `crates/ares-core/src/options.rs` parses `initial_layer_travel_speed` as a positive numeric-or-percent value over the resolved `travel_speed`, defaulting to `100%` of travel speed.
- `crates/ares-core/src/speeds.rs` stores the three first-layer speeds in `SpeedOptions`.
- `generate_speed_moves` uses layer id `0` as first-layer context and assigns first-layer speeds to emitted first-layer speed moves.
- First-layer travel moves use `initial_layer_travel_speed`.
- `crates/ares-core/src/gcode.rs` uses `initial_layer_travel_speed` for the first emitted layer-change Z travel to mirror Orca's first-layer Z travel handling; later layer-change Z travels keep using `travel_speed`.
- First-layer `ExternalPerimeter` and `InternalPerimeter` moves use `initial_layer_speed`.
- First-layer `SparseInfill` moves use `initial_layer_infill_speed`.
- First-layer `Brim` uses `initial_layer_speed`, matching its existing Ares mapping to external perimeter speed.
- First-layer `Skirt`, `Bridge`, and `InternalBridge` keep their existing dedicated speed mappings because Ares already exposes role-specific speed options for those roles and the previous bridge/skirt slices intentionally consume them.

This slice may add a layer-aware `speed_for_layer(kind, role, is_first_layer)` helper while keeping `speed_for_role(kind, role)` as the non-first-layer compatibility path for existing tests and callers.

## Required Local Structure

- `crates/ares-core/src/speeds.rs` is already close to the 400 LOC limit. Move the existing inline speed unit tests into a new sibling file `crates/ares-core/src/speeds/tests.rs` before adding first-layer speed tests so both files remain below the repository LOC gate.
- `crates/ares-core/src/options/tests.rs` is exactly 400 LOC. Register the new focused option test module by editing the existing final `option_test_modules!(...)` macro rather than adding a new standalone module line.

## Out Of Scope

- No new option registration or registry metadata.
- No `slow_down_layers` interpolation.
- No raft or object-layer-over-raft speed behavior.
- No acceleration, jerk, volumetric speed, support, solid infill, top surface, gap fill, ironing, or overhang speed behavior.
- No geometry/path ordering changes.
- No Ares-owned pipeline redesign.

## Acceptance Criteria

- A numeric `initial_layer_speed` changes first-layer external and internal perimeter G-code feedrates.
- The same setting does not change second-layer perimeter feedrates.
- A numeric `initial_layer_infill_speed` changes first-layer sparse infill feedrate without changing perimeter feedrate.
- `initial_layer_travel_speed` accepts both numeric mm/s and percent strings over `travel_speed`; both forms change first-layer travel feedrate.
- `initial_layer_travel_speed` changes the first layer-change Z travel feedrate and first-layer path travel feedrates.
- `initial_layer_travel_speed` does not change second-layer layer-change Z travel or second-layer path travel feedrates.
- Defaults match Orca's registered defaults: first-layer perimeter speed `30`, first-layer sparse infill speed `60`, and first-layer travel speed equal to `travel_speed`.
- Invalid first-layer speeds below or equal to zero, non-numeric strings, non-finite values, and non-number JSON values are rejected through `SliceOptions::speed_options`.
- Existing skirt, bridge, and internal bridge speed behavior remains unchanged.
- All touched Rust source files remain at or below 400 LOC.
- Verification must include focused red/green tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate.
