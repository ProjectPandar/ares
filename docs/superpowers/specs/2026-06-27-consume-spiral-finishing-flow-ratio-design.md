# Consume Spiral Finishing Flow Ratio Design

## Scope

Consume OrcaSlicer `spiral_finishing_flow_ratio` into concrete Ares vase-mode G-code behavior. This is a source-cited `libslic3r` rewrite slice, not a new Ares-owned vase pipeline.

Upstream boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1563` declares `spiral_finishing_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5717-5726` defines the option as a float in the inclusive range `0..=1`, defaulting to `0`.
- `OrcaSlicer/src/libslic3r/GCode/SpiralVase.cpp:122-126` enables transition-out only on `last_layer` and only when `use_relative_e_distances` is enabled.
- `OrcaSlicer/src/libslic3r/GCode/SpiralVase.cpp:152-160` clones each final-layer extrusion move into `transition_gcode`, scaling E by `finishing_flowrate + ((1 - factor) * (1 - finishing_flowrate))`.
- `OrcaSlicer/src/libslic3r/GCode/SpiralVase.cpp:207-215` appends the transition-out G-code after the normal transformed layer.

Ares destination boundary:

- `crates/ares-core/src/gcode_spiral_vase.rs` owns the vase-mode transition state.
- `crates/ares-core/src/gcode_spiral_vase_transition.rs` owns appended transition-out G-code emission.
- `crates/ares-core/src/gcode.rs` owns layer iteration and final-layer append timing, but must stay at or below 400 LOC. Any implementation that needs more than a minimal call site must extract logic into the focused module above.
- `crates/ares-core/src/pipeline/tests/spiral_finishing_flow_ratio_gcode.rs` covers public G-code behavior.
- `docs/roadmap.md` records the consumed slice and remaining deferred upstream behavior.

## Current Behavior

Ares already consumes `spiral_starting_flow_ratio` for relative-E vase-mode transition-in on the first body layer after normalized bottom shell layers. `spiral_finishing_flow_ratio` is still deferred: changing the option does not append a final transition-out loop or change final-layer extrusion output.

## Required Behavior

- Parse `spiral_finishing_flow_ratio` as a finite float in `0..=1`, default `0`.
- Keep finishing-flow behavior disabled unless both `spiral_mode = true` and `use_relative_e_distances = true`.
- Apply transition-out only to Ares' final emitted layer, matching Orca's `last_layer` gate for the current Ares layer pipeline.
- Preserve the normal final-layer G-code output, then append a duplicate transition-out sequence of Ares final-layer print moves.
- Scale duplicate print-move relative E deltas by Orca's transition-out factor:
  `spiral_finishing_flow_ratio + ((1 - progress) * (1 - spiral_finishing_flow_ratio))`.
- Use end-of-move printed-XY progress over final-layer print XY length, consistent with the existing transition-in approximation.
- Duplicate only Ares final-layer print moves in this slice. Re-emit each duplicate with the same XY target, feedrate, print acceleration, print jerk, speed marker, move marker, role marker, and optional G-code comment style used by ordinary Ares print moves. Do not duplicate final-layer travel moves, fan transitions, object-label commands, retraction/unretraction, pressure-advance changes, role-change custom G-code, or layer-change commands.
- Do not carry the transition-out duplicate extrusion into following moves; it is appended at print end and should not cause a later catch-up E move.
- Ignore the option when absolute-E mode is selected, matching Orca's relative-E-only tapering gate.
- Preserve existing `spiral_starting_flow_ratio` transition-in behavior.
- Update `docs/roadmap.md` by adding a new `spiral_finishing_flow_ratio` runtime entry above the starting-flow entry, and remove `spiral_finishing_flow_ratio` from the starting-flow entry's deferred list. The new entry must keep full continuous-Z spiral post-processing, XY smoothing, short-segment filtering, absolute-E tapering, non-print transition G-code duplication, and full Orca SpiralVase parity deferred.

## Non-Goals

- Do not implement full Orca continuous-Z spiral post-processing.
- Do not implement `spiral_mode_smooth` or `spiral_mode_max_xy_smoothing`.
- Do not filter short final-layer segments by `2 * resolution`; Ares does not yet own that full `SpiralVase.cpp` post-processor boundary.
- Do not duplicate non-print G-code into `transition_gcode`; Ares' current layer loop emits retraction, fan, pressure-advance, labels, and layer custom G-code outside the final-layer print move abstraction.
- Do not implement absolute-E finishing tapering.
- Do not add new crates, dependencies, file I/O, UI behavior, OpenGL behavior, or terminal behavior in `ares-core`.

## Acceptance Criteria

- A focused nextest test fails before implementation and passes after implementation, proving `spiral_finishing_flow_ratio = 0.0` appends lower-E final-layer transition-out print moves after the normal final-layer output.
- `spiral_finishing_flow_ratio = 1.0` appends duplicate final-layer print moves with the same relative E values as the normal final-layer print moves.
- The option is ignored when `use_relative_e_distances = false`.
- The option is ignored when `spiral_mode = false`.
- Invalid values outside `0..=1` or non-numeric values return `SliceError::InvalidInput` mentioning `spiral_finishing_flow_ratio`.
- Existing `spiral_starting_flow_ratio` coverage still passes.
- Full verification uses `cargo nextest run`, not `cargo test`.
