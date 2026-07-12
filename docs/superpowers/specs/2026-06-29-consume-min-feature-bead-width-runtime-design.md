# Consume Minimum Feature and Bead-Width Runtime

## Goal

Consume Orca's Arachne minimum feature and bead-width options in concrete Ares slicing output for the current detected rectangular thin-wall compatibility shell. This slice must replace the previous geometry-unchanged scaffold for `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width` with source-cited thin-wall suppression and bead-width extrusion behavior, without implementing full `Arachne::WallToolPaths` or a new Ares-owned wall generator.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1025-1027`: `PrintObjectConfig` stores `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7051-7060`: `min_feature_size` is a min-only percent option, default `25`, expressed over nozzle diameter.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7099-7107`: `initial_layer_min_bead_width` is a min-only percent option, default `85`, expressed over nozzle diameter.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7109-7119`: `min_bead_width` is a min-only percent option, default `85`, expressed over nozzle diameter.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:26-44`: Orca converts all three percent values over the minimum configured nozzle diameter and selects `initial_layer_min_bead_width` only for layer `0`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.hpp:17`: Orca's Arachne `fill_outline_gaps` thin-wall widening gate is always `true`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:77-78`: `WallToolPaths` stores scaled `min_feature_size` and `min_bead_width`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:521-535`: `min_bead_width` participates in split/add thresholds and the beading strategy factory.
- `OrcaSlicer/src/libslic3r/Arachne/BeadingStrategy/BeadingStrategyFactory.cpp:39-45`: Orca applies `WideningBeadingStrategy` only when thin walls are enabled.
- `OrcaSlicer/src/libslic3r/Arachne/BeadingStrategy/WideningBeadingStrategy.cpp:27-41`: when thickness is below the optimal width, Orca emits one bead only if thickness is at least `min_input_width`; emitted width is `max(thickness, min_output_width)`.
- `OrcaSlicer/src/libslic3r/Arachne/BeadingStrategy/WideningBeadingStrategy.cpp:57-64`: thickness below `min_input_width` has optimal bead count `0`; thickness at or above it has at least one bead.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1247-1253`: Orca's `detect_thin_wall` option belongs to the Classic perimeter generator's thin-wall branch, not to Arachne `fill_outline_gaps`.

## Current Ares Boundary

- `crates/ares-core/src/options/overhang_reverse.rs` already parses the three percent options into `PerimeterOptions`.
- `crates/ares-core/src/perimeters/options.rs` and `crates/ares-core/src/perimeters/options/arachne.rs` already store and expose the raw percent values.
- `crates/ares-core/src/perimeters/thin_walls.rs` currently emits open external centerlines for axis-aligned rectangular contours when `detect_thin_wall` is enabled and the next internal wall loop collapses in one axis. This is a temporary Ares shell that represents thin-wall output; it does not yet match Orca Arachne's always-on `fill_outline_gaps`.
- `crates/ares-core/src/perimeters/thin_walls.rs` currently filters those open centerlines only by `min_length_factor`.
- `crates/ares-core/src/perimeters.rs` has no per-perimeter line-width metadata; every path reaches extrusion with role-default line width.
- `crates/ares-core/src/print_paths.rs` and `crates/ares-core/src/moves.rs` already carry layer-height metadata through the pipeline, but not path-specific line-width metadata.
- `crates/ares-core/src/extrusions.rs` and `crates/ares-core/src/speeds.rs` already have output-side `effective_line_width_mm` metadata for extrusion and speed moves.
- `crates/ares-core/src/extrusions.rs` and `crates/ares-core/src/extrusions/options/accessors.rs` already calculate E from line width, layer height, role flow, filament diameter, and first-layer flow.
- Ares does not yet have `Arachne::WallToolPaths`, variable-width wall topology, skeletal trapezoidation, arbitrary polygon thin-wall discovery, full beading strategies, or wall split/add thresholds.

## Included Behavior

1. Compute the minimum configured nozzle diameter in `SliceOptions::perimeter_options()` from `nozzle_diameters()` and store it on `PerimeterOptions` so percent-to-millimeter conversion follows the cited Orca `min_nozzle_diameter` rule.
2. Put the raw-percent-to-millimeter converters for `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width` in `crates/ares-core/src/perimeters/options/arachne.rs`, not in the already-large parent options file.
3. Select `initial_layer_min_bead_width` only when the generated layer id is `0`; select `min_bead_width` for every later layer.
4. Apply this runtime behavior only when `options.wall_generator() == WallGenerator::Arachne` and `options.detect_thin_wall()` is true. This gate is an explicit temporary Ares-shell divergence from upstream Arachne's always-on `fill_outline_gaps`; full reconciliation is deferred until Ares ports the Arachne thin-wall boundary rather than the current rectangular detected-thin-wall shell.
5. Define Ares' current detected thin-feature thickness as the full rectangle extent on the collapsed axis: `max_y - min_y` for a horizontal centerline and `max_x - min_x` for a vertical centerline.
6. Suppress the detected open thin-wall centerline when its thickness is below converted `min_feature_size`.
7. For surviving detected open thin-wall centerlines, set a per-path effective line-width override to `max(thickness, selected_min_bead_width)`, matching the cited `WideningBeadingStrategy::compute` one-bead rule for Ares' current one-centerline shell.
8. Keep closed external/internal/overhang perimeter loops on role-default line widths in this slice.
9. Add optional effective line-width metadata to `PerimeterPath`, `PrintPath`, and `ToolpathMove`, defaulting to `None`, following the existing `with_effective_layer_height_mm` / `with_effective_line_width_mm` builder style.
10. Propagate optional effective line width from perimeter paths through print paths and toolpath moves into the existing `ExtrusionMove::effective_line_width_mm` metadata; do not add duplicate line-width fields to `ExtrusionMove` or `SpeedMove`.
11. When a print `ToolpathMove` has a line-width override, compute E using that override while retaining the path role's flow ratios, filament diameter, first-layer flow behavior, and diagnostic role; preserve existing `ExtrusionMove::effective_line_width_mm()` reporting for both default and override widths.
12. Preserve role-default extrusion behavior when no line-width override is present.
13. Preserve `WallGenerator::Classic` compatibility behavior for detected thin-wall centerlines; this Arachne-specific option cluster must not change Classic output.
14. Pass `layer_id` into `thin_walls::append_rectangular_thin_wall(...)` from the existing call site so the function can select the first-layer bead width.
15. Preserve existing `min_length_factor` filtering order: feature-size suppression and width assignment occur for an Arachne detected centerline that also satisfies the current length threshold. This ordering is an Ares-shell composition of the current open-wall length filter and the new Arachne-derived width filter.

## Deferred Behavior

- Full `Arachne::WallToolPathsParams` parity beyond the three converted values.
- Full `BeadingStrategyFactory`, `DistributedBeadingStrategy`, `RedistributeBeadingStrategy`, `LimitedBeadingStrategy`, and wall split/add threshold behavior.
- Arbitrary polygon thin-feature discovery and non-rectangular open wall generation.
- Scaled-coordinate conversion and exact Orca integer rounding.
- Variable-width closed perimeter walls.
- Multiple bead output for wider features.
- Interaction with wall-transition parameters beyond existing parse-only state.
- Exact top/bottom surface classification beyond current first/topmost `min_length_factor` protection.
- Replacing Ares' `detect_thin_wall` shell gate with Orca Arachne's always-on `fill_outline_gaps`.
- Orca binary E2E geometry or extrusion parity.

## Acceptance Criteria

1. The three options still parse, validate, store, and expose the existing Orca defaults, min-only percent semantics, and values above `100`.
2. `PerimeterOptions` converts the three raw percent values over the minimum configured nozzle diameter, including multi-nozzle input.
3. An Arachne detected thin wall whose thickness is below converted `min_feature_size` does not emit the open centerline.
4. A matching Classic detected thin wall still emits the current open centerline under the same configured minimum feature size.
5. A surviving first-layer detected thin wall uses `initial_layer_min_bead_width` for its effective line width.
6. A surviving later-layer detected thin wall uses `min_bead_width` for its effective line width.
7. The effective line-width override reaches `LayerPrintPaths`, `LayerToolpathMoves`, and `LayerExtrusionMoves`.
8. Extrusion E and `ExtrusionMove::effective_line_width_mm()` reflect the override for thin-wall print moves.
9. Existing rectangular closed perimeter geometry remains unchanged when `detect_thin_wall` is false.
10. Existing `min_length_factor`, wall-generation, wall-sequence, wall-direction, and wall-simplification tests continue to pass.
11. Touched Rust files remain under or at the 400-line split guideline.

## Verification

- Replace the parse-only geometry-deferral test in `crates/ares-core/src/perimeters/tests/min_feature_bead_width.rs` with focused runtime tests.
- Add focused propagation/extrusion assertions where the existing tests do not prove the line-width override reaches E calculation.
- `cargo nextest run -p ares-core min_feature_bead_width min_length_factor`
- `cargo nextest run -p ares-core wall_generator wall_sequence wall_direction wall_maximum_resolution_deviation`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
- Touched Rust LOC guard with `wc -l`.

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width` now affect Ares' detected Arachne thin-wall centerlines through nozzle-relative min-feature suppression and per-path bead-width extrusion, while full Arachne beading strategy and variable-width wall topology remain deferred.
