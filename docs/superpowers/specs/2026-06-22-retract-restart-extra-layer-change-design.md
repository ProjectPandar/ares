# Retract Restart Extra Layer-Change Runtime Design

## Objective

Consume the existing OrcaSlicer `retract_restart_extra` option in Ares' concrete layer-change retraction G-code. Ares already emits configured `retraction_length` retract and unretract moves for `retract_when_changing_layer`; this slice makes the matching unretract move push `retraction_length + retract_restart_extra`, matching Orca's restart-extra semantics for ordinary retractions.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5306-5313` defines `retract_restart_extra` as `ConfigOptionFloats`, labels it "Extra length on restart", documents that it pushes additional filament after travel, and defaults it to `0.`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1004-1078` passes `factor * filament()->retract_restart_extra()` into `_retract(...)`; the later `unretract()` emits the filament state after `filament()->unretract()`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:4806-4817` emulates firmware `G11` unretract as `retraction_length + retract_restart_extra`, confirming the restart-extra distance belongs to unretract, not retract.

## Ares Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction.rs` owns the typed runtime extraction for Ares' current layer-change retraction slice.
- `crates/ares-core/src/gcode.rs` owns the current layer-change retract/unretract placement around non-first-layer Z transitions.
- `crates/ares-core/src/gcode_move_emit.rs` owns the final print-move E target passed to `GCodeWriter`; this slice may carry a local E-position offset so print moves continue from restart-extra unretract state without changing path planning.
- `crates/ares-core/src/tests/layer_change_retraction_gcode.rs` owns the focused G-code tests.

## Included Behavior

- Parse `retract_restart_extra` from the existing dynamic options using the same first single-extruder numeric forms accepted by `retraction_length`: scalar number, numeric string, non-empty numeric array, comma-separated string, or semicolon-separated string.
- Use Orca's default `0.0` when `retract_restart_extra` is absent.
- Reject empty, negative, or non-finite `retract_restart_extra` values with `SliceError::InvalidInput` that includes the option key.
- Preserve existing `retract_when_changing_layer` and `retraction_length` enablement: if layer-change retraction is disabled or `retraction_length == 0`, no retract or unretract G-code is emitted.
- Keep the retract move distance equal to `retraction_length`.
- Emit the pending layer-change unretract move with distance `retraction_length + retract_restart_extra`.
- Keep later print moves in the same actual E coordinate frame by carrying the emitted restart-extra offset through G-code move emission.
- Preserve existing feedrate behavior: `deretraction_speed == 0` still falls back to `retraction_speed`; non-zero `deretraction_speed` controls unretract feedrate.
- Preserve relative and absolute E state semantics through the existing `GCodeWriter`.
- Preserve comment-on/comment-off formatting and existing layer-change placement.

## Deferred Behavior

- `retract_restart_extra_toolchange`, `retract_length_toolchange`, and tool-change retraction remain deferred.
- `retract_before_wipe`, wipe during retract, z-hop, firmware retraction `G10`/`G11`, long/nozzle-cut retraction, travel-minimum triggers, seam/scarf interaction, multi-extruder filament state, and full Orca `GCode::retract` orchestration remain deferred.
- The later Option pinning cleanup removes source-line-only tuple tests without changing this runtime-consumption behavior.

## Docs Impact

Update `docs/roadmap.md` after implementation review to add a new 2026-06-22 runtime slice entry for `retract_restart_extra`. The entry must state that the option now affects layer-change unretract E distance and must keep the same deferred upstream behavior boundaries as this spec. Do not rewrite the existing layer-change retraction entry beyond what is needed to avoid stale wording.

## Acceptance Criteria

- With `retract_when_changing_layer = true`, `retraction_length = 0.5`, `retract_restart_extra = 0.12`, and comments enabled, second-layer G-code contains `G1 E-0.5 ... ; retract` and `G1 E0.62 ... ; unretract`; it must not retract by `0.62`.
- With vector/string inputs, Ares uses only the first single-extruder `retract_restart_extra` value.
- With custom `deretraction_speed`, the restart-extra unretract uses the deretraction feedrate.
- In absolute E mode, the E delta from retract line to unretract line increases by `retraction_length + retract_restart_extra`, and the next print extrusion remains after the unretract state.
- Invalid `retract_restart_extra` input fails before successful G-code output and includes `retract_restart_extra` in the error.
- Existing default layer-change retraction tests still pass when `retract_restart_extra` is absent.
- Fresh verification passes: focused nextest for layer-change retraction, related G-code tests, workspace nextest, rustfmt, clippy, wasm check, `git diff --check`, and touched Rust LOC guard.

## Safety and Rollback

The change is narrow and reversible: removing the new parsed field and returning `gcode.rs` to the old unretract length restores prior behavior. No new dependencies, public API, filesystem access, UI behavior, OpenGL behavior, terminal behavior, or platform-specific code are introduced.
