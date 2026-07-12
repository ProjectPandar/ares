# Consume Ordinary Travel Lift-Enforce Design

## Goal

Consume the already parsed `retract_lift_enforce` option in ordinary travel retraction Z-hop so Ares gates travel lifts the same way Orca's `GCode::retract` gates all retraction lifts.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`, not an Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:390-395` defines `RetractLiftEnforceType`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5187-5200` defines `retract_lift_enforce`, its enum values, and default `All Surfaces`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7185-7187` updates `m_last_notgapfill_extrusion_role` after non-gap-fill print paths.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7280-7370` routes ordinary travel through `GCode::travel_to`, calls `needs_retraction`, then calls `GCode::retract(..., role)` before the travel move.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7582-7637` implements `GCode::retract`; it reads `retract_lift_enforce`, treats first-layer retractions as bottom-surface eligible, treats the last non-gap-fill top solid or ironing role as top-surface eligible, and only calls `lazy_lift` when the enforce gate allows.
- `OrcaSlicer/src/libslic3r/GCode.hpp:580-582` owns `m_last_notgapfill_extrusion_role` for this gate.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-648` applies the remaining `retract_lift_above`, `retract_lift_below`, and `z_hop` writer-level lift gate.

## Rust Destination

- `crates/ares-core/src/options/layer_change_retraction.rs` remains the temporary parsed retraction option boundary and already parses `retract_lift_enforce`.
- `crates/ares-core/src/gcode.rs` already tracks `last_non_gap_fill_print_role`; pass that state plus the current layer index into ordinary travel retraction.
- `crates/ares-core/src/gcode_travel_retraction.rs` owns the ordinary travel retraction/lift decision and should apply `RetractLiftEnforce::allows(...)` before using the already computed `z_hop`.
- `crates/ares-core/src/tests/travel_retraction_gcode/z_hop.rs` receives focused behavior tests for ordinary travel lift enforcement.
- `crates/ares-core/src/pipeline/layer_change_test_support.rs` may receive one focused helper for same-layer ordinary travel role scenarios if the existing square-pyramid fixture cannot express the role state directly.

## Included Behavior

- Ordinary travel retraction remains controlled by `retraction_minimum_travel`, retraction length, firmware/E-axis mode, and pending retraction state exactly as before.
- Ordinary travel Z-hop now also requires `retract_lift_enforce` to allow lifting:
  - `All Surfaces` keeps the current lift behavior.
  - `Top Only` lifts only when the previous non-gap-fill print role is `TopSolidInfill`.
  - `Bottom Only` lifts only for ordinary travel retractions on layer index `0`, matching Orca's `m_layer_index == 0` branch for first-layer retractions.
  - `Top and Bottom` lifts on either of those conditions.
- `GapFill` must not replace the previous non-gap-fill role for this gate; a top-solid print followed by gap fill followed by ordinary travel remains top-eligible.
- If the enforce gate suppresses lift, Ares must still emit the ordinary travel retract, XY travel, unretract, and next print move.
- Existing layer-change `retract_lift_enforce` behavior remains unchanged.
- Existing `z_hop`, `retract_lift_above`, and `retract_lift_below` gates still apply after the enforce gate.

## Deferred Behavior

- Orca ironing-role top eligibility remains deferred because Ares' current public print-role vocabulary has no separate ironing role in this path.
- Toolchange, wipe, wipe tower, support-island/internal-infill suppression, avoid-crossing-perimeters, full `z_hop_types`, slope/spiral/auto lift, multi-extruder per-tool lift enforcement, and full Orca `GCode::retract` orchestration remain deferred.
- This slice does not change layer-change lift enforcement, retraction length/speed, restart extra, firmware retraction command formatting, or travel path planning.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core travel_retraction_gcode` fails before implementation because ordinary travel still lifts for `retract_lift_enforce = "Top Only"` after a non-top print role.
- After implementation, that focused nextest command passes.
- Tests prove:
  - `Top Only` suppresses ordinary travel lift/restore after a non-top previous non-gap-fill role while retaining retract/unretract;
  - `Top Only` allows ordinary travel lift/restore after `TopSolidInfill`;
  - `Bottom Only` allows ordinary travel lift/restore on layer `0` and suppresses it on later layers;
  - `Top and Bottom` allows a later-layer ordinary travel lift after `TopSolidInfill`;
  - `GapFill` does not overwrite a previous `TopSolidInfill` role for the ordinary travel lift gate.
- Existing layer-change retraction and ordinary travel retraction tests still pass.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The change is confined to platform-neutral `ares-core` G-code formatting and tests. It adds no dependencies, filesystem access, terminal behavior, UI, OpenGL, or public API changes. Rollback is removing the ordinary travel lift-enforce fields/tests and this spec/plan pair.
