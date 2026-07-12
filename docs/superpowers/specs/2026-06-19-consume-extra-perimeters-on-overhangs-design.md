# Consume extra_perimeters_on_overhangs Design

## Goal

Consume OrcaSlicer's `extra_perimeters_on_overhangs` option as concrete perimeter, print-path, and G-code behavior in `ares-core`. This slice must add an executable overhang perimeter path on eligible unsupported rectangles instead of adding more option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1200` declares `extra_perimeters_on_overhangs` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1439-1444` registers it as a boolean quality option with default `false` and describes additional perimeter paths over steep overhangs and bridge areas that cannot be anchored.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1089-1114` applies extra overhang perimeters when spiral vase is off, lower slices exist, `detect_overhang_wall` and `extra_perimeters_on_overhangs` are true, `wall_loops > 0`, and the current layer is above raft layers. It generates extra overhang paths, prepends them to island perimeters, and removes their filled area from fill surfaces.

## Current Ares Boundary

- `crates/ares-core/src/options/overhang_reverse.rs` owns current runtime parsing of perimeter options into `PerimeterOptions`.
- `crates/ares-core/src/perimeters.rs` owns rectangle perimeter generation, wall-loop count resolution, and existing whole-loop overhang role assignment.
- `crates/ares-core/src/perimeters/overhang.rs` already maps a fully unsupported rectangular external loop to `PerimeterRole::Overhang` when `detect_overhang_wall` is enabled.
- `crates/ares-core/src/print_paths.rs`, move generation, extrusion generation, speed generation, and G-code formatting already carry `PerimeterRole::Overhang` as `PrintPathRole::OverhangPerimeter`.
- Ares does not yet model Orca `ExPolygons` fill surfaces or subtract `filled_area` from infill regions.

## Included Behavior

1. Parse `extra_perimeters_on_overhangs` as a bool in `PerimeterOptions`, defaulting to `false` to match Orca.
2. Expose the parsed value through a `PerimeterOptions` getter and a builder used by tests.
3. Add one additional inset `PerimeterRole::Overhang` loop when all of these are true:
   - the current layer has an immediately preceding contour layer,
   - `detect_overhang_wall` is true,
   - `extra_perimeters_on_overhangs` is true,
   - the effective wall-loop count is greater than zero,
   - the current contour is an axis-aligned rectangle,
   - the current external loop is classified as `PerimeterRole::Overhang` by the existing Ares overhang detector,
   - the rectangle has room for one more inset loop inside the already emitted effective wall loops.
4. Place the extra loop in the first available perimeter slot inside the emitted wall loops:
   - for one emitted wall loop, the extra loop uses the existing first-internal shrink distance `(external_line_width + internal_line_width) / 2.0`;
   - for multiple emitted wall loops, the extra loop uses that first-internal shrink plus one `internal_line_width` for each emitted internal loop.
5. Use the configured `wall_direction` and existing `overhang_reverse` orientation logic for the extra loop.
6. Let the existing `wall_sequence` ordering function place the extra loop with the other non-external perimeter paths; the extra path keeps the `Overhang` role after ordering.
7. Preserve existing behavior when the option is missing or false.
8. Preserve existing behavior when `detect_overhang_wall` is false.
9. Preserve `wall_loops = 0`: the option must not synthesize any perimeter.
10. Carry the extra loop through `PrintPathRole::OverhangPerimeter`, moves, extrusion, speed, and G-code comments using existing role plumbing.

## Deferred Behavior

- Full `generate_extra_perimeters_over_overhangs(...)` parity is deferred until Ares has source-cited polygon offset, intersection, union, diff, and fill-surface subtraction boundaries.
- Segment-level steep-overhang clipping and mixed supported/unsupported perimeter spans are deferred. This slice only covers the current Ares whole-rectangle unsupported role boundary.
- Removing extra-perimeter covered area from infill is deferred because Ares currently generates infill independently from perimeter-covered fill-surface subtraction.
- Spiral vase, raft-layer offsets, lower-slice growth series, Arachne path generation, anchoring sort, and medial-axis gap paths are deferred.
- `make_overhang_printable`, `slowdown_for_curled_perimeters`, `detect_thin_wall`, `min_feature_size`, `min_bead_width`, ironing, fuzzy skin, and `ensure_vertical_shell_thickness` remain outside this slice.

## Acceptance Criteria

1. `extra_perimeters_on_overhangs` defaults to `false` in `PerimeterOptions`.
2. Explicit `extra_perimeters_on_overhangs: true` parses into `PerimeterOptions`.
3. Non-boolean `extra_perimeters_on_overhangs` returns `SliceError::InvalidInput` and names the option.
4. With `wall_loops = 1`, `detect_overhang_wall = true`, `extra_perimeters_on_overhangs = true`, and a second-layer unsupported rectangle, `generate_perimeters` emits two `PerimeterRole::Overhang` paths on the second layer: the existing external rectangle and one inset extra loop.
5. With the same geometry and `extra_perimeters_on_overhangs = false` or a missing value, the second layer keeps only the existing overhang perimeter.
6. With `detect_overhang_wall = false`, the option does not add an extra path.
7. With `wall_loops = 0`, the option does not add any path.
8. With `wall_loops = 2`, the extra loop is placed inside the configured external plus internal wall loops, not duplicated over the first internal wall.
9. Pipeline/G-code regression proves enabling the option increases second-layer `overhang_perimeter` print-path and move output for the unsupported rectangle.
10. The same G-code regression explicitly asserts generated `;PERIMETER:overhang:` and `;PRINT_PATH:overhang_perimeter:` comments for the added inset loop.
11. Existing overhang speed, overhang reversal, wall sequence, only-one-wall, alternate-extra-wall, extrusion, speed, and infill tests continue to pass.
12. `docs/roadmap.md` records that the historical one-wall quality registry milestone no longer defers all `extra_perimeters_on_overhangs` behavior: this slice consumes rectangle-only extra overhang perimeter generation, while full polygon clipping and fill-surface subtraction remain deferred.
13. `cargo test -p ares-core --lib` passes.
14. `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
15. Rust source files under `crates/**/src/**/*.rs` remain at or below 400 LOC.

## Documentation Impact

This executable slice is captured in this SDD spec and its implementation plan. No architecture document update is required because crate boundaries do not change. Update the historical one-wall quality registry paragraph in `docs/roadmap.md` that currently says overhang extra perimeter generation remains deferred, so it records this rectangle-only runtime slice and keeps full polygon clipping plus fill-surface subtraction deferred.

## Safety

This is a local `ares-core` slicing change with no filesystem, terminal, OpenGL, native UI, or platform-specific runtime behavior. It adds no dependencies and preserves WASM suitability. The change is constrained to perimeter option parsing and deterministic rectangle perimeter generation.
