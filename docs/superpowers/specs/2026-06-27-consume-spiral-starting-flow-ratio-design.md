# Consume Spiral Starting Flow Ratio

## Context

Ares already normalizes Orca `spiral_mode` into a single-wall, no-sparse-infill vase-style print boundary, but the adjacent Orca flow-ratio option is still metadata-only.

Upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1564` declares `spiral_starting_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5706-5715` defines `spiral_starting_flow_ratio` as a float in the inclusive range `0..=1`, defaulting to `0`.
- `OrcaSlicer/src/libslic3r/GCode/SpiralVase.cpp:118-151` applies the option only when `use_relative_e_distances` is enabled, scaling extrusion on the transition-in layer from `spiral_starting_flow_ratio` toward `1.0` by layer path progress.

Ares destination boundary:

- `crates/ares-core`, platform-neutral Rust only.
- Runtime parsing belongs in a small option/runtime helper, not in generated PrintConfig metadata files.
- G-code emission should consume the parsed value while formatting print moves; `crates/ares-core/src/gcode.rs` is already near the 400 LOC limit, so new behavior should live in a focused module with minimal call-site wiring.

## Decision

Implement the transition-in portion of Orca `SpiralVase::process_layer` for the current Ares vase-mode boundary:

- Parse `spiral_starting_flow_ratio` as a finite float in `0..=1`, default `0`.
- Enable the runtime behavior only when:
  - `spiral_mode` is true after current option normalization,
  - `use_relative_e_distances` is true,
  - the current layer is the first vase body layer after the normalized bottom/base layers.
- The transition layer is exactly `layer_index == normalized_bottom_shell_layers`, where `normalized_bottom_shell_layers` is read from the already-normalized `bottom_shell_layers` option value that Ares stores on the pipeline options. If that index is outside the layer list, or that layer has no printable XY length, no taper is emitted.
- For print moves on that transition layer, compute progress from cumulative printed XY length over total printed XY length for that layer. Match upstream ordering by adding the current move's XY length first, then computing `progress = cumulative_after_move / total_printed_xy_length`; scale that move's E delta by:
  - `spiral_starting_flow_ratio + progress * (1.0 - spiral_starting_flow_ratio)`.
- Keep later E moves continuous by carrying the scaled E offset forward instead of letting subsequent moves compensate back to the unscaled total.

This is an execution slice of the cited Orca transition-in behavior. It intentionally does not implement full Orca `SpiralVase` post-processing.

## Deferred

- `spiral_finishing_flow_ratio` is deferred because Orca handles it by appending a transition-out duplicate loop, which should be sliced with explicit duplicated-path G-code semantics.
- `spiral_mode_smooth` and `spiral_mode_max_xy_smoothing` are deferred because they require the previous-layer point lookup/interpolation behavior from `GCode/SpiralVase.cpp`.
- Z ramping inside the layer is deferred to a later full spiral-vase G-code post-processing slice. This slice only consumes extrusion tapering into the existing Ares vase-mode G-code output.
- Absolute E tapering is deferred, matching the upstream comment that tapering is switched off for absolute extruder distances.

## Acceptance Criteria

- A new G-code runtime test fails before implementation and passes after implementation, proving that `spiral_starting_flow_ratio` changes emitted relative E values on the first vase body layer.
- The test proves no catch-up extrusion is emitted on the following move after a scaled move.
- `spiral_starting_flow_ratio` is ignored when `use_relative_e_distances` is false.
- Invalid `spiral_starting_flow_ratio` values outside `0..=1` or non-finite values return `SliceError::InvalidInput`.
- No file edited by this slice exceeds 400 LOC.
- Verification uses `cargo nextest run`, not `cargo test`, and includes:
  - `cargo fmt --check`
  - targeted `cargo nextest run -p ares-core <spiral starting flow tests>`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`

## Docs Impact

Update `docs/roadmap.md` with this source-cited runtime slice after the implementation reviewer approves.
