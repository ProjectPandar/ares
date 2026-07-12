# Consume `wipe_before_external_loop` Design

## Source Boundary

- Upstream option declaration: `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1186`, `((ConfigOptionBool, wipe_before_external_loop))`.
- Upstream option definition: `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5517-5526`.
- Upstream behavior: `OrcaSlicer/src/libslic3r/GCode.cpp:5756-5824` in `_extrude_loop`, where Orca inserts a no-extrusion fake external-perimeter path before extruding an external loop when `wipe_before_external_loop` is enabled and a neighboring perimeter makes the inward move safe.
- Ares metadata already cites the option in `crates/ares-core/src/options/tests/registry_lookup_wipe_speed_loop.rs`, but `crates/ares-core/src/gcode.rs` and `crates/ares-core/src/options/layer_change_retraction.rs` currently do not consume `wipe_before_external_loop`.

## Ares Destination Boundary

- Add a platform-neutral `ares-core` runtime option accessor for `wipe_before_external_loop`.
- Add a small G-code formatting helper in a new focused module, called by `crates/ares-core/src/gcode.rs` before external-perimeter print moves.
- Keep `crates/ares-core/src/gcode.rs` at or below the repository 400 LOC limit. The integration in `gcode.rs` must be only a narrow call site; helper logic and helper tests must live outside that file.
- Keep the change inside byte-oriented slicing/G-code output. Do not add file I/O, UI, OpenGL, terminal behavior, dependencies, or a new crate.

## Included Behavior

- Parse absent `wipe_before_external_loop` as `false`.
- Accept only a JSON boolean for `wipe_before_external_loop`; reject non-boolean values with `SliceError::InvalidInput` mentioning the option key.
- When enabled, insert a two-line no-extrusion wipe before the first print move of an external perimeter if the current layer has at least one internal perimeter print path. This mirrors Orca's safety gate that requires more than one perimeter before doing the inward wipe.
- The helper inputs are the current layer's `LayerPrintPaths`, the current toolpath/extrusion/speed move index, the writer's current XY position, and the current external print move's feedrate and effective line width.
- The helper must identify an eligible first external-perimeter print move by checking that the current move is `ToolpathMoveKind::Print`, the current role is `PrintPathRole::ExternalPerimeter`, and the immediately preceding move is the travel move to the same external perimeter start.
- The wipe target is computed from the current external loop start `S` toward the nearest projected point `I` on any same-layer internal-perimeter segment. If no internal segment exists, or `S` and `I` are coincident, the helper emits nothing.
- The wipe distance is `min(distance(S, I), effective_external_line_width / 2.0)`. The target point `W` is `S + normalize(I - S) * wipe_distance`.
- The inserted G-code sequence is exactly two no-extrusion moves: `S -> W`, then `W -> S`. The second return move is required so the following external-perimeter extrusion starts from the original loop start.
- Both inserted moves must use the current external-perimeter feedrate and `gcode_comments` must control whether they carry a comment.
- The inserted moves must not change E position. The following external-perimeter extrusion must still start from the original loop start XY and the same E value it would have used without this option.
- The `gcode.rs` integration point is after layer-change resume and travel-retraction unretract output have run and after the pending travel lift has been computed, but before `gcode_move_emit::move_gcode` emits the first external-perimeter print move. At that point the writer must be at the external loop start `S`; after the wipe helper returns, the writer XY must be restored to `S` and E must be unchanged so the normal external-perimeter extrusion is emitted exactly from the original start.
- Disabled `wipe_before_external_loop` must preserve existing G-code bytes for the same input.
- If there is no internal perimeter on the layer, the option must not insert a wipe move.

## Deferred Behavior

- Full Orca AABB line-neighbor search, hole/contour winding decisions, exact angle rotation, and Orca's `min(nozzle_diameter, path.width) / 2` distance rule are deferred until Ares ports the richer perimeter loop geometry from `libslic3r`.
- Full `wipe_on_loops` behavior is out of scope.
- Scarf seam behavior, seam slope behavior, arc fitting, and complete `ExtrusionLoop` parity are out of scope.
- This slice does not alter travel retraction wipe behavior in `crates/ares-core/src/gcode_travel_retraction.rs`.

## Acceptance Criteria

- A focused RED nextest run fails before implementation because enabled `wipe_before_external_loop` does not add the pre-external-loop no-extrusion move.
- After implementation, focused nextest passes for:
  - enabled external-loop wipe with an internal perimeter on the same layer, proving the two no-extrusion moves appear before the first external extrusion and the external extrusion still starts from the original loop start,
  - disabled byte preservation for the same role sequence,
  - enabled option with no internal perimeter preserving output,
  - invalid non-boolean option values returning an error that contains `wipe_before_external_loop`.
- Existing travel retraction wipe tests still pass.
- OrcaSlicer reference verification is required for this runtime G-code behavior slice when a local OrcaSlicer CLI/binary and a matching comparison fixture/profile are available. The reference case must enable `wipe_before_external_loop` with at least two wall loops and confirm Orca emits an inward no-extrusion move before external-loop extrusion only when a neighboring internal perimeter exists. If this workspace lacks that executable harness, the implementation report must record the exact missing binary/harness as `Not-tested`, cite `OrcaSlicer/src/libslic3r/GCode.cpp:5756-5824`, and must not claim full Orca binary parity.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run -p ares-core wipe_before_external_loop`
  - `cargo nextest run -p ares-core travel_retraction_gcode::wipe`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, with every touched Rust file at or below 400 LOC.

## Documentation Impact

- No user-facing CLI or API documentation is required for this narrow runtime option slice.
- The implementation plan and final commit message must keep the slice source-cited and state that richer Orca perimeter-loop parity remains deferred.
