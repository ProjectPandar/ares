# Consume Support Object First Layer Gap Proxy Design

## Upstream Boundary

This slice ports the first-layer XY clearance role of OrcaSlicer's
`support_object_first_layer_gap` option into Ares' current rectangular support
proxy boundary. It builds directly on the existing
`support_object_xy_distance` proxy slice and does not implement support contact
generation, raft layer generation, or tree-support collision geometry.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:996` declares
  `support_object_first_layer_gap` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5938-5947` defines
  `support_object_first_layer_gap` as a support-category millimeter option,
  with range `0..=10` and default `0.2`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:84-85,240-241`
  stores `support_object_xy_distance` as `gap_xy` and
  `support_object_first_layer_gap` as `gap_xy_first_layer` for normal support.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:286-288,376-388`
  trims first-layer support column bases against first object layer slices
  offset by `gap_xy_first_layer`, including the no-raft first-layer expansion
  path.
- `OrcaSlicer/src/libslic3r/Support/TreeSupportCommon.hpp:70-74` stores the
  first-layer support/object distance for tree support data.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2082-2084,2356-2361`
  applies `support_object_first_layer_gap` to tree/hybrid first-layer
  collision and trimming paths.

## Current Ares State

`ares-core` already registers `support_object_first_layer_gap` in
`options/registry/definitions/table/tail_support.rs`, and
`options/support_placement.rs` parses it with Orca's default `0.2` and
inclusive `0..=10` millimeter range. It is currently consumed only by
`consume_runtime()` and validation tests.

The previous `support_object_xy_distance` proxy slice added a contour-aware
print-path finalizer and rectangle-only clipping for closed rectangular
`SupportMaterial` and `SupportMaterialInterface` paths against same-layer
rectangular object contours. That pass currently uses
`support_object_xy_distance` for every non-raft layer, including layer `0`.

Ares still does not have Orca support-layer storage, support contact polygons,
tree support collision data, or true raft-support generation.

## Design

Extend the existing rectangle-only support/object XY clipping pass so first
print-layer support proxy geometry uses `support_object_first_layer_gap`.

Print-path behavior:

- Keep the no-context `finalize_print_paths(paths, options)` path unchanged.
- In the contour-aware finalizer, parse `support_placement_options()` before
  disabled-support filtering as today, so invalid
  `support_object_first_layer_gap` values still fail before support proxy paths
  can be removed.
- Pass both `object_xy_distance_mm()` and `object_first_layer_gap_mm()` into the
  existing rectangle-only clipping pass.
- For non-raft layer `0`, inflate same-layer rectangular object contour bounds
  by `support_object_first_layer_gap`.
- For other non-raft layers, continue inflating object contour bounds by
  `support_object_xy_distance`.
- Keep layers whose `layer_id < raft_layers` skipped by support/object XY
  clipping. This preserves the existing Ares raft proxy boundary and avoids
  modeling Orca raft/contact layer generation in this slice.
- Preserve all existing rectangular proxy limits: only closed rectangular
  `SupportMaterial` and `SupportMaterialInterface` paths are clipped; open,
  non-rectangular, non-support, and non-rectangular contour inputs remain
  unchanged.
- Keep the pass after `support_expansion`, `raft_expansion`, and
  `raft_first_layer_expansion`, and before support base/interface spacing,
  support ironing, and G-code emission.

## Included Behavior

- `support_object_first_layer_gap` remains parsed with Orca's default `0.2` and
  range `0..=10`.
- A contour-aware layer `0` rectangular support proxy path overlapping a
  rectangular object contour is clipped using the first-layer gap instead of
  `support_object_xy_distance`.
- Setting `support_object_first_layer_gap = 0.0` clips direct first-layer
  overlap without extra first-layer clearance.
- Setting a larger first-layer gap increases the first-layer cleared object
  area and can drop fully covered first-layer support rectangles.
- Upper non-raft layers still use `support_object_xy_distance`.
- Raft proxy layers still skip support/object XY clipping.
- Emitted support G-code coordinates change when a contour-aware first-layer
  support proxy path overlaps rectangular object contours and the first-layer
  gap is configured differently.

## Deferred Behavior

- Full Orca support contact generation.
- True first support layer detection independent of `layer_id == 0`.
- Support layer storage, support/object Z-overlap scanning, and support
  generator invalidation parity.
- Orca raft contact/base/interface layer generation and first-layer raft
  trimming parity.
- Tree/organic support collision geometry and hybrid tree first-layer trimming.
- Non-rectangular ExPolygon offsetting, clipping, hole handling,
  simplification, sorting, and linking.
- `no_overlap_xy_gap`, `sharp_tail_xy_gap`, and full `is_layers_overlap()`
  offset selection.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. The contour-aware finalizer still uses a rectangle-only support/object XY
   clipping pass.
2. Direct calls to the existing two-argument `finalize_print_paths()` preserve
   today's no-contour behavior.
3. Invalid `support_object_first_layer_gap` values return
   `SliceError::InvalidInput` containing `support_object_first_layer_gap` before
   disabled-support filtering can remove support paths.
4. On non-raft layer `0`, default values clip first-layer support with `0.2` mm
   clearance, not the ordinary `0.35` mm `support_object_xy_distance`.
5. On non-raft layer `0`, `support_object_first_layer_gap = 0.0` clips direct
   overlap without extra first-layer clearance.
6. On non-raft layer `0`, a larger `support_object_first_layer_gap` clips more
   aggressively or drops fully covered support rectangles.
7. On upper non-raft layers, `support_object_xy_distance` continues to control
   the clipping distance.
8. Layers whose `layer_id < raft_layers` remain unclipped by support/object XY
   distance and first-layer gap.
9. Clipped first-layer `SupportMaterialInterface` rectangles remain visible
   before support interface spacing and support ironing.
10. Emitted G-code from a contour-aware first-layer support proxy changes when
    `support_object_first_layer_gap` changes.
11. `docs/roadmap.md` records this source-cited proxy slice and names the
    deferred full support-generator behavior.

## Verification Plan

- Extend the existing contour-aware support/object XY proxy tests under
  `crates/ares-core/src/pipeline/tests/support_object_xy_distance_proxy.rs`.
- Extend or add G-code visibility coverage under
  `crates/ares-core/src/pipeline/tests/support_object_xy_distance_proxy_gcode.rs`.
- Reuse the existing `options/tests/support_placement.rs` parser coverage.
- Update `docs/roadmap.md`.
- Run targeted tests:
  - `cargo nextest run -p ares-core support_object`
  - `cargo nextest run -p ares-core support_expansion support_raft_expansion support_base_pattern_spacing support_interface_spacing support_ironing`
  - `cargo nextest run -p ares-core support_enable support_placement`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines
