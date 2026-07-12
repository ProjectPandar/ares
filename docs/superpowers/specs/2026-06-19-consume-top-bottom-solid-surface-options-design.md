# Consume Top And Bottom Solid Surface Options Design

## Goal

Consume OrcaSlicer's concrete top and bottom solid-surface behavior in the Ares slicing pipeline instead of treating all generated 100% solid infill as the generic `PrintPathRole::SolidInfill`. This slice introduces explicit top and bottom solid print-path roles for the existing deterministic multi-layer infill scaffold, then wires already registered top/bottom surface options into extrusion, speed, acceleration, jerk, print-domain roles, and emitted G-code.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1045` declares `top_surface_acceleration`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1056` declares `top_surface_jerk`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1090-1091` declares `top_surface_pattern` and `bottom_surface_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1166` declares `top_surface_line_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1169` declares `top_surface_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1193-1194` declares `top_solid_infill_flow_ratio` and `bottom_solid_infill_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1286-1305` registers top and bottom solid infill flow ratios as `Float` values with range `[0, 2]`, default `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1986-2015` registers top and bottom surface pattern options. Pattern geometry remains deferred in this slice.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3086-3093` registers `top_surface_acceleration`, default `500`, min `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3206-3213` registers `top_surface_jerk`, default `9`, min `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6543-6553` registers `top_surface_line_width` as `FloatOrPercent` over `nozzle_diameter`, min `0`, default `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6555-6562` registers `top_surface_speed` as a positive `Float` in mm/s, default `100`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:20-35` gives `frTopSolidInfill` an automatic width of one nozzle diameter, while solid/internal infill keeps the 1.125x nozzle default.
- `OrcaSlicer/src/libslic3r/Flow.cpp:40-53` maps `top_surface_line_width` to `frTopSolidInfill`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6363-6364` applies `top_surface_acceleration` to top-surface roles.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6379-6380` applies `top_surface_jerk` to top-surface roles.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6403-6406` applies `top_solid_infill_flow_ratio` to `erTopSolidInfill` and `bottom_solid_infill_flow_ratio` to `erBottomSurface`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6466-6471` applies `top_surface_speed` to `erTopSolidInfill` and `initial_layer_infill_speed` to `erBottomSurface`.

## Ares Boundary

- In `crates/ares-core/src/print_paths.rs`, add `PrintPathRole::TopSolidInfill` and `PrintPathRole::BottomSurface`. Map generated solid infill paths to `BottomSurface` on layer `0`, `TopSolidInfill` on the last layer in the current layer set, and `SolidInfill` on interior layers. Layer `0` has precedence when the current layer set contains only one generated layer, so a single-layer 100% solid-infill scaffold emits `BottomSurface` and no `TopSolidInfill`.
- In `crates/ares-core/src/extrusion_entity.rs` and `crates/ares-core/src/print.rs`, map the new print roles to existing `ExtrusionRole::TopSolidInfill` and `ExtrusionRole::BottomSurface`.
- In `crates/ares-core/src/extrusions/options.rs` and `crates/ares-core/src/options/flow_ratios.rs`, parse and use `top_surface_line_width`, `top_solid_infill_flow_ratio`, and `bottom_solid_infill_flow_ratio`. Top-surface width defaults to nozzle-diameter auto width when both `top_surface_line_width` and `line_width` are zero. Top and bottom flow ratios are always role-specific, matching Orca's GCode branch; they do not depend on `set_other_flow_ratios`. `BottomSurface` composes with existing first-layer extrusion behavior: `initial_layer_line_width` still supplies the first-layer infill width when configured, `first_layer_flow_ratio` still applies on layer `0`, and `bottom_solid_infill_flow_ratio` additionally scales only the bottom-surface role.
- In `crates/ares-core/src/options/speed.rs`, parse `top_surface_speed` and thread it to `SpeedOptions`.
- In `crates/ares-core/src/options/acceleration.rs` and `crates/ares-core/src/speeds/kinematics.rs`, parse and use `top_surface_acceleration` and `top_surface_jerk` for `TopSolidInfill` on non-first-layer print moves. First-layer acceleration and jerk precedence remains unchanged.
- In `crates/ares-core/src/speeds/config.rs`, use `top_surface_speed` for `TopSolidInfill`. `BottomSurface` keeps `initial_layer_infill_speed`, matching Orca's `erBottomSurface` speed branch. Keep this file under the 400 LOC project limit by moving role-specific speed tests into focused test modules instead of expanding production comments or table scaffolding in this file.
- Keep sparse infill, internal solid infill, first-layer flow, volumetric caps, slowdown, role fan behavior, role-change custom G-code, line comments, and adapter APIs unchanged except for the new role strings in generated artifacts and G-code comments.

## Included Behavior

- For generated solid infill from `sparse_infill_density == 100`:
  - Layer `0` print paths are `bottom_surface`.
  - The highest generated layer in the current pipeline is `top_solid_infill`.
  - Interior layers remain `solid_infill`.
  - In a single-layer pipeline, layer `0` precedence wins and the path is `bottom_surface`.
- `top_surface_line_width` changes only `TopSolidInfill` extrusion width. Zero `top_surface_line_width` uses `line_width` when positive, otherwise one nozzle diameter.
- `top_solid_infill_flow_ratio` scales only `TopSolidInfill`.
- `bottom_solid_infill_flow_ratio` scales only `BottomSurface` and composes with first-layer flow ratio on layer `0`.
- `BottomSurface` uses the same first-layer width precedence as previous first-layer solid infill: `initial_layer_line_width` when configured, otherwise the role/default infill width.
- `top_surface_speed` changes only non-first-layer top solid surface feedrates.
- `TopSolidInfill` uses `top_surface_acceleration` and `top_surface_jerk` on non-first-layer print moves when acceleration/jerk output is enabled.
- `BottomSurface` keeps first-layer speed, acceleration, and jerk precedence.
- Invalid top/bottom flow ratios, top surface speed, top surface acceleration, top surface jerk, and top surface line width return `SliceError::InvalidInput` mentioning the offending key.

## Deferred Behavior

- Full top/bottom surface geometric classification from shell layers, `top_shell_layers`, `bottom_shell_layers`, top/bottom shell thickness, sparse-infill-adjacent solid surfaces, and multi-surface `SurfaceCollection` partitioning.
- `top_surface_pattern`, `bottom_surface_pattern`, `top_surface_density`, `bottom_surface_density`, `min_width_top_surface`, ironing, bridge-specific bottom surfaces, support surfaces, and gap-fill geometry.
- `gap_fill_flow_ratio` and `gap_infill_speed`, because Ares does not yet emit gap-fill print paths.
- Any new option metadata or registry additions; this slice consumes already registered option definitions.
- Full OrcaSlicer E2E comparison. This slice is verified inside the existing deterministic Ares multi-layer scaffold.

## Acceptance Criteria

1. Print-path tests prove solid infill paths are mapped to `BottomSurface` on first layer, `TopSolidInfill` on last layer, and `SolidInfill` on interior layers, while sparse infill stays `SparseInfill`.
2. Print-path tests prove a single-layer 100% solid-infill scaffold maps that layer to `BottomSurface` and does not emit `TopSolidInfill`.
3. Print-domain tests prove the new print roles map to `ExtrusionRole::BottomSurface` and `ExtrusionRole::TopSolidInfill`.
4. Options and extrusion tests prove `top_surface_line_width` reaches only `TopSolidInfill`, and zero top width falls back to nozzle-diameter auto width when `line_width == 0`.
5. Options and extrusion tests prove `BottomSurface` still uses `initial_layer_line_width` when configured and composes `first_layer_flow_ratio` with `bottom_solid_infill_flow_ratio`.
6. Options and extrusion tests prove `top_solid_infill_flow_ratio` and `bottom_solid_infill_flow_ratio` validate `[0, 2]`, apply regardless of `set_other_flow_ratios`, and scale only their target roles.
7. Options and speed tests prove `top_surface_speed`, `top_surface_acceleration`, and `top_surface_jerk` reach `TopSolidInfill` without changing `SolidInfill`, `BottomSurface`, or `SparseInfill` behavior.
8. Pipeline/G-code tests prove multi-layer `sparse_infill_density == 100` emits `bottom_surface`, `solid_infill`, and `top_solid_infill` comments/role strings and that top/bottom numeric options change emitted extrusion deltas/feedrates/acceleration/jerk commands independently.
9. Pipeline/G-code tests prove single-layer `sparse_infill_density == 100` emits `bottom_surface` role strings and no `top_solid_infill` role strings.
10. Existing internal-solid numeric tests continue to pass, proving the previous internal-solid slice remains intact for interior solid infill.
11. All touched Rust files under `crates/` remain at or below 400 LOC.

## LOC-Safe Implementation Guidance

- Do not add more code to `crates/ares-core/src/options/tests.rs`, which is already at the 400 LOC limit. Move the existing `internal_solid_numeric` test module to `crates/ares-core/src/options/tests/internal_solid_numeric.rs`, create `crates/ares-core/src/options/tests/top_bottom_solid_surface.rs`, and keep `crates/ares-core/src/options/tests.rs` as a module harness.
- Keep `crates/ares-core/src/speeds/config.rs` under 400 LOC. If adding top-surface speed fields pushes it near the limit, move any expanded role tests into `crates/ares-core/src/speeds/tests/top_bottom_solid_surface.rs` and keep production changes compact.
- Add pipeline coverage in `crates/ares-core/src/pipeline/tests/top_bottom_solid_surface.rs` instead of expanding `crates/ares-core/src/pipeline/tests.rs`.
- Add extrusion coverage in a new `crates/ares-core/src/extrusions/tests/top_bottom_solid_surface.rs` module instead of growing `solid_infill.rs`.
- After each implementation phase that touches Rust files, run the LOC gate:
  - `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## Verification

- RED/GREEN targeted tests:
  - `cargo test -p ares-core --lib top_bottom_solid_surface`
  - `cargo test -p ares-core --lib print_paths::tests`
  - `cargo test -p ares-core --lib extrusions::tests`
  - `cargo test -p ares-core --lib speeds::tests`
  - `cargo test -p ares-core --lib pipeline::tests::top_bottom_solid_surface`
- Regression tests:
  - `cargo test -p ares-core --lib internal_solid_numeric`
  - `cargo test -p ares-core --lib internal_solid_infill`
- Full verification:
  - `cargo test -p ares-core --lib`
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with this spec, the reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and its implementation plan are the documentation artifacts for the slice. No CLI or WASM docs are required because the public byte-in/options-to-byte-output API remains unchanged.
