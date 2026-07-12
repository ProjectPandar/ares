# Consume wipe_on_loops Design

Consume OrcaSlicer's `wipe_on_loops` option into concrete Ares G-code behavior. When enabled, Ares appends a zero-extrusion inward move immediately after a supported external perimeter loop finishes, so the nozzle moves slightly inside before the following travel.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1185` declares `PrintRegionConfig::wipe_on_loops`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5510-5515` registers `wipe_on_loops` as a boolean option with default `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5926-5961` emits a no-extrusion "move inwards before travel" after external perimeter loop extrusion when `wipe_on_loops` is true and `wall_loops > 1`.

## Ares Boundary

- Parse `wipe_on_loops` through `SliceOptions`, defaulting to `false` and rejecting non-boolean input with `SliceError::InvalidInput` naming the option.
- Implement G-code emission in a new focused `ares-core` module, called from `crates/ares-core/src/gcode.rs` after an external perimeter print move is formatted and before the next travel/retraction path is emitted.
- Use existing in-memory path and move artifacts only: `LayerPrintPaths`, `ToolpathMove`, `SpeedMove`, `GCodeWriter`, and configured hardware nozzle diameter. Do not add file I/O, terminal behavior, UI behavior, dependencies, or non-WASM-safe APIs.

## Behavior

When all conditions are true, append one `G1 X... Y... F...` line with no `E` value and comment `move inwards before travel` when comments are enabled:

1. `wipe_on_loops` is true.
2. Configured `wall_loops` is greater than `1`.
3. The current move is the closing print move of a closed `PrintPathRole::ExternalPerimeter` path.
4. The source path has at least three points and a non-zero first segment.
5. The closed perimeter has a generated closing target, so the wipe starts near the external loop seam rather than crossing the object from an open path end.

The target mirrors Orca's local loop-corner move for Ares' current path model:

- Start from the path's first point.
- Move along the first segment by `0.2 * min(first_segment_length, first_nozzle_diameter)`.
- Rotate that local point around the first point by one third of the signed angle from the incoming closing segment to the outgoing first segment.
- Emit the move at the current print move feedrate with zero extrusion.

The emitted wipe changes the writer XY position but not E. It must not create print-path, extrusion, speed, acceleration, jerk, or role-change diagnostics, because Orca emits it as a writer move after path extrusion rather than as a slicer path.

## Deferred Behavior

This slice does not implement full Orca loop semantics for holes, multi-part external loops, scarfed loops, split `ExtrusionMultiPath` state, Arachne variable-width loops, or exact `is_hole == loop.is_counter_clockwise()` winding-side handling. Those require a fuller port of Orca loop objects and G-code path assembly. This slice is limited to Ares' existing closed external perimeter paths, which currently cover generated rectangular classic perimeters.

`wipe_before_external_loop` remains separate. Enabling one option must not imply the other.

## Acceptance Criteria

- `wipe_on_loops` defaults to `false`.
- `wipe_on_loops: true` is accepted by `SliceOptions`.
- Non-boolean `wipe_on_loops` values return `SliceError::InvalidInput` and name `wipe_on_loops`.
- With `wall_loops = 2`, an external rectangular perimeter, zero seam gap, and `wipe_on_loops: true`, generated G-code contains an inward no-E move after the external perimeter closing extrusion and before the following internal-perimeter travel.
- With `gcode_comments: true`, that line contains `; move inwards before travel`.
- With `wipe_on_loops` missing or false, the same input contains no `move inwards before travel` line and matches the current output.
- With `wall_loops = 1`, `wipe_on_loops: true` emits no inward move.
- With `seam_gap` large enough to remove the generated external closing move, `wipe_on_loops: true` emits no inward move.
- `wipe_before_external_loop` tests continue to pass and no output line uses its `wipe before external loop` comment for `wipe_on_loops`.
- `docs/roadmap.md` records that `wipe_on_loops` now has a concrete rectangle-path G-code behavior slice while full Orca loop parity remains deferred.

## Verification

- RED before implementation: `cargo nextest run -p ares-core wipe_on_loops`
- GREEN after implementation: `cargo nextest run -p ares-core wipe_on_loops`
- Adjacent regression: `cargo nextest run -p ares-core wipe_before_external_loop travel_retraction_gcode::wipe`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust file LOC guard, with every touched `.rs` file at or below 400 lines
