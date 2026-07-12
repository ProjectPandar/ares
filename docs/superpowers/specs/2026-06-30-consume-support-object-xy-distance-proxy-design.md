# Consume Support Object XY Distance Proxy Design

## Upstream Boundary

This slice ports the XY clearance role of OrcaSlicer's existing
`support_object_xy_distance` option into Ares' current rectangular support
proxy boundary. It does not implement full support contact generation or
arbitrary polygon clipping.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:995` declares
  `support_object_xy_distance` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5927-5936` defines
  `support_object_xy_distance` as a support-category millimeter option, with
  range `0..=10` and default `0.35`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:84` stores the value
  as `gap_xy` for classic support generation.
- `OrcaSlicer/src/libslic3r/Support/TreeSupportCommon.hpp:70-74` stores the
  same value as tree-support XY distance and derives an overhang separation
  from it.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:445,2730,3103` passes
  `m_support_params.gap_xy` into support-layer object trimming.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:3111-3190` offsets
  Z-overlapping object layer polygons by `gap_xy` while trimming non-raft
  support layers by object geometry, while non-overlapping and sharp-tail
  cases use fixed 0.2 mm offsets from
  `SupportMaterial.cpp:1364-1365`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1034-1038` uses
  `support_object_xy_distance` when constructing tree support preview collision
  data.

## Current Ares State

`ares-core` already registers `support_object_xy_distance` in
`options/registry/definitions/table/tail_support.rs`, and
`options/support_placement.rs` parses it with Orca's default `0.35` and
inclusive `0..=10` millimeter range. That runtime value is currently consumed
only by `consume_runtime()`, so valid values do not affect support proxy
geometry.

Ares does not yet generate Orca support contact polygons or separate support
layers. It has a rectangular support proxy stream in `LayerPrintPaths`:
existing `SupportMaterial` and `SupportMaterialInterface` paths may be closed
rectangles, and later finalization passes convert those rectangles into support
base/interface line paths, support ironing, toolpath moves, extrusion moves,
speed moves, diagnostics, and G-code.

The full pipeline already has rectangular object contours available at print
path finalization time through `LayerContours`. Direct unit tests that call
`finalize_print_paths()` do not currently pass contour context.

## Design

Add a bounded rectangular object-clearance pass for existing support proxy
rectangles.

Finalization API:

- Keep `finalize_print_paths(paths, options)` for tests and callers without
  contour context; it keeps today's no-object-clearance behavior.
- Add a contour-aware finalizer used by the main slicing pipeline and
  contour-aware tests.
- Parse `support_placement_options()` inside the contour-aware finalizer before
  disabled-support filtering, so invalid placement values still fail before any
  support proxy output is removed.

Print-path behavior:

- If contour context is absent, leave support proxy geometry unchanged.
- If `support_object_xy_distance == 0.0`, still trim direct overlap between a
  closed rectangular support proxy path and rectangular object contour bounds.
- For each non-raft support proxy layer, inflate same-layer rectangular object
  contour bounds by `support_object_xy_distance`. This is the slice's
  same-layer proxy for Orca's `is_layers_overlap()` gate.
- Clip closed rectangular `SupportMaterial` and `SupportMaterialInterface`
  proxy paths by subtracting those inflated rectangular object bounds.
- Preserve any remaining pieces as closed rectangular support proxy paths with
  the source role, extrusion role, closure, and print-path metadata preserved.
- Drop a support proxy rectangle if object clearance fully covers it.
- Preserve open paths, non-rectangular support paths, non-support paths,
  non-rectangular object contours, layer ids, and `print_z`.
- Skip layers whose `layer_id < raft_layers`. This is Ares' proxy
  approximation of Orca's z-based non-raft trimming guard in
  `SupportMaterial.cpp:3121-3123`, using the existing `raft_layers` layer-id
  semantics and preserving current raft proxy expansion behavior.
- Run the pass after `support_expansion`, `raft_expansion`, and
  `raft_first_layer_expansion`, and before support base/interface spacing and
  support ironing. The resulting clipped rectangles therefore drive emitted
  G-code coordinates.

Rectangular difference scope:

- This slice may split one support rectangle into up to four rectangular
  strips around the inflated object rectangle.
- Multiple rectangular object contours are applied sequentially to each
  remaining support piece.
- Exact ExPolygon boolean operations, holes, island ordering, path linking, and
  fill-engine parity are deferred.

## Included Behavior

- `support_object_xy_distance` remains parsed with Orca's default `0.35` and
  range `0..=10`.
- Full slicing pipeline output uses the configured value to keep existing
  rectangular support proxy paths away from same-layer rectangular object
  contours.
- `support_object_xy_distance = 0.0` removes direct overlap but adds no extra
  clearance.
- Larger configured values increase the cleared object area and can remove
  smaller support proxy rectangles entirely.
- Clipped `SupportMaterial` rectangles feed support base pattern spacing.
- Clipped `SupportMaterialInterface` rectangles feed support interface spacing
  and support ironing.
- Raft proxy layers remain governed by the existing raft expansion/density
  slices and are not clipped by object XY distance.
- Emitted support G-code coordinates change when a contour-aware pipeline has
  support proxy rectangles overlapping rectangular object contours.

## Deferred Behavior

- Full Orca support contact generation.
- Support layer storage, support/object Z-overlap scanning, and support
  generator invalidation parity.
- `support_object_first_layer_gap`.
- `support_on_build_plate_only`, `support_critical_regions_only`, and
  `support_remove_small_overhang`.
- `support_threshold_angle` and `support_threshold_overlap` overhang detection.
- Orca's full support/object Z-overlap scan across multiple object layers in
  `SupportMaterial.cpp:3156-3161`.
- Orca's `is_layers_overlap()` offset selection in
  `SupportMaterial.cpp:3163-3171`: because Ares does not yet have support-layer
  Z-overlap scanning, this proxy applies `support_object_xy_distance` to
  same-layer rectangular contours and diverges from Orca on non-overlapping
  layers where Orca uses fixed `no_overlap_xy_gap = 0.2` from
  `SupportMaterial.cpp:1365`.
- Orca's sharp-tail-specific fixed `sharp_tail_xy_gap = 0.2` behavior from
  `SupportMaterial.cpp:1364,3166-3169`.
- Non-rectangular ExPolygon offsetting, clipping, hole handling, simplification,
  sorting, and linking.
- Tree/organic support branching and collision geometry.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. The main slicing pipeline uses a contour-aware print-path finalizer.
2. Direct calls to the existing two-argument `finalize_print_paths()` preserve
   today's behavior when no layer-contour context is supplied.
3. In the contour-aware finalizer, invalid `support_object_xy_distance` values
   still return `SliceError::InvalidInput` containing
   `support_object_xy_distance` before disabled-support filtering can remove
   support paths.
4. With a rectangular support proxy path covering a same-layer rectangular
   object contour, the default `0.35` mm clearance clips support output away
   from the object contour.
5. With `support_object_xy_distance = 0.0`, overlapping support output is
   clipped to the object bounds with no extra clearance.
6. With a larger configured clearance, the clipped support output reflects the
   larger inflated object rectangle, or the support proxy path is dropped when
   no positive-area support pieces remain.
7. Non-rectangular support paths, open support paths, non-support paths, and
   non-rectangular object contours are unchanged by this proxy pass.
8. Layers whose `layer_id < raft_layers` are not clipped by
   `support_object_xy_distance`.
9. Clipped `SupportMaterialInterface` rectangles are visible before support
   interface spacing and support ironing.
10. Emitted G-code from a contour-aware pipeline changes support coordinates
    compared with `support_object_xy_distance = 0.0` or no contour context.
11. `docs/roadmap.md` records this source-cited proxy slice and names the
    deferred full support-generator behavior.

## Verification Plan

- Add a contour-aware finalization test module under
  `crates/ares-core/src/pipeline/tests/support_object_xy_distance_proxy.rs`.
- Add a G-code visibility test under
  `crates/ares-core/src/pipeline/tests/support_object_xy_distance_proxy_gcode.rs`.
- Reuse the existing `options/tests/support_placement.rs` parser coverage.
- Register the new pipeline test modules in
  `crates/ares-core/src/pipeline/tests.rs`.
- Update `docs/roadmap.md`.
- Run targeted tests:
  - `cargo nextest run -p ares-core support_object_xy_distance`
  - `cargo nextest run -p ares-core support_expansion support_raft_expansion support_base_pattern_spacing support_interface_spacing support_ironing`
  - `cargo nextest run -p ares-core support_enable support_placement`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines
