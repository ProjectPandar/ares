# Consume Retraction Minimum Travel Design

## Source Boundary

Port a narrow runtime slice of OrcaSlicer ordinary travel retraction into `ares-core`.

Upstream sources:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5048-5054` defines `retraction_minimum_travel` as `coFloats`, default `{ 2. }`, and describes it as the travel-distance threshold for triggering retraction.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7280-7435` routes ordinary XY travel through `GCode::travel_to`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7458-7595` implements `GCode::needs_retraction`, first skipping retraction when `travel.length() < FILAMENT_CONFIG(retraction_minimum_travel)`, then retracting before eligible travel.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1004-1078` emits the E-axis retract/unretract commands used by ordinary travel retraction.

Rust destination:
- `crates/ares-core/src/options/layer_change_retraction.rs` remains the temporary retraction option boundary and will parse the first single-extruder `retraction_minimum_travel` value.
- `crates/ares-core/src/gcode.rs` will consume the parsed threshold while formatting normal XY travel moves.
- `crates/ares-core/src/gcode_layer_change_retraction.rs` may host reusable retract/unretract command helpers shared by layer-change and travel retraction.
- Tests live under `crates/ares-core/src/tests/` as a focused G-code runtime module.

## Included Behavior

This slice adds ordinary travel retraction for the current Ares single-active-extruder path:
- Parse `retraction_minimum_travel` from number, numeric string, or numeric list, selecting index zero, defaulting to Orca's `2.0` mm.
- Reject empty, negative, non-finite, or non-numeric values with `SliceError::InvalidInput` mentioning `retraction_minimum_travel`.
- Before each normal XY travel move after at least one print move, compute the XY travel distance from the current writer position to the travel target.
- If the travel distance is greater than or equal to `retraction_minimum_travel` and `retraction_length > 0`, emit a retract before the travel move and defer unretract until just before the next print move.
- Use existing first-extruder `retraction_length`, `retraction_speed`, `deretraction_speed`, `retract_restart_extra`, `use_firmware_retraction`, and relative/absolute E writer behavior.
- Keep layer-change retraction working independently; if a layer-change unretract is already pending, do not add a second ordinary-travel retract before that pending unretract is resolved.
- Keep `retract_when_changing_layer = false` from disabling ordinary travel retraction; Orca's ordinary travel retraction depends on travel distance and retract length, not that layer-change-only boolean.

## Deferred Behavior

This slice explicitly does not implement:
- Wipe while retracting: `wipe`, `wipe_distance`, `wipe_speed`, `role_based_wipe_speed`, or `retract_before_wipe`.
- Full Orca `GCode::travel_to` avoid-crossing-perimeters, support-island, internal-infill, overhang-detection, and short-travel acceleration/jerk branches.
- `z_hop_types`, `travel_slope`, lazy/eager lift for ordinary travel, slope lift, spiral lift, and auto lift selection.
- Toolchange retraction, wipe tower travel, multi-extruder per-filament state, or per-tool retraction state.
- Filament override normalization beyond the first effective scalar already used by Ares retraction parsing.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core travel_retraction_gcode` fails before implementation because long ordinary travel does not emit retract/unretract.
- After implementation, the focused nextest command passes.
- Tests prove:
  - default `retraction_minimum_travel = 2.0` emits ordinary travel retract/unretract for a long inter-path travel even when `retract_when_changing_layer` is false;
  - a threshold above the same travel distance suppresses ordinary travel retraction;
  - `retraction_minimum_travel` list values use index zero;
  - `retract_restart_extra` affects the ordinary-travel unretract length without changing the retract length;
  - invalid `retraction_minimum_travel` values are rejected with the option key.
- Existing layer-change retraction tests still pass.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.

## Documentation Impact

Update `docs/roadmap.md` with a concise source-cited entry after implementation. The roadmap entry must state included behavior and deferred upstream behavior so future milestones do not treat ordinary travel retraction as full Orca retraction parity.
