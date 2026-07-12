# Consume Ordinary Travel Z-hop Design

## Goal

Consume the already parsed `z_hop`, `retract_lift_above`, and `retract_lift_below` options in ordinary travel retraction G-code so Ares emits the same normal vertical lift/restore behavior Orca applies when a travel move triggers retraction.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`, not an Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5048-5054` defines `retraction_minimum_travel`, the threshold already consumed by Ares ordinary travel retraction.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122-5131` defines `z_hop` as a per-filament float vector, default `0.4`, with the tooltip that retraction lifts the nozzle to create travel clearance.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5133-5147` defines `retract_lift_above` and `retract_lift_below`, the lower and upper Z-hop boundaries.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7280-7435` routes ordinary travel moves through `GCode::travel_to`, asks `needs_retraction`, calls `retract(...)` before eligible travel, then emits the travel move.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7458-7580` implements `needs_retraction`, including the minimum-travel gate and lift-type selection.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7582-7637` implements `GCode::retract`; after retracting it evaluates lift enforcement and calls `m_writer.lazy_lift(...)`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-648` implements `lazy_lift`, reading `retract_lift_above`, `retract_lift_below`, and `z_hop`, then deferring a lift until the following travel.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1084-1092` implements `unlift`, restoring the layer Z before continuing after a lifted retraction.

## Rust Destination

- `crates/ares-core/src/options/layer_change_retraction.rs` remains the temporary retraction option boundary and already parses `z_hop`, `retract_lift_above`, and `retract_lift_below`.
- `crates/ares-core/src/gcode.rs` consumes ordinary travel retraction state while formatting G-code.
- Add `crates/ares-core/src/gcode_travel_retraction.rs` for ordinary travel retraction state and threshold decisions so `gcode.rs`, currently at the 400 LOC guard, does not keep growing.
- Reuse `crates/ares-core/src/gcode_layer_change_retraction.rs` retract/unretract/Z-restore helpers until a broader retraction module split is justified.
- Extend `crates/ares-core/src/tests/travel_retraction_gcode.rs` with a focused child module under `crates/ares-core/src/tests/travel_retraction_gcode/` so the parent test file stays under the 400 LOC guard.

## Included Behavior

- When an ordinary XY travel move triggers retraction, compute a normal vertical Z-hop from the current writer Z using `LayerChangeRetraction::z_hop_for_z`.
- Emit retract first, then emit `G1 Z<current + z_hop> ... ; lift Z`, then emit the XY travel move.
- Before the next print move, emit `G1 Z<original_z> ... ; restore layer Z`, then unretract, then the print move.
- Apply the same behavior for E-axis retraction and firmware retraction (`G10`/`G11`), because Orca's lift is tied to the retraction event, not to the E-axis command shape.
- Respect `retract_lift_above` and `retract_lift_below`; if the current Z is outside the allowed range, ordinary travel retraction still retracts/unretracts but emits no lift/restore.
- If a lifted ordinary travel retraction remains pending across a layer boundary, the new layer Z move supersedes the old restore target. The first print on the new layer must not restore down to the previous layer's Z.
- Preserve the existing guard that suppresses layer-change retraction while an ordinary travel unretract is pending.

## Deferred Behavior

- `z_hop_types`, `travel_slope`, slope lift, spiral lift, auto lift, and travel-path Z interpolation remain deferred.
- Orca avoid-crossing-perimeters, support-island checks, internal-infill retraction suppression, overhang detection, wipe, and wipe tower travel remain deferred.
- Toolchange retraction, multi-extruder per-filament Z-hop state, filament override normalization beyond the existing first effective scalar, and lazy lift coalescing across multiple independent travel planners remain deferred.
- `retract_lift_enforce` for ordinary travel remains deferred. Ares currently uses that enum for layer-change lifts based on previous print role; this slice ports the writer-level Z range gate for ordinary travel and does not invent missing role/top-bottom semantics beyond the current Ares state.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core travel_retraction_gcode` fails before implementation because ordinary travel retraction emits no Z lift/restore.
- After implementation, the same focused nextest command passes.
- Tests prove:
  - default `z_hop = 0.4` lifts after ordinary travel retract and restores before ordinary travel unretract;
  - `z_hop = 0` preserves no-hop ordinary travel retraction;
  - `retract_lift_above` and `retract_lift_below` suppress ordinary travel Z-hop outside their allowed range while retaining retract/unretract;
  - firmware ordinary travel retraction lifts/restores around `G10`/`G11`;
  - a lifted ordinary travel retraction crossing a layer boundary does not restore to the previous layer Z before the first print on the new layer.
- Existing layer-change retraction and ordinary travel retraction tests still pass.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The change is confined to platform-neutral `ares-core` G-code formatting and tests. It adds no dependencies, filesystem access, terminal behavior, UI, OpenGL, or public API shape changes. Rollback is removing the ordinary travel Z-hop state changes, focused tests, module registration, and this spec/plan pair.
