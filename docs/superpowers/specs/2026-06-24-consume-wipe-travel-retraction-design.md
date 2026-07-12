# Consume Wipe Travel Retraction Design

## Goal

Consume the existing OrcaSlicer `wipe`, `wipe_distance`, and `retract_before_wipe` options in Ares' ordinary travel-retraction G-code path. This slice must emit concrete wipe-and-retract G-code when a normal travel move retracts after a printed segment, instead of leaving the options as registry/config metadata only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1367`: declares `retract_before_wipe` as `ConfigOptionPercents`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1569`: declares `wipe` as `ConfigOptionBools`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1573`: declares `wipe_distance` as `ConfigOptionFloats`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5055-5060`: defines `retract_before_wipe`, default `100%`, as the fast retraction share before wipe.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6628-6641`: defines `wipe` and `wipe_distance`, default disabled and `1mm`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:312-360`: calculates retraction split before and during wipe from retraction length, `retract_before_wipe`, and `wipe_distance`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:426-505`: emits wipe movement along the stored prior extrusion path and marks it as "wipe and retract".
- `OrcaSlicer/src/libslic3r/GCode.cpp:7589-7599`: invokes wipe before the remaining normal retract when wipe is enabled.

## Ares Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction.rs`: parse the first effective single-extruder `wipe`, `wipe_distance`, and `retract_before_wipe` values into the existing `LayerChangeRetraction` runtime options.
- `crates/ares-core/src/options/validation/firmware_retraction.rs`: preserve the existing validation that rejects `use_firmware_retraction=true` when any effective `wipe` bool-vector value is true.
- `crates/ares-core/src/gcode_writer/retraction.rs`: add a focused `GCodeWriter` helper that emits one XY wipe command with optional `E` and explicit inline `F`, for example `G1 X0.5 Y0 E-0.4 F7200 ; wipe and retract` or `G1 X0.5 Y0 F7200 ; wipe and retract` when the wipe has zero E. Do not custom-format wipe lines in `gcode_travel_retraction.rs`, and do not grow `gcode_writer.rs`; keep the helper in the existing retraction submodule.
- `crates/ares-core/src/gcode_writer/tests/retraction.rs`: lock relative-E, absolute-E, and zero-E formatting for that writer helper.
- `crates/ares-core/src/gcode_travel_retraction.rs`: split ordinary travel retraction into before-wipe and during-wipe portions, accept the current travel move feedrate on `TravelRetractionCommand`, use that feedrate for the wipe helper's explicit `F` field, emit an optional wipe move, and keep unretract/z-hop state behavior unchanged.
- `crates/ares-core/src/gcode.rs`: pass enough previous-print-segment information into `TravelRetractionState` so the wipe move follows the previous printed segment in Ares' current straight-segment path model, and pass the current travel `speed_move.feedrate_mm_min()` into `TravelRetractionCommand`. Do not infer wipe feedrate from `writer.current_feedrate()`, because before travel retraction that state may still reflect the previous emitted move. This file is already near the repository LOC guard, so implementation must keep the final file at or below 400 LOC. Prefer a net-small local edit; if that cannot stay under the guard, extract the observation glue into a new private helper module instead of growing `gcode.rs`.
- `crates/ares-core/src/tests/travel_retraction_gcode/wipe.rs`: add focused output and validation tests.
- `crates/ares-core/src/tests/travel_retraction_gcode.rs`: register the new `mod wipe;` test module.

## Included Behavior

1. Parse `wipe` from a bool or bool vector, using the first value and defaulting to `false`.
2. Parse `wipe_distance` from a numeric scalar/vector/string list, using the first value, defaulting to `1.0`, and rejecting empty, non-finite, or negative values.
3. Parse `retract_before_wipe` as a percent scalar/vector/string list, using the first value divided by `100.0`, defaulting to `100%`, and rejecting empty, non-finite, negative, or over-`100%` values.
4. When ordinary travel retraction would already happen, `wipe = true`, `wipe_distance > 0`, `retraction_length > 0`, firmware retraction is disabled, and the previous printed move has a non-zero segment, emit:
   - a fast before-wipe retract for `retraction_length * retract_before_wipe`,
   - then a `wipe and retract` XY move along the tail of the previous printed segment for the remaining retract amount that can fit into the configured wipe distance,
   - then keep the existing pending unretract and optional Z-hop behavior.
5. If `retract_before_wipe = 100%`, emit all retraction before wipe and still emit a non-extruding wipe move along the prior segment, matching Orca's "allow 100% retract before wipe" path.
6. If the previous printed segment is shorter than `wipe_distance`, clamp the wipe move to the available segment length.
7. If the wipe path can only consume part of the remaining retraction, emit the leftover as a normal retract after the wipe so total retracted E still equals `retraction_length`.
8. Preserve current behavior when `wipe` is omitted, `false`, `wipe_distance = 0`, no previous print path exists, travel distance is below `retraction_minimum_travel`, or `reduce_infill_retraction` suppresses the travel retraction.
9. Preserve the existing firmware-retraction validation in `crates/ares-core/src/options/validation/firmware_retraction.rs`: `use_firmware_retraction=true` plus any true `wipe` value returns an error under `use_firmware_retraction`. Do not add wipe output for firmware retraction.

## Detailed Wipe Algorithm

- Validation applies to all supplied values for `wipe_distance` and `retract_before_wipe`, not only the first effective value. A vector or string list containing a later invalid entry is rejected even when the first value is valid.
- `wipe_distance` uses the first validated value in millimeters.
- `retract_before_wipe` uses the first validated percent divided by `100.0`.
- `before_wipe_retract = retraction_length * retract_before_wipe`.
- `remaining_retract = retraction_length - before_wipe_retract`.
- `available_wipe_distance = min(wipe_distance, previous_print_segment_length)`.
- The wipe move starts at the current writer XY position, which is the end of the previous printed segment, and moves backward along that segment for `available_wipe_distance`.
- `during_wipe_retract = min(remaining_retract, available_wipe_distance)`. This keeps this slice deterministic without porting Orca's wipe-speed and retraction-speed time calculation.
- `leftover_retract = remaining_retract - during_wipe_retract`.
- Emit the before-wipe retract if `before_wipe_retract > 0`.
- Emit the wipe move whenever `available_wipe_distance > 0`, including a zero-E wipe for `retract_before_wipe = 100%`.
- Emit leftover retract after the wipe if `leftover_retract > 0`.
- Use the current travel move feedrate for the wipe XY move and emit that feedrate explicitly on the wipe line. `gcode.rs` must pass `speed_move.feedrate_mm_min()` into `TravelRetractionCommand`, and `gcode_travel_retraction.rs` must pass that command field into the writer helper. Do not use `writer.current_feedrate()` as the wipe feedrate source. `wipe_speed` and `role_based_wipe_speed` are deferred and must not be added in this slice.

Representative expected G-code for a previous printed segment from `(0,0)` to `(1,0)`, an ordinary travel from `(1,0)` to `(2,0)`, `retraction_length = 0.8`, `retraction_speed = 30`, `z_hop = 0`, `travel_speed = 120`, and `gcode_comments = true`:

```gcode
; default wipe=false
G1 E-0.8 F1800 ; retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract

; wipe=true, wipe_distance=0.5, retract_before_wipe=50
G1 E-0.4 F1800 ; retract
G1 X0.5 Y0 E-0.4 F7200 ; wipe and retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract

; wipe=true, wipe_distance=0.5, retract_before_wipe=100
G1 E-0.8 F1800 ; retract
G1 X0.5 Y0 F7200 ; wipe and retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract

; wipe=true, wipe_distance=0.25, retract_before_wipe=0
G1 X0.75 Y0 E-0.25 F7200 ; wipe and retract
G1 E-0.55 F1800 ; retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract

; wipe=true, wipe_distance=2.0, retract_before_wipe=0, previous segment length=1.0
G1 X0 Y0 E-0.8 F7200 ; wipe and retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract
```

Suppressed cases keep today's output and emit no wipe line: `wipe=false`, `wipe_distance=0`, missing prior print segment, travel shorter than `retraction_minimum_travel`, `reduce_infill_retraction=true` suppressing the travel retraction, or `use_firmware_retraction=true` rejected by existing validation when `wipe=true`.

## Deferred Behavior

- Full Orca wipe-path storage from multiple prior perimeter points, loop clipping, reverse path traversal across arbitrary polylines, and `m_wipe.path` parity.
- Toolchange wipe, wipe tower, MMU/MMU2, filament-specific current-tool selection beyond Ares' current first-extruder runtime path.
- `role_based_wipe_speed`, `wipe_speed`, cooling marker tags, GCodeProcessor wipe start/end reserved tags, adaptive pressure-advance wipe handling, and avoid-crossing-perimeters interactions.
- Full Orca E-state partial-retraction tracking. This slice preserves Ares' current pending-unretract model and ensures the total emitted retract amount for one travel equals the configured travel retraction length.
- UI, CLI, file I/O, OpenGL, or new public API behavior.

## Tests And Acceptance

- Add RED tests before implementation in `crates/ares-core/src/tests/travel_retraction_gcode/wipe.rs`.
- Focused RED command: `cargo nextest run -p ares-core travel_retraction_wipe`.
- Focused GREEN command after implementation: `cargo nextest run -p ares-core travel_retraction_wipe`.
- Adjacent regression commands:
  - `cargo nextest run -p ares-core -- travel_retraction_gcode`
  - `cargo nextest run -p ares-core -- layer_change_retraction_gcode`
  - `cargo nextest run -p ares-core -- firmware_retraction_rejects_wipe`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust LOC guard at or below 400 lines.

## Documentation

After implementation review approval, update `docs/roadmap.md` to record that `wipe`, `wipe_distance`, and `retract_before_wipe` now affect ordinary Ares travel-retraction G-code, and list the deferred full Orca wipe-path/toolchange/wipe-tower behavior.

## Safety And Platform Constraints

Code and runtime behavior changes stay inside `ares-core`, use deterministic in-memory G-code formatting, and remain compatible with WASM, Windows, macOS, and Linux. The only planned non-`ares-core` change is the post-implementation `docs/roadmap.md` note required above. No dependencies, filesystem access, terminal behavior, UI, OpenGL, or Ares-owned independent pipeline design are added. Any touched Rust file that is near the 400 LOC guard must either receive a net-small edit that leaves it at or below 400 LOC, or move the added behavior into an existing/new private submodule with a narrow boundary. In this slice, the XY+E+F writer helper belongs in `gcode_writer/retraction.rs` specifically to avoid growing `gcode_writer.rs`.
