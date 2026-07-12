# Consume Wall Simplification Runtime

## Goal

Consume Orca's Arachne wall maximum resolution/deviation behavior in Ares perimeter generation by simplifying low-deviation perimeter vertices. This slice must replace the previous parse-only scaffold for `wall_maximum_resolution` and `wall_maximum_deviation` with observable geometry behavior for Ares perimeter paths, while staying inside the current perimeter generator and not implementing full variable-width Arachne.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1030-1031`: `PrintObjectConfig` stores `wall_maximum_resolution` and `wall_maximum_deviation`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7076-7097`: Orca defines `wall_maximum_resolution` as millimeters, default `0.5`, range `0.005..=0.5`, and `wall_maximum_deviation` as millimeters, default `0.025`, range `0.005..=0.05`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:58-62`: Orca copies configured millimeter values into `WallToolPathsParams`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:487-503`: Arachne uses the configured resolution/deviation to simplify prepared outlines before wall generation.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:704-710`: Arachne forwards `wall_maximum_resolution` and `wall_maximum_deviation` into generated toolpath simplification.
- `OrcaSlicer/src/libslic3r/Arachne/utils/ExtrusionLine.hpp:155-183`: Arachne's simplification contract removes junctions connected to short segments only when resulting deviation stays within the allowed distance.

## Current Ares Boundary

- `crates/ares-core/src/options/overhang_reverse.rs` already parses `wall_maximum_resolution` and `wall_maximum_deviation` into `PerimeterOptions`.
- `crates/ares-core/src/perimeters/options.rs` and `crates/ares-core/src/perimeters/options/arachne.rs` already store and expose both millimeter values.
- `crates/ares-core/src/perimeters.rs` currently emits the oriented/fuzzy external contour directly, then current rectangular internal loops, overhang loops, and thin walls.
- `crates/ares-core/src/perimeters/fuzzy_skin.rs` can intentionally add external/internal wall points after base wall generation.
- `crates/ares-core/src/perimeters/tests/wall_maximum_resolution_deviation.rs` currently asserts the options do not affect geometry.
- Ares does not yet have `Arachne::WallToolPaths`, variable-width `ExtrusionLine`, scaled coordinates, skeletal trapezoidation, extrusion-area deviation checks, or prepared-outline boolean repair.

## Included Behavior

1. Add a perimeter-local simplification pass for closed perimeter loops.
2. Put the pass in a small `crates/ares-core/src/perimeters/simplification.rs` module rather than growing `perimeters.rs`.
3. Remove an intermediate vertex when both adjacent segments are at or below `wall_maximum_resolution` and replacing the two segments with one segment keeps the vertex's point-to-segment distance at or below `wall_maximum_deviation`.
4. Treat the f64 point-to-segment rule as Ares' current point-loop adaptation of Orca's cited allowed-deviation contract; exact scaled-integer `ExtrusionLine` junction metrics remain deferred.
5. Repeat simplification until no more vertices can be removed, while preserving at least three vertices.
6. Apply the pass only when `options.wall_generator() == WallGenerator::Arachne`, because the cited upstream runtime behavior lives in `Arachne::WallToolPaths`.
7. Apply the pass to Ares-generated closed perimeter loops after `wall_direction().orient_points(...)` creates the ordered base loop and before any fuzzy-skin point generation, `overhang_reverse::orient_points(...)`, or `seams::position_loop(...)`.
8. Preserve fuzzy-skin generated vertices by simplifying only the base wall loop before `fuzzy_skin().external_points(...)` or `fuzzy_skin().internal_wall_points(...)` runs.
9. Preserve existing rectangular perimeter output under defaults because rectangle segments are longer than the configured maximum resolution.
10. Preserve `WallGenerator::Classic` compatibility geometry for the same low-deviation notch input.
11. Preserve open detected thin walls; they are already governed by `min_length_factor` and should not enter this closed-loop simplification pass.
12. Add tests proving non-default maximum resolution/deviation remove a low-deviation notch from emitted perimeter and G-code geometry for Arachne.
13. Add tests proving too-small resolution or too-small deviation preserves the notch.

## Deferred Behavior

- Full `Arachne::WallToolPaths` parity.
- Variable-width `ExtrusionLine` simplification and extrusion-area deviation checks.
- Scaled-coordinate conversion and exact Orca integer rounding.
- Exact Arachne junction-width and accumulated-area simplification metrics.
- Prepared-outline offset/union/self-intersection repair.
- Wall-transition, wall-distribution, min-feature, and bead-width geometry behavior.
- Orca binary E2E parity for the simplifier.

## Acceptance Criteria

1. `wall_maximum_resolution` and `wall_maximum_deviation` still parse, validate, store, and expose existing Orca defaults and ranges.
2. A contour with a short, low-deviation extra vertex produces fewer perimeter/G-code points when the configured thresholds allow removal.
3. The same contour preserves the extra vertex when either maximum resolution or maximum deviation is below the required threshold.
4. The same contour preserves the extra vertex when `wall_generator = "classic"`.
5. Enabling fuzzy skin with simplification thresholds that would otherwise remove low-deviation points still preserves fuzzy-skin point generation behavior.
6. Existing rectangular perimeter geometry tests continue to pass.
7. Existing `min_length_factor` thin-wall behavior continues to pass.
8. Touched Rust files remain under or at the 400-line split guideline.

## Verification

- Focused RED/GREEN tests in `crates/ares-core/src/perimeters/tests/wall_maximum_resolution_deviation.rs`.
- Pipeline/G-code test in `crates/ares-core/src/pipeline/tests/wall_maximum_resolution_deviation.rs` if no equivalent G-code assertion exists after perimeter tests.
- `cargo nextest run -p ares-core wall_maximum_resolution_deviation min_length_factor wall_sequence fuzzy_skin`
- `cargo nextest run -p ares-core wall_generator wall_direction precise_outer_wall`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
- Touched Rust LOC guard with `wc -l`.

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `wall_maximum_resolution` and `wall_maximum_deviation` now affect Ares perimeter simplification for eligible closed-loop vertices, while full Arachne variable-width simplification remains deferred.
