# Consume make_overhang_printable Rectangles Design

## Source Boundary

- Upstream option tuple: `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1199`
  `((ConfigOptionBool, make_overhang_printable))`.
- Upstream defaults and constraints:
  - `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4850-4855` defines `make_overhang_printable`, default `false`.
  - `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4857-4867` defines `make_overhang_printable_angle`, min `0`, max `90`, default `55`.
  - `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4869-4878` defines `make_overhang_printable_hole_size`, min `0`, default `0`.
- Upstream behavior owner: `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:1397-1510`
  `PrintObject::apply_conical_overhang()`. Orca walks layers from top to bottom, converts the configured angle from
  degrees to radians, offsets each upper layer inward by `tan(angle_radians) * layer_height`, then unions that printable
  projection into the layer below when the region enables `make_overhang_printable`.

## Ares Destination Boundary

- Add a platform-neutral contour-stage transform in `crates/ares-core/src/contours/overhang_printable.rs`.
- Wire it between `stitch_layer_slices` and perimeter/infill/brim/skirt generation in `crates/ares-core/src/pipeline.rs`.
- Parse the three options through `SliceOptions::perimeter_options()` / `PerimeterOptions` because the current Ares
  perimeter/contour decisions already consume PrintRegion-style wall options there.

## Included Behavior

1. `make_overhang_printable` defaults to `false`; when false, Ares preserves existing contours exactly.
2. When enabled, Ares applies a source-cited rectangular subset of Orca's conical overhang transform:
   - Only layers whose current and upper contours are single or multiple axis-aligned rectangles are changed.
   - The transform walks from the top layer down.
   - For each upper rectangle, compute
     `shrink_mm = make_overhang_printable_angle.to_radians().tan() * layer_height_mm`.
   - If the angle is `90`, preserve geometry.
   - Otherwise shrink the upper rectangle by that distance on all sides and union the resulting rectangle into the
     layer below when it has positive area.
   - Existing lower rectangles that already overlap remain present; the new projection is added as another contour.
   - Test fixture coordinate contract: with layer height `0.2`, angle `45`, and upper rectangle `(10, 0) -> (14, 4)`,
     the lower layer receives a new rectangle `(10.2, 0.2) -> (13.8, 3.8)` because
     `tan(45deg) * 0.2 = 0.2`.
3. Generated perimeters, print paths, moves, and G-code must reflect the new lower-layer contour, proving this is
   geometry/slicing behavior rather than option metadata.
4. `make_overhang_printable_angle` accepts numeric JSON values or numeric strings in `0..=90`, defaulting to `55`.
5. `make_overhang_printable_hole_size` accepts non-negative numeric JSON values or numeric strings, defaulting to `0`.
   It is parsed and validated in this slice but complex hole-preservation behavior is deferred.

## Deferred Behavior

- Full polygon union/difference, per-region ownership, overlap removal between regions, and hole protection from
  `PrintObject::apply_conical_overhang()` are deferred until Ares has polygon-with-holes and region ownership at this
  boundary.
- Non-rectangular contours are preserved unchanged.
- `hole_to_polyhole` and `hole_to_polyhole_*` remain separate upstream slices.
- This slice does not add dependencies or introduce native file I/O, UI, OpenGL, terminal behavior, or WASM-incompatible
  APIs.

## Acceptance Criteria

- Focused RED/GREEN test:
  `cargo nextest run -p ares-core make_overhang_printable`
  initially fails after adding tests because the lower layer does not receive the shrunken upper projection.
- After implementation:
  - Disabled/default `make_overhang_printable` preserves the lower contour list, perimeter count, print-path count,
    toolpath print move count, extrusion print move count, and formatted G-code for the focused two-layer fixture.
  - Enabled `make_overhang_printable` on a two-layer rectangular case adds a lower-layer contour derived from the upper
    layer with exact points `(10.2, 0.2)`, `(13.8, 0.2)`, `(13.8, 3.8)`, `(10.2, 3.8)`.
  - That enabled fixture increases lower-layer perimeter count, print-path count, toolpath print move count, extrusion
    print move count, and formatted G-code `;PERIMETER:` / `;PRINT_PATH:` output when `gcode_comments` is explicitly
    enabled.
  - `make_overhang_printable_angle = 90` preserves geometry.
  - Invalid angle and hole-size values produce `SliceError::InvalidInput` through `perimeter_options()`.
- Full verification before commit:
  - `cargo fmt --check`
  - focused nextest for `make_overhang_printable`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files stay at or under 400 LOC.

## Safety And Rollback

- The transform is gated by an explicit boolean defaulting false.
- The new code is pure Rust data transformation over existing `LayerContours` and `Point2` values.
- Rollback is deleting the new transform module, removing the pipeline call, and removing the focused tests/spec/plan.

## Docs Impact

- This spec and the implementation plan are the required documentation updates for the slice.
- No `docs/roadmap.md` update is required because this is not adding a new milestone; it consumes an existing
  source-cited milestone option into concrete runtime behavior.
- No architecture ADR is required because the change follows the existing contour/perimeter pipeline boundary and does
  not introduce a new crate, dependency, cross-platform constraint, or irreversible architectural decision.
