# Consume Brim Flow Ratio Design

## Goal

Make Ares consume OrcaSlicer's `brim_flow_ratio` option in concrete slicing output by scaling only brim extrusion `E` values, not by adding another metadata-only option.

## Upstream Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r` brim flow behavior:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:921` declares `PrintObjectConfig::brim_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1637-1645` defines `brim_flow_ratio` as a float with min `0`, max `2`, default `1`, and support-category UI metadata.
- `OrcaSlicer/src/libslic3r/Brim.cpp:837-873` emits brim extrusion entities with role `erBrim` and base `flow.mm3_per_mm()`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6409-6410` multiplies `_mm3_per_mm` by `m_config.brim_flow_ratio` when the path role is `erBrim`.
- `OrcaSlicer/src/libslic3r/Print.cpp:1958-1978` computes base brim flow width from initial layer, inner wall, or line width. That width-selection behavior is not part of this slice.

## Ares Destination Boundary

Implement the `GCode.cpp` role-specific multiplier in Ares' existing extrusion stage:

- Parse `brim_flow_ratio` through `SliceOptions::extrusion_options()`.
- Store the ratio in `ExtrusionOptions`.
- Apply it in `ExtrusionOptions::extrusion_per_mm()` only when `role == PrintPathRole::Brim`.
- Preserve existing brim geometry generation, brim width fallback, toolpath move ordering, and G-code comment formatting.

## Requirements

- `brim_flow_ratio` defaults to `1.0`.
- Values must be finite and in Orca's configured range `0.0..=2.0`.
- A ratio of `0.0` is accepted and produces zero brim extrusion for printed brim moves, matching Orca's option minimum.
- A ratio of `2.0` is accepted.
- Invalid values, including negative values, values above `2.0`, strings that are not numeric, `NaN`, and infinities, must return `SliceError::InvalidInput` through the existing option parsing path.
- The ratio must not affect skirt, perimeter, bridge, or infill extrusion.
- Existing bridge flow behavior must remain independent.
- End-to-end pipeline output must show changed `;EXTRUSION:print:brim` extrusion deltas when `brim_flow_ratio` changes, with unchanged brim path count for the same geometry input. Because Ares emits cumulative `E`, tests must compare the delta from the previous extrusion position, not the absolute first brim `E` value.

## Deferred Behavior

This slice does not implement:

- `print_flow_ratio`, filament flow ratio, object flow ratio, or first-layer flow ratio composition.
- `brim_use_efc_outline`.
- Orca `Print::brim_flow()` width derivation from `initial_layer_line_width`, `inner_wall_line_width`, and `line_width`.
- New brim geometry modes, brim ears, painted brim behavior, or extruder selection behavior.
- Any new crate, dependency, feature flag, or Ares-owned pipeline design.

## Docs Impact

No user-facing option catalog or registry metadata docs need to change in this slice because `brim_flow_ratio` is already registered with Orca-sourced metadata. The new documentation artifact for this work is this source-cited implementation spec plus the implementation plan.

## Test Strategy

- Add a focused option wiring test proving parsed `brim_flow_ratio` reaches `ExtrusionOptions`, scales `PrintPathRole::Brim`, and does not scale `PrintPathRole::ExternalPerimeter`.
- Add validation tests for `0.0`, `2.0`, negative, above-max, non-numeric, `"NaN"`, `"inf"`, and `"-inf"` inputs.
- Add an extrusion unit test proving direct `ExtrusionOptions` brim scaling while preserving non-brim roles.
- Add a pipeline/G-code regression test comparing the first emitted `;EXTRUSION:print:brim:` delta from the prior cumulative `E` position for the same rectangular input at different `brim_flow_ratio` values.

## Acceptance Criteria

- The first brim extrusion delta from the prior cumulative `E` position changes proportionally with `brim_flow_ratio` for identical brim geometry.
- Non-brim extrusion math remains unchanged except for existing bridge-specific behavior.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- Touched Rust source files stay at or below 400 LOC.
