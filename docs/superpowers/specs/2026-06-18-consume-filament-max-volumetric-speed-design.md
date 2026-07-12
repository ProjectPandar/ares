# Consume Filament Max Volumetric Speed Design

## Goal

Port the concrete OrcaSlicer `filament_max_volumetric_speed` print-speed cap into Ares' current single-filament G-code pipeline. This slice must make the existing option affect emitted print feedrates, not add more inert option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1333` declares `filament_max_volumetric_speed` as a `ConfigOptionFloats` G-code config field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2462-2470` defines "Max volumetric speed", units `mm3/s`, minimum `0`, and default `{ 2. }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6484-6492` uses the filament max volumetric speed to derive a print speed when a path speed is zero.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6544-6547` caps every print path speed with `filament_max_volumetric_speed / _mm3_per_mm` when the configured cap is greater than zero.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6560-6564` re-applies the same cap after resonance avoidance.

This Ares slice implements only the cap behavior that has an existing Ares destination: current generated print moves and their emitted G-code feedrates.

## Current Ares Boundary

- `crates/ares-core/src/extrusions.rs` already calculates cumulative `E` positions per print move from line geometry, layer height, filament diameter, `filament_flow_ratio`, `print_flow_ratio`, and role flow ratios.
- `crates/ares-core/src/speeds.rs` assigns per-move speeds and feedrates from `SpeedOptions`.
- `crates/ares-core/src/gcode_move_emit.rs` emits the selected feedrate in `;SPEED:*` comments and G-code `F` values.
- `crates/ares-core/src/options.rs` parses `SpeedOptions` and `filament_diameter`.

## Design

Add `filament_max_volumetric_speed` to `SpeedOptions` as a non-negative `mm3/s` scalar for the active first filament. Also store the active first filament diameter in `SpeedOptions` so `generate_speed_moves(layers, options)` keeps its current call shape while having the filament area needed for the cap. `SliceOptions::speed_options()` must populate that diameter from `filament_diameters()?[0]`.

Missing `filament_max_volumetric_speed` values default to Orca's `2.0`; zero disables the cap because the upstream runtime cap branch only applies when the value is greater than zero.

During `generate_speed_moves`, keep the existing role and first-layer speed selection, then cap only `ToolpathMoveKind::Print` moves. Travel and Z travel feedrates must not be capped.

The cap must be derived from the already generated extrusion stream instead of duplicating extrusion-area logic in `speeds.rs`:

1. Track `last_point: Option<Point2>` and `last_print_e: f64` while iterating layers in order.
2. Travel moves update `last_point` to their XY point and leave `last_print_e` unchanged.
3. Print moves require a current cumulative `E`; compute XY distance from `last_point.unwrap_or(current_point)` to the current point.
4. Compute `delta_e = current_e - last_print_e`, then update `last_print_e = current_e` after deriving the cap input.
5. After deriving the cap input for a print move, update `last_point = Some(current_point)` so consecutive print segments measure from the previous print endpoint.
6. The tracking state is intentionally shared across layers because `generate_extrusion_moves` emits cumulative E positions across the whole print.
7. Convert to effective volume per path millimeter using the active filament area:
   `mm3_per_mm = (delta_e * filament_area) / distance`.
8. If `mm3_per_mm > 0`, cap speed with:
   `min(configured_speed_mm_s, filament_max_volumetric_speed / mm3_per_mm)`.
9. Store both `speed_mm_s` and `feedrate_mm_min` from the capped speed.

This intentionally composes with current extrusion behavior, including `filament_flow_ratio`, `print_flow_ratio`, first-layer line width, and role flow ratios, because those are already reflected in cumulative `E`.

## File Placement and Size Constraints

- Put `filament_max_volumetric_speed` parsing in a new focused module, `crates/ares-core/src/options/volumetric_speed.rs`.
- Wire that module into `crates/ares-core/src/options.rs` with one `mod volumetric_speed;` line and a narrow call from `SliceOptions::speed_options()`.
- Put cap helper logic in a new focused `crates/ares-core/src/speeds/volumetric.rs` submodule so `speeds.rs` stays under the 400 LOC gate.
- Put unit tests for the cap helper in `crates/ares-core/src/speeds/volumetric/tests.rs` or keep them in the new submodule if shorter; avoid growing `crates/ares-core/src/speeds/tests.rs` beyond 400 LOC.
- Add parser tests in a new `crates/ares-core/src/options/tests/filament_max_volumetric_speed.rs`.
- Add pipeline G-code tests in a new `crates/ares-core/src/pipeline/tests/filament_max_volumetric_speed.rs`.

## Parsing Rules

- Accept number, numeric string, array, or delimited numeric string through the existing numeric-vector parser shape used by other filament vector options.
- Use the first parsed value for the current single-filament pipeline.
- Reject empty vectors, non-numeric values, non-finite values, and negative values.
- Missing option defaults to `2.0`.
- A parsed value of `0.0` is valid and disables the cap.

## Docs Impact

No user-facing documentation, registry metadata, or roadmap updates are required for this slice. The existing option metadata already records the upstream `PrintConfig.hpp` tuple; this work changes runtime consumption and is documented by the spec, plan, tests, and commit.

## Out of Scope

- `filament_adaptive_volumetric_speed` and `volumetric_speed_coefficients`.
- Wipe tower, toolchange, purge, and calibration uses of `filament_max_volumetric_speed`.
- Multi-extruder per-tool cap selection beyond using the first filament value.
- Global `max_volumetric_speed`; Orca comments that branch out in the cited G-code section.
- Support, ironing, gap-fill, top/bottom solid infill, overhang, and other roles Ares does not yet generate.
- Any new Ares-owned speed model independent of the cited `libslic3r` behavior.

## Acceptance Criteria

- `filament_max_volumetric_speed` changes emitted print feedrates when the configured role speed would exceed the volumetric cap.
- A value of `0` leaves print feedrates unchanged.
- Travel feedrates remain controlled by travel speed options and are not limited by the filament volumetric cap.
- The cap composes with `filament_flow_ratio`: increasing filament flow ratio increases effective volume per millimeter and lowers the capped feedrate for the same path.
- Invalid negative, non-finite, or non-numeric values are rejected at option parsing.
- Tests cover unit-level speed capping and pipeline-level G-code feedrate emission.
- Fresh verification must include targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate for changed Rust files.
