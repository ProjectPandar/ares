# Consume Skirt Start Angle Design

## Goal

Consume Orca `skirt_start_angle` as executable Ares skirt path behavior instead of leaving it as registry/metadata-only option coverage.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:927` declares `PrintObjectConfig` `skirt_start_angle`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5549-5557` defines `skirt_start_angle` as `coFloat`, labels it "Skirt start point", constrains it to `[-180, 180]`, and defaults it to `-135`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4277-4302` computes a desired start point from the loop bounds center, loop corner radius, and configured angle where zero degrees points right and positive angles rotate counter-clockwise.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4331-4390` passes the desired start point only for the first layer's first emitted skirt loop.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5112-5114` routes combined skirt emission through `generate_skirt(...)` using `layer.object()->config().skirt_start_angle`.

## Current Ares State

- `skirt_start_angle` exists in the option registry but is not parsed into runtime skirt options.
- `SkirtOptions` carries loop count, distance, height, speed, draft shield, skirt type, minimum skirt length, and single-loop draft-shield behavior, but not the start angle.
- Ares combined skirts currently emit rectangular loops starting at the lower-left corner in generated point order.
- Per-object skirts are already rejected because Ares has no per-object contour ownership or object-specific skirt artifacts.

## Design

Add `skirt_start_angle_degrees` to `SkirtOptions`, defaulting to Orca's `-135.0`. Parse `skirt_start_angle` at the `SliceOptions::skirt_options()` boundary as a finite numeric value in `[-180.0, 180.0]`.

For combined skirts, preserve the existing rectangular skirt geometry and loop count. When generating the first non-empty Ares skirt layer, reorder only the first emitted skirt path so its first point is the rectangle corner nearest Orca's desired start point for the configured angle. Compute that desired point from the rectangle bounds center and corner radius, matching the upstream angle convention: `0` points right, `90` points up, `180` and `-180` point left, and `-90` points down. The current Ares skirt path has four rectangle corners, so the closest existing corner is the appropriate current representation of Orca's continuous-loop start point.

Only the first path on the first non-empty generated skirt layer is reordered. Other loops on that layer and all later generated layers keep their existing point order. This mirrors upstream `GCode.cpp:4378-4383`, where the desired start point is passed only when `first_layer && i == loops.first`.

This slice composes with existing `min_skirt_length` and `single_loop_draft_shield`: min-length may add extra first-layer loops, but only the first emitted path is start-angle-reordered; later single-loop draft-shield layers are not reordered by start angle.

For `SkirtType::PerObject`, keep the existing unsupported-input error. Per-object start-angle behavior remains deferred with per-object skirt generation.

## Deferred Upstream Behavior

- Exact continuous loop splitting at arbitrary non-corner points remains deferred until Ares owns richer loop geometry than four-corner rectangular paths.
- Per-object `skirt_start_angle` remains deferred until per-object skirt artifacts exist.
- Exact Orca rounded offset geometry remains deferred; Ares continues using rectangular combined-skirt bounds.

## Acceptance Criteria

- `SliceOptions::skirt_options()` returns `skirt_start_angle_degrees() == -135.0` when the option is absent.
- Numeric `skirt_start_angle` values, including numeric strings, parse to the matching `SkirtOptions` value when they are finite and within `[-180.0, 180.0]`.
- Non-numeric, non-finite, and out-of-range `skirt_start_angle` values return `SliceError::InvalidInput` at the skirt option boundary.
- With the default `-135.0` angle, existing skirt path output remains unchanged because the nearest rectangle corner is the current lower-left starting point.
- With `skirt_start_angle: 45`, the first emitted skirt path starts at the upper-right rectangle corner while preserving the same closed rectangle points and loop count.
- With `skirt_start_angle: 0`, the first emitted skirt path starts at the lower-right rectangle corner because it is the deterministic first nearest corner on the right side of Ares' rectangular representation.
- With `skirt_loops: 2`, only the first emitted path on the first non-empty generated skirt layer is reordered; the second loop keeps the existing lower-left start.
- With `draft_shield: "enabled"` and multiple generated layers, later generated layers keep existing lower-left path order.
- With `single_loop_draft_shield == true`, later generated single-loop draft-shield paths keep existing lower-left path order.
- The reordered first path appears through real skirt artifacts, print paths, moves, extrusion moves, speed moves, and G-code path-following commands rather than metadata-only changes.
- `skirt_type: "perobject"` still returns the existing unsupported per-object error even when `skirt_start_angle` is set.
- Rust source files touched by this slice stay below the repository 400 LOC split threshold.

## Files

- Modify `crates/ares-core/src/skirts/mod.rs` for the `SkirtOptions` field, accessor/builder, and first-path point-order selection.
- Add focused tests in `crates/ares-core/src/skirts/tests.rs` if the file remains below 400 LOC; split if needed.
- Modify `crates/ares-core/src/options.rs` to parse `skirt_start_angle` and pass it into `SkirtOptions`.
- Add `crates/ares-core/src/options/tests/skirt_start_angle.rs` for focused option-boundary tests.
- Modify `crates/ares-core/src/options/tests.rs` only by adding `skirt_start_angle` to the existing focused module macro while preserving the 400 LOC limit.
- Add `crates/ares-core/src/tests/skirt_start_angle_gcode.rs` for slice/G-code regressions.
- Modify `crates/ares-core/src/tests/mod.rs` only to register the focused G-code test module.

## Docs Impact

No architecture or roadmap update is required for this narrow runtime consumption slice. This source-cited design document, the implementation plan, and regression tests document the included combined-skirt behavior and the explicit deferrals.

## Verification

- `cargo test -p ares-core skirt_start_angle --lib`
- `cargo test -p ares-core skirt_start_angle_gcode --lib`
- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `wc -l` on touched Rust files to confirm none exceed 400 LOC
