# Consume Spiral Mode XY Smoothing Design

## Goal

Consume OrcaSlicer `spiral_mode_smooth` and `spiral_mode_max_xy_smoothing` as concrete spiral-vase G-code behavior in `ares-core`, instead of leaving them as option metadata only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1561-1562` declares:
  - `spiral_mode_smooth`
  - `spiral_mode_max_xy_smoothing`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5686-5704` defines `spiral_mode_smooth` default `false`, `spiral_mode_max_xy_smoothing` default `200%`, `ratio_over = "nozzle_diameter"`, and range `0..=1000`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3637-3641` and `3738-3741` resolve `spiral_mode_max_xy_smoothing` against nozzle diameter and pass it into `SpiralVase`.
- `OrcaSlicer/src/libslic3r/GCode/SpiralVase.hpp:20-46` stores `spiral_mode_smooth` and max XY smoothing.
- `OrcaSlicer/src/libslic3r/GCode/SpiralVase.cpp:114-184` smooths printed XY moves by finding the nearest point on the previous spiral layer and interpolating current-layer XY toward that point when the nearest distance is below `spiral_mode_max_xy_smoothing`.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_spiral_vase.rs` owns spiral-vase option parsing, run state, previous-layer original point storage, nearest-point smoothing, and E-length scaling for this slice.
- `crates/ares-core/src/gcode_move_emit.rs` is the narrow G-code move emission integration point; it should consume the adjusted XY target and adjusted E target returned by spiral-vase state.
- `crates/ares-core/src/gcode.rs` may only replace existing spiral-vase setup/call-site lines with equivalent run-state calls. It must not gain net LOC because it is already at the 400 LOC limit.
- Pipeline tests belong under `crates/ares-core/src/pipeline/tests/`.

## Included Behavior

- Parse `spiral_mode_smooth` as a boolean option with Orca default `false`.
- Parse `spiral_mode_max_xy_smoothing` as numeric millimeters or percent over the first nozzle diameter, with Orca default `200%` resolving to `2.0 * nozzle_diameter`.
- Validate the resolved effective millimeter distance as `0.0..=1000.0`; reject non-numeric, non-finite, negative, or greater-than-1000 resolved values with an error mentioning `spiral_mode_max_xy_smoothing`.
- When `spiral_mode` and `spiral_mode_smooth` are both enabled, smooth printed XY moves on spiral-vase layers using the previous layer's original unsmoothed printed target points. This XY smoothing must not be gated by `use_relative_e_distances`; Orca only uses relative-E gating for the existing transition tapering behavior.
- Store each current layer's original unsmoothed printed target points, before any XY target adjustment, as the next layer's previous-point polyline. Do not store emitted smoothed XY targets in this buffer.
- For each printed move with positive original XY distance and a previous-layer nearest polyline point closer than `spiral_mode_max_xy_smoothing`, interpolate target XY as:
  - `nearest_previous * (1 - layer_progress) + current_point * layer_progress`
- Scale that move's E delta by `smoothed_xy_distance / original_xy_distance`, matching Orca's length-ratio adjustment.
- Preserve existing `spiral_starting_flow_ratio` and `spiral_finishing_flow_ratio` behavior, including their current relative-E gate.
- Keep XY smoothing off when `spiral_mode_smooth` is false, `spiral_mode` is false, no previous layer is available, or the nearest previous-layer distance is at or beyond the max smoothing distance. A max distance of `0` therefore disables XY smoothing by construction because the upstream condition is strict `< max`.

## Deferred Behavior

- Do not rewrite Ares into Orca's string-based `GCodeReader` filter.
- Do not add arc fitting, pressure equalizer behavior, wipe tower behavior, or full `SpiralVase::process_layer` parity beyond this XY smoothing slice.
- Do not implement nearest-point acceleration structures; a simple deterministic scan over the previous printed polyline is enough for this slice.
- Do not implement Orca's travel-line skipping in smooth spiral mode; this slice adjusts printed move endpoints and E length only.
- Do not implement Orca's degenerate smoothed-line clearing when the smoothed segment becomes zero-length; tests for this slice must use positive-length smoothed segments.
- Do not implement Orca's separate raw percent maximum and literal millimeter maximum split for `spiral_mode_max_xy_smoothing`; this slice validates the resolved effective millimeter value as `0.0..=1000.0`.
- Do not add new crates or dependencies.

## Acceptance Criteria

- A RED pipeline test demonstrates that enabling `spiral_mode_smooth` and a large `spiral_mode_max_xy_smoothing` changes second body-layer XY output when the second layer contour is horizontally shifted from the first.
- A second test demonstrates that `spiral_mode_max_xy_smoothing = 0` or a too-small threshold preserves the unsmoothed shifted XY output.
- A test demonstrates that a third shifted layer is smoothed against the second layer's original unsmoothed targets, not the second layer's emitted smoothed XY targets.
- A test demonstrates that XY smoothing still applies when `use_relative_e_distances` is `false`, while existing starting/finishing flow-ratio transition behavior remains relative-E-gated.
- A validation test demonstrates percent parsing over nozzle diameter, default `200%` resolution, and invalid value rejection including resolved values greater than `1000`.
- Existing spiral starting/finishing flow tests continue to pass.
- Verification uses `cargo nextest run`; no new `cargo test` workflow is introduced.

## Safety And Platform Constraints

- `ares-core` remains platform-neutral and WASM-compatible: no filesystem, terminal, UI, OpenGL, or native-only APIs.
- All touched Rust files remain at or below 400 LOC.
- The implementation must stay source-cited to the Orca files above and must not create an Ares-owned replacement pipeline.
