# Consume Overhang Speed Bands Design

## Goal

Consume the existing `overhang_1_4_speed`, `overhang_2_4_speed`, and `overhang_3_4_speed` options into concrete Ares overhang-perimeter speed behavior instead of treating only `overhang_4_4_speed` as the single runtime overhang speed.

## Source Boundary

Upstream source slice:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1171-1175`: `enable_overhang_speed`, `overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, and `overhang_4_4_speed` option tuple fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1500-1577`: option definitions, defaults, and UI descriptions for overhang wall speed bands.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6587-6641`: G-code generation path that builds `dynamic_overhang_speeds` and passes them to `ExtrusionQualityEstimator::estimate_extrusion_quality`.
- `OrcaSlicer/src/libslic3r/GCode/ExtrusionProcessor.hpp:348-452`: estimator logic that maps overhang distance, line width, and configured overlap/speed sections to the speed used for extrusion segments.

Rust destination boundary:

- `crates/ares-core/src/options/overhang_speed.rs`: parse all four overhang speed bands as `FloatOrPercent` over the outer wall speed and preserve Orca's current gate semantics.
- `crates/ares-core/src/options/speed.rs` and `crates/ares-core/src/speeds/config.rs`: carry the parsed band table in `SpeedOptions`.
- `crates/ares-core/src/perimeters.rs` and `crates/ares-core/src/print_paths.rs`: carry the existing Ares overhang unsupported-span estimate from overhang perimeter detection into print paths.
- `crates/ares-core/src/moves.rs`, `crates/ares-core/src/extrusions.rs`, and `crates/ares-core/src/speeds/*`: carry that estimate into speed generation and select the concrete overhang perimeter speed.

## Included Behavior

This slice keeps Ares' current overhang detection model and makes the existing speed bands executable within that model:

- `enable_overhang_speed = false` keeps existing bridge-speed fallback for `PrintPathRole::OverhangPerimeter`.
- Missing or zero speed-band values keep existing defaults: `overhang_1_4_speed`, `overhang_2_4_speed`, and `overhang_3_4_speed` act as no explicit slowdown, while `overhang_4_4_speed` continues to default to bridge-speed fallback when not configured.
- Numeric and percent `FloatOrPercent` forms are parsed for all four speed-band keys against `outer_wall_speed`, matching the existing `overhang_4_4_speed` parser style.
- For non-first-layer overhang perimeter print moves with an Ares `unsupported_span_mm`, speed selection uses `unsupported_span_mm / external_line_width` as the Ares approximation of upstream overhang distance relative to path width:
  - ratio `<= 0.25`: use `overhang_1_4_speed` when configured.
  - ratio `<= 0.50`: use `overhang_2_4_speed` when configured.
  - ratio `<= 0.75`: use `overhang_3_4_speed` when configured.
  - ratio `> 0.75`: use `overhang_4_4_speed` when configured.
- If the selected band is not configured or resolves below Orca's meaningful speed threshold (`< 0.5 mm/s`), Ares keeps the current overhang speed for the path instead of creating a new slowdown.
- Selected band speeds are clamped to no higher than the current overhang perimeter base speed for that move, mirroring Orca's estimator clamp that prevents overhang speed from increasing after other speed caps.
- Existing first-layer behavior remains: first-layer overhang perimeters use first-layer wall speed and ignore dynamic overhang speed bands.
- Existing volumetric caps, extrusion-rate smoothing, layer-time slowdown, slow-down-layers interpolation, small-perimeter exclusion, and G-code emission remain in their current order except that overhang speed bands now influence the role speed before later caps/slowdowns.

## Deferred Behavior

This slice deliberately does not port the full Orca estimator:

- No `ExtrusionQualityEstimator` equivalent, no AABB-tree previous-layer distance calculation, no per-point overlap calculation, and no path subdivision at speed-section boundaries.
- No curled-line slowdown or `slowdown_for_curled_perimeters`.
- No bridge-infill dynamic speed bands beyond the current Ares `Bridge` and `InternalBridge` role behavior.
- No sloped/scarf interaction, raft/object-layer checks, multi-region/object state, or current-object estimator state.
- No G-code post-processing markers beyond the current Ares generated speed comments.

## Test Requirements

Use `cargo nextest run`, not `cargo test`.

Required RED/GREEN tests:

- Option parser test proving `overhang_1_4_speed`, `overhang_2_4_speed`, and `overhang_3_4_speed` parse numeric and percent values and reject invalid values.
- Speed-generation test proving an overhang move with unsupported-span metadata chooses the expected 1/4, 2/4, 3/4, and 4/4 band speeds.
- Pipeline/G-code test proving a generated unsupported second-layer overhang with configured lower-band speed reaches `;SPEED:print:overhang_perimeter:*` at the selected feedrate.
- Regression test proving `enable_overhang_speed = false` and first-layer overhangs preserve existing behavior.

Full verification before commit:

- `cargo fmt --check`
- focused `cargo nextest run -p ares-core overhang_speed`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Rust touched-file LOC guard, keeping every touched Rust file at or below 400 LOC.

## Documentation

Update `docs/roadmap.md` with a short completed-slice note that names the same upstream source boundary and the deferred full estimator work.
