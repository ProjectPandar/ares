# Z Hop For Layer-Change Retraction Design

## Goal

Consume the existing OrcaSlicer `z_hop`, `retract_lift_above`, and `retract_lift_below` options in concrete Ares layer-change retraction G-code by lifting Z after a layer-change retract and restoring the layer Z before the next print move.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122-5159`: defines `z_hop` as non-negative millimeters with default `0.4`, `z_hop_types` with default `Slope Lift`, `travel_slope`, and the `retract_lift_above` / `retract_lift_below` Z gates.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5622-5628`: `GCode::change_layer` calls `retract(...)` when `retract_when_changing_layer` is active and the writer will move Z.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7606-7634`: `GCode::retract` only lifts after a real or firmware retraction and delegates Z lift to `GCodeWriter::lazy_lift` / `eager_lift`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-644`: `GCodeWriter::lazy_lift` reads `z_hop`, `retract_lift_above`, and `retract_lift_below`; when the current Z is inside the configured gates and `z_hop > 0`, it schedules or emits a lift.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1084-1090`: `GCodeWriter::unlift` restores the nominal layer Z after a lift.

## Ares Destination Boundary

- Extend `crates/ares-core/src/options/layer_change_retraction.rs` so `LayerChangeRetraction` records a resolved layer-change Z-hop distance and Z-gate bounds.
- Update `crates/ares-core/src/gcode.rs` only at the existing layer-change retract, layer Z travel, and pending-unretract emission points.
- Add focused runtime tests under `crates/ares-core/src/tests/layer_change_retraction_gcode/z_hop.rs` and register that submodule from `crates/ares-core/src/tests/layer_change_retraction_gcode.rs` to keep the main test file below 400 LOC.

## Included Behavior

- `z_hop` defaults to Orca's `0.4` mm. Existing tests that need old no-hop byte output will set `z_hop = 0`.
- When layer-change retraction is enabled, `z_hop > 0`, and the current writer Z is inside the `retract_lift_above` / `retract_lift_below` gates, Ares emits:
  - the existing retract command before layer Z travel;
  - the layer Z travel to the next nominal layer Z;
  - a normal Z-hop lift to `layer_z + z_hop`;
  - before the first print move, a restore move back to the nominal layer Z, then the existing unretract command.
- Z-hop applies to both E-axis retraction and firmware retraction, because Orca's `needs_lift` accepts either positive retraction length or firmware retraction.
- `z_hop = 0` preserves the previous layer-change retraction output with no extra Z lift / restore commands.
- `retract_lift_above` defaults to `0`; `retract_lift_below` defaults to `0`, which means no upper bound. If the current pre-change Z is below `retract_lift_above`, or above a positive `retract_lift_below`, no Z-hop is emitted.
- Invalid `z_hop`, `retract_lift_above`, or `retract_lift_below` values return `SliceError::InvalidInput` naming the offending key.
- The implementation uses ordinary vertical Z moves for this slice. That corresponds to the safe normal-lift subset of Orca's writer behavior and keeps Ares output deterministic with the existing writer.

## Deferred Behavior

- `z_hop_types`, `travel_slope`, slope lift, spiral lift, arc fitting, and auto overhang-dependent lift selection are deferred.
- Ordinary travel retraction, minimum travel retraction triggers, wipe, wipe distance, toolchange retraction, multi-extruder/filament-specific Z-hop selection, `retract_lift_enforce`, top/bottom-surface lift enforcement, seam/scarf behavior, spiral vase special handling, and full Orca `GCode::retract` orchestration are deferred.
- Exact Orca lazy-lift fusion into the next XY/XYZ travel is deferred because Ares' current layer-change path has no separate XY travel between layer Z travel and the first print move.

## Docs Impact

Add one roadmap entry describing the `z_hop` layer-change runtime slice, its source boundary, included behavior, and deferred adjacent Orca behavior.

## Acceptance Criteria

- A focused RED run of `cargo nextest run -p ares-core layer_change_retraction_gcode::z_hop` fails before implementation because no Z-hop lift / restore commands are emitted.
- After implementation, the focused command passes.
- Related retraction tests pass with `cargo nextest run -p ares-core layer_change_retraction_gcode`.
- Firmware retraction with `z_hop > 0` emits `G10`/`G11` plus Z lift / restore, and does not emit E-axis retract/unretract commands.
- Default no-option layer-change retraction now includes Orca's default `z_hop = 0.4`; tests that assert old no-hop behavior explicitly set `z_hop = 0`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC check.
- No new crates or dependencies are added.
- Touched Rust files remain at or below 400 LOC.
