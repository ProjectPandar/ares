# Consume Aligned Rectilinear Infill Angle Design

## Goal

Implement a source-cited OrcaSlicer rewrite slice that makes `sparse_infill_pattern = alignedrectilinear` differ from ordinary `rectilinear` sparse infill in generated Ares infill toolpaths. The concrete behavior is layer-to-layer infill angle selection: ordinary line-based infill alternates by 90 degrees on odd layers, while aligned rectilinear keeps the configured sparse infill direction on every layer.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-90` declares `InfillPattern`, including `ipRectilinear`, `ipAlignedRectilinear`, `ipZigZag`, `ipLine`, and `ipGrid`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2928-2985` defines `sparse_infill_pattern`, includes `alignedrectilinear`, and defaults sparse infill to `ipCrossHatch`.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-54` dispatches `ipRectilinear` to `FillRectilinear`, `ipAlignedRectilinear` to `FillAlignedRectilinear`, `ipLine` to `FillLine`, and `ipGrid` to `FillGrid`.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.hpp:211` defines the default layer angle as `0` on even layers and `PI/2` on odd layers unless the angle is fixed.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:38-46` defines `FillAlignedRectilinear` as a `FillRectilinear` variant whose `_layer_angle` always returns `0`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:67-77` defines `FillGrid` as a `FillRectilinear` variant whose `_layer_angle` always returns `0`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:645-651` repeats the same distinction for split solid fill: patterns other than `ipAlignedRectilinear` alternate by 90 degrees based on layer index.

## Ares Destination Boundary

- `crates/ares-core/src/options/infill.rs` already parses `alignedrectilinear`, `rectilinear`, `line`, `grid`, `zigzag`, and `crosshatch` into `InfillPattern`.
- `crates/ares-core/src/infills.rs` owns sparse infill line generation from `LayerContours` and `InfillOptions`.
- This slice changes only sparse infill pass-angle selection in `generate_infills`; it does not add new infill pattern families or new surface classification.

## Included Behavior

- `InfillPattern::Rectilinear` alternates sparse infill angle by layer id: even layers use `infill_direction`, odd layers use `infill_direction + 90` modulo 360.
- `InfillPattern::Line`, `InfillPattern::ZigZag`, and the current `CrossHatch` scaffold follow the same layer-angle alternation because Ares currently models them as single-pass line-based sparse infill.
- `InfillPattern::AlignedRectilinear` keeps `infill_direction` on every layer.
- `InfillPattern::Grid` keeps its existing two perpendicular passes on every layer and does not acquire an additional odd-layer rotation.
- Density, line width, minimum sparse infill area, hole clipping, segment sorting, zigzag alternating segment direction, volumetric speed, and G-code emission remain unchanged except for coordinates caused by the selected angle.

## Deferred Behavior

- Full Orca `FillLine` oscillation/interconnection behavior remains out of scope.
- Full Orca `FillCrossHatch` geometry remains out of scope; Ares continues using its current scaffold until a separate source-cited crosshatch slice is planned.
- Solid infill patterns, top/bottom surface patterns, monotonic infill, bridge infill, support infill, and `align_infill_direction_to_model` remain out of scope.
- No new crates, dependencies, or Ares-owned slicing pipeline concepts are introduced.

## Acceptance Criteria

- On a square layer with `infill_direction = 0`, `Rectilinear` generates vertical sparse lines on layer 0 and horizontal sparse lines on layer 1.
- On the same geometry, `AlignedRectilinear` generates vertical sparse lines on both layer 0 and layer 1.
- `Line`, `ZigZag`, and `CrossHatch` sparse infill use the same odd-layer 90-degree angle alternation as ordinary rectilinear within Ares' current supported scaffold.
- `Grid` still generates both vertical and horizontal passes on each layer and is not rotated into duplicate equivalent behavior on odd layers.
- End-to-end G-code for `sparse_infill_pattern = rectilinear` exposes changed second-layer sparse infill coordinates, while `alignedrectilinear` keeps the same layer direction.
- Existing supported sparse infill pattern parsing remains valid and known unimplemented patterns remain rejected.
- No Rust file under `crates/` exceeds 400 LOC.
