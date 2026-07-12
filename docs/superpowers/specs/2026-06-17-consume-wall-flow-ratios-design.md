# Consume Wall Flow Ratios Design

## Goal

Consume OrcaSlicer `outer_wall_flow_ratio` and `inner_wall_flow_ratio` in Ares extrusion generation so already registered wall flow options alter external and internal perimeter E output.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1324-1332` registers `outer_wall_flow_ratio` as a float with min `0`, max `2`, and default `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1334-1342` registers `inner_wall_flow_ratio` as a float with min `0`, max `2`, and default `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1216-1217` includes both wall flow ratios in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6416-6419` multiplies material by `outer_wall_flow_ratio` for `erExternalPerimeter` and by `inner_wall_flow_ratio` for `erPerimeter`.

## Ares Destination Boundary

- `crates/ares-core/src/options/flow_ratios.rs`: parse both wall flow ratios with the same absent-default, numeric-or-string, finite, inclusive `0..=2` contract as the already consumed `sparse_infill_flow_ratio`.
- `crates/ares-core/src/options.rs`: wire parsed wall flow ratios into `ExtrusionOptions` through a single helper call so the file remains at or below 400 LOC.
- `crates/ares-core/src/extrusions.rs`: store wall flow ratios and apply them only to `PrintPathRole::ExternalPerimeter` and `PrintPathRole::InternalPerimeter`.
- `crates/ares-core/src/options/tests/bridge_wiring.rs`: add option-to-extrusion tests for default behavior, bounds, invalid values, and role scoping.
- `crates/ares-core/src/pipeline/tests/wall_flow_ratios.rs`: add a G-code-facing pipeline regression proving wall flow ratios change external/internal perimeter E deltas without changing perimeter path count.
- `crates/ares-core/src/pipeline/tests.rs`: register the focused test module.

## Included Behavior

- Omitted wall flow ratio options default to `1.0`, preserving current perimeter extrusion behavior.
- Values `0.0` and `2.0` are accepted for both options.
- Negative, above-range, non-numeric, NaN, and infinite values are rejected through `SliceOptions::extrusion_options`.
- `outer_wall_flow_ratio` scales only `PrintPathRole::ExternalPerimeter`.
- `inner_wall_flow_ratio` scales only `PrintPathRole::InternalPerimeter`.
- Brim, bridge, internal bridge, skirt, and sparse infill flow behavior remains unchanged.

## Deferred Behavior

- Orca `set_other_flow_ratios` gating is not introduced in this slice; Ares currently consumes analogous flow-ratio options directly and does not yet model the upstream toggle.
- First-layer, overhang, internal solid infill, gap fill, support, object flow ratio, filament flow ratio, and UI behavior remain outside this slice.
- Geometry generation remains unchanged; only role-scoped extrusion amount changes.

## Acceptance Criteria

- Add wall flow ratio option and pipeline regression tests before implementation, observe them fail against the current code, then preserve final passing tests that prove the intended scaling and unchanged path counts.
- After implementation, option-level tests prove parsing, bounds, default behavior, invalid values, and role scoping.
- Pipeline-level tests prove generated G-code external/internal perimeter E deltas scale with configured wall flow ratios while perimeter geometry/path count remains unchanged.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and Rust file LOC checks pass.
