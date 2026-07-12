# Consume Internal Solid Infill Numeric Options Design

## Goal

Consume OrcaSlicer's concrete internal solid infill numeric behavior for the `PrintPathRole::SolidInfill` path that Ares now emits when `sparse_infill_density == 100`. The current Ares compatibility shell still reuses sparse infill width, flow ratio, speed, and acceleration for solid infill. This slice replaces that shell with the already registered `internal_solid_infill_line_width`, `internal_solid_infill_flow_ratio`, `internal_solid_infill_speed`, and `internal_solid_infill_acceleration` runtime behavior.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1049-1050` declares `sparse_infill_acceleration` followed by `internal_solid_infill_acceleration`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1170-1171` declares `internal_solid_infill_line_width` and `internal_solid_infill_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1227-1228` declares `sparse_infill_flow_ratio` followed by `internal_solid_infill_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1415-1434` registers `sparse_infill_flow_ratio` and `internal_solid_infill_flow_ratio`, both defaulting to `1` with range `[0, 2]`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3197-3214` registers `sparse_infill_acceleration` and `internal_solid_infill_acceleration` as `FloatOrPercent` over `default_acceleration`, both defaulting to `100%`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5754-5764` registers `internal_solid_infill_line_width` as `FloatOrPercent` over `nozzle_diameter`, min `0`, max `1000`, default `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5766-5775` registers `internal_solid_infill_speed` as a positive `Float` in mm/s, default `100`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6434-6437` chooses `sparse_infill_acceleration` for `erInternalInfill` and `internal_solid_infill_acceleration` for `erSolidInfill`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6498-6504` applies `sparse_infill_flow_ratio` for `erInternalInfill` and `internal_solid_infill_flow_ratio` for `erSolidInfill` when `set_other_flow_ratios` is enabled.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6540-6544` chooses `sparse_infill_speed` for `erInternalInfill` and `internal_solid_infill_speed` for `erSolidInfill`.

## Ares Boundary

- In `crates/ares-core/src/options/flow_ratios.rs`, parse `internal_solid_infill_line_width` and thread it into `ExtrusionOptions` as the role-specific width for `PrintPathRole::SolidInfill`. The default `0` keeps fallback behavior through `line_width`, matching the existing Ares width fallback convention.
- In `crates/ares-core/src/options/flow_ratios.rs`, parse `internal_solid_infill_flow_ratio` with the same `[0, 2]` range as other Orca flow-ratio options. It must be validated even when `set_other_flow_ratios` is false, but only applied when that gate is true.
- In `crates/ares-core/src/extrusions.rs`, use `internal_solid_infill_line_width` for `PrintPathRole::SolidInfill` width and `internal_solid_infill_flow_ratio` for its role flow. Keep first-layer line-width and first-layer flow-ratio behavior shared with non-brim/skirt print roles.
- In `crates/ares-core/src/options.rs` / speed parsing, parse `internal_solid_infill_speed` and thread it into `SpeedOptions` as the steady-state speed for `PrintPathRole::SolidInfill`. Sparse infill continues to use `sparse_infill_speed`.
- In `crates/ares-core/src/options/acceleration.rs`, parse `internal_solid_infill_acceleration` as non-negative numeric or percent over `default_acceleration`, defaulting to `default_acceleration`.
- In `crates/ares-core/src/speeds/kinematics.rs`, use `internal_solid_infill_acceleration` for `PrintPathRole::SolidInfill` on non-first-layer print moves. Sparse infill continues to use `sparse_infill_acceleration`.
- In `crates/ares-core/src/speeds/config.rs` and `crates/ares-core/src/speeds/slow_down_layers.rs`, keep first-layer solid infill speed using `initial_layer_infill_speed`, matching the current Ares first-layer surface limitation. The internal-solid steady-state speed applies when `is_first_layer == false`.
- Keep `PrintPathRole::SolidInfill` volumetric speed caps, slowdown mechanics, jerk, fan behavior, role strings, and G-code comment behavior unchanged except that their input speed/extrusion values now come from internal-solid numeric options.

## Included Behavior

- `PrintPathRole::SparseInfill` keeps using `sparse_infill_line_width`, `sparse_infill_flow_ratio`, `sparse_infill_speed`, and `sparse_infill_acceleration`.
- `PrintPathRole::SolidInfill` uses `internal_solid_infill_line_width` when it is greater than zero; otherwise it falls back through `line_width` / auto width as Ares already does for zero role widths.
- `PrintPathRole::SolidInfill` uses `internal_solid_infill_flow_ratio` only when `set_other_flow_ratios == true`; when the gate is false, the option is validated but does not scale extrusion.
- `PrintPathRole::SolidInfill` uses `internal_solid_infill_speed` on non-first-layer print moves. First-layer solid infill keeps `initial_layer_infill_speed`.
- `PrintPathRole::SolidInfill` uses `internal_solid_infill_acceleration` on non-first-layer print moves when acceleration output is enabled through `default_acceleration > 0`. First-layer acceleration keeps `initial_layer_acceleration` precedence.
- `internal_solid_infill_speed` rejects zero, negative, non-number, and non-finite values through `SliceOptions::speed_options()`.
- `internal_solid_infill_acceleration` accepts absolute non-negative numbers and percentages over `default_acceleration`, and rejects invalid values with a `SliceError::InvalidInput` mentioning the key.
- `internal_solid_infill_line_width` follows the existing Ares `FloatOrPercent` width parser and key-specific validation errors.
- Pipeline/G-code tests prove that changing internal-solid width/flow/speed/acceleration changes only the solid-infill path behavior while leaving sparse-infill behavior under its sparse options.

## Deferred Behavior

- Top solid infill, bottom surface, ironing, gap fill, support, and multi-surface `SurfaceCollection` role classification.
- `top_solid_infill_flow_ratio`, `bottom_solid_infill_flow_ratio`, `top_surface_line_width`, `gap_fill_flow_ratio`, support flow ratios, and top/bottom surface speeds.
- Any new option metadata or registry additions; the target options are already registered.
- Orca's complete print-region/object override hierarchy, preset UI behavior, object-specific config merging, and filament/extruder routing.
- Full Orca E2E comparison. This slice is still verified inside the existing Ares deterministic geometry/G-code scaffold.

## Acceptance Criteria

1. Options tests prove `internal_solid_infill_line_width` reaches `ExtrusionOptions::width_for_role(PrintPathRole::SolidInfill)` and does not change `SparseInfill` width.
2. Options tests prove `internal_solid_infill_flow_ratio` is validated with `[0, 2]`, is ignored when `set_other_flow_ratios` is false, and scales only `SolidInfill` when the gate is true.
3. Options tests prove `internal_solid_infill_speed` reaches `SpeedOptions::speed_for_role(Print, SolidInfill)` and does not change `SparseInfill` speed.
4. Options tests prove `internal_solid_infill_acceleration` reaches `AccelerationOptions` / `SpeedOptions::acceleration_for_layer(Print, SolidInfill, false)` and does not change sparse infill acceleration.
5. Extrusion unit tests prove `SolidInfill` no longer reuses sparse infill width or flow when internal-solid-specific values are configured, while first-layer line width and first-layer flow ratio still apply to solid infill.
6. Speed unit tests prove `SolidInfill` no longer reuses sparse infill speed or acceleration when internal-solid-specific values are configured, while first-layer speed/acceleration precedence and infill jerk behavior stay unchanged.
7. Pipeline/G-code tests prove `sparse_infill_density == 100` plus internal-solid width/flow/speed/acceleration changes the emitted `solid_infill` extrusion deltas/feedrates/acceleration commands independently of sparse options.
8. Existing tests for sparse infill flow, sparse infill speed, sparse infill acceleration, 100% internal-solid role identity, volumetric caps, slowdown, role fans, and G-code comments continue to pass.
9. All touched Rust files under `crates/` remain at or below 400 LOC.

## Verification

- Targeted RED/GREEN tests for options parsing and role-specific extrusion/speed behavior.
- `cargo test -p ares-core --lib internal_solid_numeric`
- `cargo test -p ares-core --lib solid_infill`
- `cargo test -p ares-core --lib sparse_infill_flow_ratio`
- `cargo test -p ares-core --lib internal_solid_infill`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- Rust LOC gate: `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and the implementation plan are the documentation artifacts for the slice. No CLI or WASM docs are needed because this change consumes existing Orca option keys inside `ares-core` and does not change public adapter APIs.
