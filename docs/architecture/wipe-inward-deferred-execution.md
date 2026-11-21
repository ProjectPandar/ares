# Execute inward wipe paths during retraction

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r/GCode.cpp`:

- `:8766–8817`: an accepted inward offset replaces the stored wall wipe path. The loop does not execute that path. Preserve the separate `wipe_on_loops` pre-move (`:8822–8827`).
- `:10838–10852`: the existing retract path consumes the stored wipe using the same executor as ordinary wiping. Short travels that do not retract do not execute it. `:538–602` computes wipe-time retraction, and `:699–744` replaces the start with the current nozzle position, clips the path, and emits marked XY moves with the computed relative E.

In `crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/wipe_inward/emit.rs`, `finish_loop` should retain the accepted offset for the existing `motion/travel/wipe_sequence.rs` retract executor. The immediate zero-retraction call is specific to the earlier beta behavior and is not a compatibility path for v2.5.0. Keep the offset geometry, ordinary path fallback, travel policy, configuration and formatter unchanged.

## Public behavior and checks

Use the existing Classic Inner/Outer fixtures in `tests/parity/wipe-inward/` with the strict whole-output Native/API/CLI comparison. An inward witness is a marked wipe after a closed outer wall with an earlier inner wall and at least one wipe XY endpoint strictly inside that wall's bounds. For the enabled fixture, the witness must also include a wipe XY move with negative relative E. Wipe geometry is independent of E; neither zero E nor negative E alone indicates an inward path.

In each control, reject that geometry regardless of E and require a retracting wipe on the closed outer wall boundary. The enabled single-wall case must still contain no inner-wall extrusion and no interior wipe after a closed outer wall. Use stable Native double replay; do not change the fixture bytes, comparator or normalized fields.

## Failure modes

- A zero-E-only observer rejects the valid Native offset before comparing outputs; negative E alone would admit an ordinary boundary wipe.
- Executing at loop completion emits a wipe before Native's object/layer boundary and without wipe-time retraction. It also bypasses the travel decision and would wipe on a short travel that otherwise would not retract.
- Dropping or replacing the accepted path too early produces an ordinary contour wipe instead of the inward offset at retraction. Changing the ordinary fallback regresses disabled and single-wall prints.
- Changing retract bookkeeping, markers or feed formatting can preserve XY while still changing the ordered G-code. Compare the entire stream, not just the offset moves.
