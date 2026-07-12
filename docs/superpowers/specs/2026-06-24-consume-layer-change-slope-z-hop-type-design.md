# Consume Layer-Change Slope Z-Hop Type Design

## Scope

Consume the already-parsed OrcaSlicer `z_hop_types`, `filament_z_hop_types`, and `travel_slope` values in Ares' layer-change retraction Z-hop path. The previous slice made these options affect ordinary travel retraction; this slice applies the same source boundary to layer-change retraction so the default Orca `Slope Lift` produces concrete layer-change slope-lift travel G-code instead of always emitting a separate vertical `lift Z` line.

This is a source-cited Rust rewrite slice of Orca `libslic3r`, not a new Ares-owned layer-change pipeline.

## Upstream Boundary

The upstream OrcaSlicer checkout is vendored under this repository at `OrcaSlicer/`, so the source paths below are directly inspectable from `/home/indexyz/Ares`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:246-250` defines `LiftType` including `NormalLift`, `SpiralLift`, and `SlopeLift`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1377-1378` owns `z_hop_types` and `travel_slope` on `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5149-5171` defines `z_hop_types` default `Slope Lift` and `travel_slope` default `3` degrees with range `1..=90`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7170-7224` and `PrintConfig.cpp:8188-8201` keep `z_hop_types` / `travel_slope` in the filament/printer variant override path.
- `OrcaSlicer/src/libslic3r/Extruder.cpp:215-218` converts `travel_slope` degrees to radians.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5617-5629` applies `z_hop_types` during `GCode::change_layer(...)` before layer Z movement: when layer-change retraction is enabled and the writer will move Z, Orca converts the active `z_hop_types` value to a lift type and calls `retract(...)`; the auto case is force-mapped to spiral for upstream layer changes.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7443-7455` maps `ZHopType` to `LiftType`: normal maps to normal, slope maps to slope, spiral maps to spiral, and other values fall back to normal.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-648` schedules lazy lift by storing the target lift amount and lift type when lift gates allow Z-hop.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:719-747` consumes scheduled `SlopeLift` during the following XY/XYZ travel by emitting an early raised XYZ slope-top move when the XY distance is long enough, or a raised target move otherwise; normal lift emits a vertical Z move in that travel consumption path.

## Rust Destination Boundary

- Layer-change lift scheduling belongs in `crates/ares-core/src/gcode_layer_change_retraction.rs`, because `crates/ares-core/src/gcode.rs` is at the 400 LOC guard and should not grow.
- Reuse the existing parsed `LayerChangeRetraction::z_hop_lift` tuple from `crates/ares-core/src/options/layer_change_retraction.rs`; do not add more option metadata.
- Reuse the existing `TravelLiftMove` / `GCodeWriter::travel_to_xyz_with_comment(...)` travel emission path from the ordinary travel slope slice where possible.
- Focused tests belong in a new `crates/ares-core/src/tests/layer_change_retraction_gcode/z_hop_type.rs` module registered from `layer_change_retraction_gcode.rs`.
- Existing layer-change tests that assert the old vertical lift shape should explicitly set `z_hop_types = ["Normal Lift"]`.
- `docs/roadmap.md` records the completed runtime slice only after independent implementation review approval.

## Included Behavior

1. Preserve explicit `Normal Lift` layer-change behavior: after layer-change retract and layer-Z move, emit the vertical `lift Z`; before the first print move, restore layer Z and unretract.
2. For default or explicit `Slope Lift`, do not emit the separate post-layer-Z vertical `lift Z` line. Instead, schedule the layer-change lift for the next same-layer travel move.
3. When the next travel move has enough XY distance for the configured `travel_slope`, emit an early raised XYZ slope-top travel move from the current XY position toward that travel target, followed by the remaining XY travel at raised Z. Restore to the layer Z before unretracting and printing.
4. When the next travel move is too short for the configured slope, emit one raised XYZ travel to the travel target, then restore to the layer Z before unretracting and printing.
5. When layer-change retraction is followed by a print move before any same-layer travel consumes the scheduled slope lift, emit the existing vertical `lift Z` line immediately before the restore/unretract sequence. This fallback preserves the already-tested layer-change no-travel shape and avoids starting a print while a slope lift is pending.
6. Use the existing effective first single-active-filament `z_hop_types`, nullable `filament_z_hop_types` override, `travel_slope`, `z_hop`, `filament_z_hop`, lift-above/below gates, and lift-enforce gates.
7. Accept `Auto Lift` and `Spiral Lift` values but keep this slice on explicit normal-lift fallback for layer-change output. This deliberately follows Ares' current ordinary travel fallback instead of implementing Orca's change-layer auto-to-spiral branch without spiral-arc support.
8. Preserve firmware retraction, E-axis retraction, restart-extra, deretraction speed, absolute/relative E state, zero `z_hop`, zero retraction length, disabled layer-change retraction, and current first-layer/non-top/top/bottom lift-enforce behavior.

## Deferred Behavior

- Orca's change-layer `Auto Lift` forcing `SpiralLift`.
- `Spiral Lift` arc output, bed-area spiral safety checks, and unknown-position fallback details.
- Eager lift and timelapse/custom-injection lift behavior.
- Toolchange, nozzle-change, cut, wipe-tower, seam/scarf, avoid-crossing-perimeters, support/internal exceptions, spiral-vase-specific layer-change behavior, and full `GCode::retract` orchestration.
- Multi-extruder/current-filament selection beyond Ares' current first-value single-active-filament path.

## Acceptance Criteria

- RED: after adding focused layer-change lift-type tests, `cargo nextest run -p ares-core layer_change_retraction_gcode::z_hop_type` fails because layer-change Z-hop ignores `z_hop_types` and still emits vertical `lift Z`.
- GREEN: after implementation, `cargo nextest run -p ares-core layer_change_retraction_gcode::z_hop_type` passes.
- A focused layer-change test proves explicit `Normal Lift` still emits the vertical `lift Z` after the layer-Z move and restores before unretract.
- A focused layer-change test proves default `Slope Lift` emits a raised XYZ slope-top travel plus remaining XY travel before restore/unretract, with no separate `lift Z` line.
- A focused layer-change test proves too-short slope travel emits one raised XYZ travel to the first layer travel target and no separate `lift Z` line.
- A focused layer-change test proves a pending layer-change slope lift falls back to the vertical `lift Z` line when the first same-layer move is a print move, so restore/unretract does not run while a lift is still pending.
- A focused layer-change test proves `filament_z_hop_types = Normal Lift` overrides the default slope behavior.
- Existing layer-change retraction tests continue to pass after their vertical-lift expectations are made explicit with `z_hop_types = ["Normal Lift"]`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
