# Consume Acceleration G-code Design

## Goal

Port the OrcaSlicer print/travel acceleration option behavior into Ares G-code output so the existing registered acceleration options emit concrete `M204` commands instead of remaining registry metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1042-1049` declares `default_acceleration`, `outer_wall_acceleration`, `inner_wall_acceleration`, `initial_layer_acceleration`, `travel_acceleration`, and `sparse_infill_acceleration`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1779-1786` registers `default_acceleration` as a non-negative float with default `500`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3068-3085` registers `inner_wall_acceleration` and `travel_acceleration` as non-negative floats with default `10000`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3095-3102` registers `outer_wall_acceleration` as a non-negative float with default `500`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3114-3122` registers `sparse_infill_acceleration` as a non-negative float-or-percent over `default_acceleration`, default `100%`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3134-3141` registers `initial_layer_acceleration` as a non-negative float with default `300`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6345-6367` chooses print-path acceleration from first-layer, bridge, sparse infill, outer wall, inner wall, and default acceleration before emitting the path.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7313-7344` chooses travel acceleration from outer-wall short-travel or travel acceleration before emitting travel acceleration.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:216-249` emits acceleration with `M204`/firmware-specific commands, suppressing zero and unchanged values.

## Ares Boundary

Implement the runtime slice in `crates/ares-core` only:

- Add an `AccelerationOptions` data model in the existing speed/move layer rather than creating a new crate.
- Parse these existing registry keys through `SliceOptions`:
  - `default_acceleration`: non-negative float, default `500`.
  - `initial_layer_acceleration`: non-negative float, default `300`.
  - `outer_wall_acceleration`: non-negative float, default `500`.
  - `inner_wall_acceleration`: non-negative float, default `10000`.
  - `travel_acceleration`: non-negative float, default `10000`.
  - `sparse_infill_acceleration`: non-negative numeric-or-percent value over `default_acceleration`, default `100%`.
- Assign an acceleration to each emitted Ares move using the source-cited Orca precedence that Ares can currently represent:
  - If `default_acceleration` is `0`, acceleration output is disabled.
  - First-layer print moves use `initial_layer_acceleration` when it is greater than `0`.
  - Sparse infill print moves use `sparse_infill_acceleration` when it is greater than `0`.
  - External perimeter print moves use `outer_wall_acceleration` when it is greater than `0`.
  - Internal perimeter print moves use `inner_wall_acceleration` when it is greater than `0`.
  - Other print moves fall back to `default_acceleration`.
  - Travel moves use `travel_acceleration` when it is greater than `0`; when `travel_acceleration` is `0`, no travel acceleration command is emitted for that travel move.
- Emit acceleration at the G-code command boundary before the move that uses it.
- For this slice, use the default Ares firmware-neutral command form `M204 S<rounded acceleration>`, matching Orca's fallback writer branch when no firmware flavor-specific writer is modeled.
- Suppress acceleration commands when the rounded acceleration is `0` or unchanged from the last emitted acceleration command.
- Preserve existing speed/feedrate, XY, Z, E, path ordering, extrusion, layer planning, and diagnostic comments except where an acceptance criterion explicitly expects an inserted `M204` command.
- Keep `gcode_comments` composition: when enabled, acceleration commands include the existing style of inline command comment text, `; adjust acceleration`.

Because `crates/ares-core/src/gcode.rs` is already at the repository 400 LOC limit, this slice must include a narrow `gcode.rs` helper split or equivalent local deletion directly serving acceleration command emission. The split must not change behavior by itself.

## Out Of Scope

- No acceleration option registry additions.
- No jerk output.
- No firmware flavor parsing or flavor-specific `M204 P`, `M204 T`, `M201`, `M202`, or `SET_VELOCITY_LIMIT` behavior.
- No machine acceleration limits or clamping.
- No Klipper acceleration-and-jerk command merging.
- No bridge acceleration, internal solid infill acceleration, top-surface acceleration, gap-fill acceleration, or overhang acceleration until the corresponding Ares roles exist.
- No short-travel outer-wall special case, because Ares does not yet track retraction-minimum-travel context for the source-cited Orca branch.
- No changes to speed/feedrate selection.
- No Ares-owned pipeline redesign.

## Acceptance Criteria

- With default acceleration options, emitted path-following G-code includes this source-aligned sequence when those roles are present:
  - `M204 S10000` before the first travel move because Orca's `travel_acceleration` default is `10000`.
  - `M204 S300` before the first first-layer print move because Orca's `initial_layer_acceleration` default is `300`.
  - `M204 S500` before the first later-layer print move that resolves to the default/sparse-infill acceleration because Orca's `default_acceleration` default is `500` and `sparse_infill_acceleration` defaults to `100%` over it.
- `default_acceleration: 0` disables all acceleration commands while preserving existing movement command counts and feedrates.
- `initial_layer_acceleration` changes first-layer print acceleration and does not change second-layer print acceleration.
- `outer_wall_acceleration` changes external perimeter print acceleration.
- `inner_wall_acceleration` changes internal perimeter print acceleration when internal perimeter paths exist.
- `sparse_infill_acceleration` accepts both numeric values and percent strings over `default_acceleration`, and changes sparse infill print acceleration.
- `travel_acceleration` changes travel acceleration.
- `travel_acceleration: 0` suppresses travel acceleration commands without causing travel moves to inherit `default_acceleration`.
- Unchanged consecutive acceleration values are not re-emitted.
- Acceleration values are rounded to the nearest integer before `M204 S...` emission.
- Invalid acceleration values that are negative, non-numeric, non-finite, or invalid percent strings are rejected through the slicing/G-code formatting path.
- `gcode_comments: true` appends `; adjust acceleration` to emitted `M204 S...` acceleration commands without changing movement coordinates, feedrates, or E values.
- All touched Rust source files remain at or below 400 LOC.
- Verification must include focused red/green tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate.
- OrcaSlicer E2E verification is required if an executable OrcaSlicer CLI/binary plus a matching local comparison fixture is available in this workspace. If no such harness is present, the implementation report must record the exact missing harness or command as `Not-tested`; this slice still uses source-cited Orca line references plus focused Ares G-code assertions and must not claim full Orca binary parity.
