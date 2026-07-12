# Consume Sparse Infill Flow Ratio Design

## Goal

Consume OrcaSlicer `sparse_infill_flow_ratio` in Ares extrusion generation so the already recognized option changes sparse infill E output instead of remaining metadata-only.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1354` registers `sparse_infill_flow_ratio` as a float with min `0`, max `2`, and default `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1219` includes `sparse_infill_flow_ratio` in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6423` multiplies internal infill material by `m_config.sparse_infill_flow_ratio` when the path role is `erInternalInfill`.

## Ares Destination Boundary

- `crates/ares-core/src/options.rs`: parse `sparse_infill_flow_ratio` into `ExtrusionOptions` using the same `0..=2` bounded float contract as other Orca flow ratios.
- `crates/ares-core/src/extrusions.rs`: store the ratio and apply it only to `PrintPathRole::SparseInfill` in `ExtrusionOptions::extrusion_per_mm`.
- `crates/ares-core/src/options/tests/bridge_wiring.rs`: add focused option-to-extrusion tests.
- `crates/ares-core/src/pipeline/tests/sparse_infill_flow_ratio.rs`: add a G-code-facing pipeline regression proving sparse infill E deltas change while sparse infill path count does not.
- `crates/ares-core/src/pipeline/tests.rs`: register the new focused test module.

## Included Behavior

- Default ratio is `1.0`, preserving current sparse infill extrusion behavior when the option is absent.
- Values `0.0` and `2.0` are accepted.
- Invalid negative, above-range, non-numeric, NaN, and infinite values are rejected through `SliceOptions::extrusion_options`.
- Sparse infill extrusion per millimeter is multiplied by the ratio.
- Brim, bridge, internal bridge, perimeter, and skirt flow behavior remains unchanged.

## Deferred Behavior

- Orca `set_other_flow_ratios` gating is not introduced because Ares currently consumes analogous flow ratio options directly and does not yet model that upstream toggle.
- Other Orca flow ratio options such as `outer_wall_flow_ratio`, `inner_wall_flow_ratio`, `gap_fill_flow_ratio`, and support flow ratios remain outside this slice.
- Solid infill, support, gap fill, object flow ratio, filament flow ratio, and UI behavior remain outside this slice.

## Acceptance Criteria

- A red test first demonstrates `sparse_infill_flow_ratio` does not affect `SparseInfill` extrusion before implementation.
- After implementation, option-level tests prove parsing, bounds, default behavior, and role scoping.
- Pipeline-level tests prove generated G-code sparse infill E deltas scale with `sparse_infill_flow_ratio` without changing sparse infill geometry/path count.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and Rust file LOC checks pass.
