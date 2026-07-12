# Consume Wall Transition Filter Runtime

## Goal

Consume the next concrete part of Orca's Arachne wall-transition option family in Ares' current rectangular thin-wall compatibility shell. This follow-up slice must keep the earlier typed parsing intact, add Orca's missing nozzle-relative millimeter conversions, and make `wall_transition_filter_deviation` plus `wall_distribution_count` affect the one-centerline Arachne thin-wall proxy without implementing full `Arachne::WallToolPaths`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1021-1024`: `PrintObjectConfig` stores `wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, and `wall_distribution_count`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7003-7027`: `wall_transition_length` and `wall_transition_filter_deviation` are min-only percent options over nozzle diameter, with defaults `100` and `25`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7029-7049`: `wall_transition_angle` is a `1..=59` degree option and `wall_distribution_count` is a min-`1` integer.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:47-54`: Orca converts transition length and filter deviation from percent to millimeters using the minimum configured nozzle diameter, then copies angle and distribution count into `WallToolPathsParams`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:519-553`: Orca sends transition length, angle, distribution count, and filter deviation into the beading strategy and skeletal trapezoidation.
- `OrcaSlicer/src/libslic3r/Arachne/BeadingStrategy/BeadingStrategyFactory.cpp:33-45`: Orca builds a `DistributedBeadingStrategy` with `wall_distribution_count` and then wraps it in `WideningBeadingStrategy` when thin walls are printed.
- `OrcaSlicer/src/libslic3r/Arachne/BeadingStrategy/BeadingStrategy.cpp:29-33`: configured transition length is the default length for positive bead-count transitions; the `0` lower-bead-count case uses a fixed tiny length.
- `OrcaSlicer/src/libslic3r/Arachne/BeadingStrategy/BeadingStrategy.hpp:73-85,109-115`: transition length smooths bead-count jumps, while transition angle controls where transition ribs are introduced.
- `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:893-919`: Orca filters transition ends using transition half-lengths.
- `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:952-958`: Orca dissolves nearby transitions only while line-width deviation stays within `allowed_filter_deviation`; the current calculation explicitly notes the single-distribution-count assumption.

## Current Ares Boundary

- `crates/ares-core/src/options/overhang_reverse.rs` already parses and validates all four wall-transition options into `PerimeterOptions`.
- `crates/ares-core/src/perimeters/options.rs` and `crates/ares-core/src/perimeters/options/arachne.rs` already expose raw percent/degree/count getters.
- `crates/ares-core/src/perimeters/options/arachne.rs` already converts min-feature and bead-width percentages using `min_nozzle_diameter`; transition length and filter deviation still lack matching millimeter accessors.
- `crates/ares-core/src/perimeters/thin_walls.rs` emits one open external centerline for an axis-aligned rectangular contour when `detect_thin_wall` is enabled and the next internal wall loop collapses in one axis.
- That open centerline is Ares' current temporary shell for thin-wall output. It does not have Orca skeletal trapezoidation, positive bead-count transition ribs, variable-width junctions, or arbitrary polygon topology.
- The previous min-feature/bead-width runtime slice already carries Arachne one-bead width metadata through `PerimeterPath`, `PrintPath`, `ToolpathMove`, and `ExtrusionMove`.

## Included Behavior

1. Add `PerimeterOptions::wall_transition_length_mm()` and `PerimeterOptions::wall_transition_filter_deviation_mm()` in `crates/ares-core/src/perimeters/options/arachne.rs`, using the same `raw_percent / 100 * min_nozzle_diameter` rule as Orca `WallToolPaths.cpp:47-51`.
2. Preserve raw getters and validation for `wall_transition_length_percent`, `wall_transition_filter_deviation_percent`, `wall_transition_angle_degrees`, and `wall_distribution_count`.
3. Keep `wall_transition_length_mm()` observable but do not use it to filter Ares' current open thin-wall proxy in this slice. The cited upstream uses the configured length for positive bead-count transitions, while the `0 -> 1` case has a fixed tiny length and Ares has no positive bead-count transition topology yet.
4. Apply new runtime filtering only when `options.wall_generator() == WallGenerator::Arachne`, `detect_thin_wall` is enabled, the current length filter passes, and the candidate also satisfies `min_feature_size`.
5. Define the current proxy's line-width deviation as the amount by which the candidate collapsed-axis thickness exceeds two selected minimum bead widths: `max(0, thickness - 2 * selected_min_bead_width)`.
6. Divide that deviation by `wall_distribution_count` before comparing with `wall_transition_filter_deviation_mm()`. This keeps `wall_distribution_count = 1` aligned with the cited single-count assumption and lets higher distribution counts absorb more over-width in the one-centerline shell.
7. Suppress the Arachne open centerline when the distributed line-width deviation is greater than `wall_transition_filter_deviation_mm()`.
8. Preserve the existing `max(thickness, selected_min_bead_width)` effective line-width assignment for surviving centerlines.
9. Preserve Classic wall-generator behavior under the same transition values; Classic detected centerlines must not receive Arachne transition filtering.
10. Preserve closed external/internal/overhang perimeter loops; this slice must not apply transition filtering to closed loops or closed-loop simplification.

## Deferred Behavior

- Full `Arachne::WallToolPathsParams` parity.
- Using `wall_transition_length` to generate or filter positive bead-count transition ends.
- Applying `wall_transition_angle` to skeletal rib creation or wedge-shaped regions.
- Exact `DistributedBeadingStrategy` width distribution for multiple variable-width walls.
- Exact line-width deviation computation on Orca scaled coordinates and junction widths.
- `SkeletalTrapezoidation`, transition mids/ends, transition dissolution, and beading propagation.
- Variable-width closed walls and multiple centerline output for overwide regions.
- Replacing Ares' `detect_thin_wall` gate with Orca Arachne's always-on `fill_outline_gaps`.
- Arbitrary polygon thin-wall discovery and Orca binary E2E geometry parity.

## Acceptance Criteria

1. Existing wall-transition option parsing tests continue to pass.
2. `wall_transition_length_mm()` and `wall_transition_filter_deviation_mm()` convert percentages over the minimum configured nozzle diameter, including multi-nozzle inputs.
3. Default wall-transition values preserve current detected rectangular thin-wall output for the existing narrow test shape.
4. An Arachne detected centerline whose collapsed-axis thickness creates line-width deviation greater than the configured filter margin is suppressed.
5. Increasing `wall_transition_filter_deviation` can make that same centerline survive.
6. Increasing `wall_distribution_count` can make that same centerline survive by spreading the proxy deviation.
7. Classic detected thin-wall centerlines remain unchanged by the new Arachne filter.
8. The surviving centerline still carries the effective line-width override through downstream tests covered by the existing min-feature/bead-width suite.
9. `wall_transition_length_mm()` is tested as observable runtime state but not asserted to change geometry in this slice.
10. Touched Rust files remain under or at the 400-line split guideline.

## Verification

- Update focused wall-transition option tests for millimeter conversion.
- Replace the existing geometry-neutral wall-transition perimeter test with runtime filter tests that prove default preservation, filter suppression, filter survival, distribution-count survival, and Classic preservation.
- Re-run min-feature/bead-width perimeter tests because this slice composes with their Arachne thin-wall width assignment.
- `cargo nextest run -p ares-core wall_transition_parameters min_feature_bead_width`
- `cargo nextest run -p ares-core min_length_factor wall_generator wall_sequence wall_direction`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
- Touched Rust LOC guard with `wc -l`.

## Docs Impact

Update `docs/roadmap.md` with a dated follow-up entry stating that transition length/filter percentages now have Orca-compatible nozzle-relative millimeter accessors, and that `wall_transition_filter_deviation` plus `wall_distribution_count` now affect Ares' current Arachne rectangular thin-wall centerline filter. Keep full transition length/angle geometry and skeletal Arachne parity deferred.

## Self-Review

- This does not pretend to implement positive bead-count transition length behavior; that remains blocked on a real Arachne topology port.
- The only geometry change is tied to an existing Ares proxy that already stands in for Arachne thin-wall output.
- The distribution-count formula is intentionally a proxy: it is source-cited to Orca's single-count deviation assumption and distributed beading intent, but it is not full `DistributedBeadingStrategy` parity.
