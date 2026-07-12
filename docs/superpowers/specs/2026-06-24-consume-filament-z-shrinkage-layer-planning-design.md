# Consume Filament Z Shrinkage In Layer Planning Design

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1622` declares `filament_shrinkage_compensation_z`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2584-2594` defines the option as a percent vector with default `100%` and range `50..=150`.
- `OrcaSlicer/src/libslic3r/Print.cpp:3628-3660` requires used extruders to share shrinkage settings and returns per-axis compensation as `100.0 / configured_percent`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:67-115` stores `object_height * object_shrinkage_compensation.z()` as the planned print Z max.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:825-847` uses `object_shrinkage_compensation_z` while generating object layer Z positions.

## Ares Boundary

Implement only the single-extruder Z shrinkage slice in `ares-core`:

- Add the parser and private `SliceOptions` accessor in existing `crates/ares-core/src/options/pellet.rs` because this file already owns pellet/shrinkage-adjacent filament behavior and has LOC headroom.
- Do not modify `crates/ares-core/src/options.rs`; it is already at the 400 LOC cap, and `pellet.rs` can attach an inherent `impl SliceOptions` method from inside the existing module.
- Apply the Z compensation in `crates/ares-core/src/planning.rs::plan_layers` before fixed-height layers are generated.
- Let the existing G-code pipeline consume the compensated `Layer::print_z()` values, so emitted layer-change and travel Z commands reflect the option.

Ares currently rejects different extruder setups before slicing. This slice therefore consumes the first configured filament Z shrinkage value and does not add a separate multi-extruder consistency gate.

## Included Behavior

- Missing `filament_shrinkage_compensation_z` defaults to `100%` and preserves current layer planning and G-code.
- Accepted inputs:
  - numeric scalar, interpreted as percent, for example `80`,
  - numeric string scalar, interpreted as percent, for example `"80"`,
  - percent string scalar, for example `"80%"`,
  - semicolon/comma-delimited string values, for example `"80%;100%"` or `"80,100"`,
  - non-empty arrays containing numbers, numeric strings, or percent strings, for example `[80]`, `["80"]`, and `["80%"]`.
- The first configured value is interpreted as Orca does: `z_compensation = 100.0 / configured_percent`.
- Every configured value must be finite and inside Orca's `50..=150` range. Values such as `49`, `"49%"`, `151`, and `"151%"` are rejected.
- A model with height `1.0`, `layer_height = 0.2`, `initial_layer_height = 0.2`, and `filament_shrinkage_compensation_z = 80%` plans up to `1.2` by default because the compensated object height is `1.25` and the existing midpoint stop rule stops before `1.4`.
- With `precise_z_height = true`, the same shape aligns the final planned Z to the compensated height `1.25`.
- Invalid values are rejected at the option boundary: zero, negative, below `50`, above `150`, non-finite, empty vectors, empty delimited entries, and non-numeric entries.

## Deferred Behavior

- XY `filament_shrink` geometry scaling is deferred until an upstream `Print::shrinkage_compensation` / model-transform slice can rewrite the XY transform boundary.
- Multi-extruder shrinkage consistency is deferred because this Ares slicing boundary already rejects different extruder setups before slicing.
- Variable layer height profile scaling beyond the existing fixed-height planner is deferred until Ares has an upstream-compatible variable layer height profile boundary.
- Support, raft, wipe tower, and object-specific shrinkage interactions are deferred.

## Tests

- Add focused layer-planning tests for:
  - default `100%` preserving current planned Z values,
  - `80%` changing fixed-height planned Z to the compensated object height window,
  - `80%` plus `precise_z_height` aligning the final planned Z to `1.25`,
  - accepted scalar, percent-string, delimited-string, and array forms,
  - invalid inputs returning `SliceError::InvalidInput`.
- Add a focused G-code pipeline test proving `filament_shrinkage_compensation_z = 80%` changes emitted layer-change/travel Z output.
- Include invalid coverage for below-min and above-max values.
- Run RED/GREEN with `cargo nextest run -p ares-core filament_z_shrinkage`.
- Run adjacent layer planning verification with separate nextest invocations for `precise_z_height`, `plan_layers`, and `pellet_flow_gcode`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.

## Docs Impact

No user-facing docs update is required beyond this SDD spec and implementation plan. The behavior is private `ares-core` option consumption covered by focused layer-planning and G-code tests.

## Safety

The change is platform-neutral and stays inside `ares-core`. It adds no file I/O, terminal behavior, UI behavior, OpenGL behavior, dependencies, feature flags, or compatibility shims.
