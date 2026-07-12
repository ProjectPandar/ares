# Consume Adaptive Bridge Pressure Advance Design

## Goal

Consume OrcaSlicer's existing `adaptive_pressure_advance_bridges` option as concrete Ares G-code behavior. When pressure advance and adaptive pressure advance are both enabled, bridge-like print moves should temporarily use the configured bridge pressure advance value instead of leaving the option as metadata only.

## Upstream rewrite boundary

This slice ports the bridge override path from these upstream OrcaSlicer sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1302-1308`
  - `enable_pressure_advance`
  - `pressure_advance`
  - `adaptive_pressure_advance`
  - `adaptive_pressure_advance_overhangs`
  - `adaptive_pressure_advance_model`
  - `adaptive_pressure_advance_bridges`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2252-2319`
  - default `enable_pressure_advance = false`
  - default `pressure_advance = 0.02`
  - default `adaptive_pressure_advance = false`
  - default `adaptive_pressure_advance_bridges = 0.0`
  - max pressure advance and bridge pressure advance value `2`
- `OrcaSlicer/src/libslic3r/GCode.cpp:6657-6770`
  - adaptive PA is evaluated only when `adaptive_pressure_advance` and `enable_pressure_advance` are both enabled
  - PA change tags mark bridge-like paths using `BR` for `erBridgeInfill` and `erOverhangPerimeter`
- `OrcaSlicer/src/libslic3r/GCode/AdaptivePAProcessor.cpp:221-272`
  - failed or disabled adaptive PA falls back to `pressure_advance`
  - when `isBridge` and `adaptive_pressure_advance_bridges > EPSILON`, the predicted PA is overridden by `adaptive_pressure_advance_bridges`
  - a pressure advance command is emitted when the predicted PA changes

The Rust destination boundary is:

- `crates/ares-core/src/options/pressure_advance.rs`
- `crates/ares-core/src/gcode_pressure_advance.rs`
- `crates/ares-core/src/gcode_move_emit.rs`
- the minimal wiring needed in `crates/ares-core/src/gcode.rs`
- tests in `crates/ares-core/src/tests/pressure_advance_gcode.rs`

`crates/ares-core/src/gcode.rs` is already near the 400 LOC limit, so its implementation constraint is no net LOC increase for this slice. The pressure-advance state and decision logic must live in `gcode_pressure_advance.rs`; `gcode.rs` may only create/pass that state with compacted existing formatting. If no-net wiring is not possible, split an existing G-code formatting responsibility into a focused helper module before adding the behavior, then keep all touched Rust files at or below 400 LOC.

## Included behavior

- Parse `adaptive_pressure_advance` from a boolean or boolean list, using the first value and defaulting to `false`.
- Parse `adaptive_pressure_advance_bridges` from the same numeric scalar/list/string forms used by `pressure_advance`, using the first value and defaulting to `0.0`.
- Reject non-finite or out-of-range `adaptive_pressure_advance_bridges` values outside `0..=2`.
- Keep the existing startup pressure advance behavior unchanged:
  - if `enable_pressure_advance` is false, emit no startup pressure advance command;
  - if `enable_pressure_advance` is true, emit the base `pressure_advance` startup command.
- Enable runtime bridge PA overrides only when:
  - `enable_pressure_advance` is true;
  - `adaptive_pressure_advance` is true;
  - `adaptive_pressure_advance_bridges > 0`.
- Treat Ares `PrintPathRole::Bridge`, `PrintPathRole::InternalBridge`, and `PrintPathRole::OverhangPerimeter` as the bridge-like roles for this slice. This maps Ares' current role vocabulary to Orca's bridge PA boundary, where upstream flags `erBridgeInfill` and `erOverhangPerimeter` for bridge override processing.
- Emit the bridge pressure advance command before the first eligible bridge-like print move after a non-bridge PA state.
- Emit the base pressure advance command before the first non-bridge print move after a bridge PA state.
- Avoid duplicate pressure advance commands while consecutive eligible bridge-like moves remain in the same PA state.
- Ignore travel moves for bridge PA state changes.
- Keep the implementation inside `ares-core` platform-neutral and WASM-compatible.
- Keep `gcode.rs` at or below 400 LOC by making the PA state a focused helper in `gcode_pressure_advance.rs` and keeping any `gcode.rs` call-site wiring line-neutral.

## Deferred behavior

This slice intentionally does not port the full Orca adaptive PA system:

- no `adaptive_pressure_advance_model` parsing or interpolation;
- no `adaptive_pressure_advance_overhangs` speed/flow-triggered recalculation beyond the bridge-like role boundary above;
- no `;PA_Change` reserved tag post-processing pipeline;
- no multi-extruder or toolchange PA model state;
- no calibration-mode behavior;
- no debug `APA` comments.

Those behaviors must remain future source-cited rewrites of `GCode.cpp` and `GCode/AdaptivePAProcessor.cpp`, not Ares-owned pipeline design.

## Acceptance criteria

- With default options, generated G-code still contains no pressure advance command.
- With `enable_pressure_advance = true`, existing startup PA output remains unchanged.
- With `enable_pressure_advance = true`, `adaptive_pressure_advance = true`, and `adaptive_pressure_advance_bridges > 0`, a bridge-like print path emits:
  - the startup base PA command;
  - a later bridge PA command before the bridge print move.
- With the same settings, a non-bridge print path emits only the startup base PA command.
- When print moves transition from bridge-like to non-bridge, the next non-bridge print move restores the base PA command before the move.
- When `adaptive_pressure_advance_bridges = 0`, no bridge-specific PA command is emitted.
- Invalid `adaptive_pressure_advance` and `adaptive_pressure_advance_bridges` inputs fail with `SliceError::InvalidInput` mentioning the relevant option key.
- The touched Rust files remain at or below 400 LOC.

## Test plan

- Add failing `cargo nextest run -p ares-core adaptive_bridge_pressure_advance` tests before implementation.
- Use `crates/ares-core/src/pipeline/test_support.rs::single_path_pipeline` for bridge and non-bridge integration checks where possible.
- Add a focused state-level test for bridge-to-non-bridge restoration if a single pipeline fixture cannot represent mixed roles without broad test helper refactoring.
- Verify with:
  - `cargo fmt --check`
  - targeted `cargo nextest run -p ares-core adaptive_bridge_pressure_advance pressure_advance_gcode`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `wc -l` on every touched Rust file

## Documentation impact

Update `docs/roadmap.md` after implementation review to record this as a concrete consumed runtime slice and list the deferred full adaptive PA model work.
