# Layer Change Retraction Design

## Scope

Consume OrcaSlicer layer-change retraction options into concrete Ares G-code output. This is a narrow `libslic3r` rewrite slice for single-extruder layer transitions, not a new Ares retraction subsystem.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5062-5074` defines `retract_when_changing_layer` as `ConfigOptionBools` default `false` and `retraction_length` as `ConfigOptionFloats` default `0.8`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5322-5337` defines `retraction_speed` default `30` mm/s and `deretraction_speed` default `0`, where `0` falls back to the retraction speed in Orca's filament runtime.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5625-5628` retracts during layer change when `retract_when_changing_layer` is enabled and the writer will move Z.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1004-1078` emits retract and unretract extrusion commands using configured lengths and speeds.

## Ares Destination Boundary

- Add a small runtime options boundary in `crates/ares-core/src/options/` that parses the first single-extruder values for `retract_when_changing_layer`, `retraction_length`, `retraction_speed`, and `deretraction_speed`.
- Keep `crates/ares-core/src/options.rs` at or below 400 LOC. If registering the new module would exceed that limit, move the registration into an existing compact `option_modules!(...)` line or make an equally local module-list-only adjustment; do not add unrelated behavior or a broad module split in this slice.
- Extend `crates/ares-core/src/gcode_writer.rs` with explicit retract/unretract helpers that update the writer's current E state and emit `G1 E... F...` lines.
- Extend `crates/ares-core/src/gcode.rs` so non-first layer Z transitions emit retract before `G1 Z...`, and the first following print move emits unretract before the print move.
- Keep the pending-unretract state in `gcode.rs`. A layer-change retract sets that pending state; travel moves, labels, fan commands, custom layer commands, and diagnostics do not consume it. The first following `ToolpathMoveKind::Print` consumes it by emitting unretract immediately before that print move is passed to `gcode_move_emit`.
- Add focused runtime tests under `crates/ares-core/src/tests/layer_change_retraction_gcode.rs` and register them from `crates/ares-core/src/tests/mod.rs`.

## Included Behavior

- Default or `retract_when_changing_layer = false` preserves existing movement commands and emits no layer-change retract/unretract lines.
- `retract_when_changing_layer` accepts an omitted value as `false`, a scalar JSON bool, or a non-empty JSON bool array; arrays use index `0` for this single-extruder slice. Strings, numbers, mixed arrays, empty arrays, objects, or null values are invalid.
- `retraction_length`, `retraction_speed`, and `deretraction_speed` accept omitted values as Orca defaults (`0.8`, `30`, and `0`), scalar JSON numbers, scalar numeric strings, non-empty numeric JSON arrays, or semicolon/comma-separated numeric strings through the existing numeric-vector parser; arrays and numeric string lists use index `0` for this single-extruder slice.
- `retraction_length`, `retraction_speed`, and `deretraction_speed` values must be finite and non-negative. `deretraction_speed = 0` is valid and falls back to `retraction_speed`.
- With `retract_when_changing_layer = true` and positive `retraction_length`, Ares emits a retract line before each non-first layer Z travel.
- The retract line uses `-retraction_length` in relative extrusion mode and `current_e - retraction_length` in absolute extrusion mode.
- The retract feedrate is `retraction_speed * 60`.
- The first print move after the layer-change Z travel emits a matching unretract line before the print move.
- `deretraction_speed = 0` falls back to `retraction_speed`; otherwise unretract uses `deretraction_speed * 60`.
- `retraction_length = 0` disables layer-change retract/unretract even when `retract_when_changing_layer = true`.
- With `gcode_comments = false` or omitted, retract/unretract commands are bare `G1 E... F...` lines. With `gcode_comments = true`, retract commands append ` ; retract` and unretract commands append ` ; unretract`, matching the existing inline-comment gating style in Ares G-code output.
- Invalid bool vector values, negative lengths, negative speeds, non-finite values, or empty configured vectors fail slicing with `SliceError::InvalidInput` mentioning the relevant option key.

## Deferred Behavior

- Do not implement z-hop, slope lift, spiral lift, `z_hop_types`, `travel_slope`, `retract_lift_above`, or `retract_lift_below`.
- Do not implement wipe, `retract_before_wipe`, firmware retraction, toolchange retraction, long retractions, nozzle-cut retractions, or multi-extruder per-tool state.
- Do not implement travel-minimum retraction triggers, seam/scarf-specific retraction, object-change labels, or full Orca `GCode::retract` orchestration.
- Do not change path geometry, extrusion planning, speed planning, fan behavior, pressure advance, or viewer data.

## Acceptance Criteria

- Focused RED/GREEN is recorded with `cargo nextest run -p ares-core layer_change_retraction_gcode`.
- Related G-code movement coverage passes with `cargo nextest run -p ares-core layer_change_retraction_gcode gcode_comments z_offset_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC guard.
- Independent spec, plan, and implementation reviewers return `VERDICT: APPROVE`.

## Docs Impact

- No public user documentation update is required for this internal slicing-behavior slice.
- Update `docs/roadmap.md` with a concise source-cited note that the `retract_when_changing_layer` option family is now consumed in runtime G-code, after implementation review passes.
