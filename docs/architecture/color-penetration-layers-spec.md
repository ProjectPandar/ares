# Color penetration layers

## Status and source boundary

This document defines the Class A P1 source-cited rewrite slice for painted
color penetration. It ports the upstream function; it is not a new Ares-owned
painting pipeline.

The authoritative source is upstream commit `345efec64`, at
`OrcaSlicer/src/libslic3r/MultiMaterialSegmentation.cpp`:

- `color_penetration_layers`: `:1241-1244`;
- `segmentation_top_and_bottom_layers`: `:1247-1582` (the complete function,
  including its return at `:1581-1582`);
- its caller in `segmentation_by_painting`: `:2449-2452`;
- combination with side-painted segmentation through
  `merge_segmented_layers`: call site `:2455`, implementation `:1999-2088`.

The configuration owners are `PrintConfig.hpp:1452,1561` and
`PrintConfig.cpp:1545-1546,8165-8166`. Ares already registers the corresponding
`bottom_color_penetration_layers` and `top_color_penetration_layers` options;
this slice ports their use in the segmentation function.

## Actual Ares gap

Ares has no `segmentation_top_and_bottom_layers` function. The existing
`project_slice/region_slices/painted/top_bottom.rs` is only partial related
scaffolding: it performs a simpler slab/embedded-shell path, does not implement
the complete upstream function, and currently resolves shell counts without the
upstream positive-penetration/zero-means-shell rule. Therefore this slice is the
full upstream function boundary `:1247-1582`, not merely a missing scalar
consumer.

The Ares wiring destination is the existing painted segmentation path:
`project_slice/load_stages.rs:107-131` constructs painted region state,
`project/region_graph/painted.rs` carries painted regions, and
`project_slice/region_slices/painted` owns the downstream region operations.
The implementation must replace or extend the temporary `top_bottom.rs`
compatibility shell at that boundary rather than create a second pipeline.

## Upstream call and merge contract

The upstream call is in `segmentation_by_painting`, after side-painted lines
have been projected, post-processed, colorized, converted to per-island
regions, and optionally cut for interlocking (`:2200-2447`). When
`include_top_and_bottom_layers == IncludeTopAndBottomLayers::Yes`, Native calls
`segmentation_top_and_bottom_layers(...)` at `:2449-2452`.

The returned array is indexed by extruder/color and layer. Native then passes
it, together with the side-painted `segmented_regions`, to
`merge_segmented_layers` at `:2455`. `merge_segmented_layers` merges ordinary
side regions and top/bottom regions, removes overlaps in color order, and
chooses the top/bottom result where appropriate (`:1999-2088`). The Ares port
must preserve this call ordering and result shape: top/bottom regions are not a
separate output and must be combined with side-painted segmentation before the
final multi-material result is returned.

## Helper inventory and port responsibility

The function calls the following upstream helpers. “Existing” means Ares has a
usable equivalent, not necessarily the same signature. “Port together” means
the helper behavior or an adapter is part of the relevant implementation slice.

### Existing Ares geometry/slicing building blocks

- Boolean union/intersection/difference and offsets: `crates/ares-core/src/geometry.rs`
  and `crates/ares-core/src/geometry/clipper/` (`union_ex`, `intersection_ex`,
  `difference_ex`, `offset_expolygons`).
- Morphological opening: `crates/ares-core/src/geometry/clipper/offset/expolygon.rs`
  (`opening_ex`).
- Polygon area filtering: the geometry `ExPolygon::area` API and existing
  filtering patterns; the exact `0.1 mm²` threshold remains part of this port.
- Mesh slab slicing: `crates/ares-core/src/mesh_slicer/slabs.rs`
  (`slice_mesh_slabs`) and its painted adapter
  `crates/ares-core/src/project_slice/region_slices/painted/slabs.rs`.
- Painted top/bottom records and downstream application:
  `crates/ares-core/src/project_slice/region_slices/painted/top_bottom.rs` and
  `apply.rs`.
- Existing loaded MMU facet annotations:
  `crates/ares-core/src/project/domain/mesh.rs` and
  `crates/ares-core/src/project/facets_annotation.rs`.

These existing pieces still need boundary adapters where Native uses
`PrintObject`, `Layer`, `ExPolygons`, transforms, or C++ coordinate scaling.
Their existence does not mean the complete upstream behavior is already
implemented.

### Helpers/behavior requiring a port or a coordinated adapter

- `zs_from_layers` (`:1266`) needs an Ares layer-height/slice-Z adapter at the
  new segmentation boundary; no same-named Ares helper exists.
- `is_volume_sinking` (`:1280-1287`) needs to be ported against Ares mesh
  vertices and transforms; no same-named Ares helper exists.
- The sinking branch's `slice_mesh` call and bottom-slice union
  (`:1291-1318`) need to be implemented or adapted; Ares has slab slicing but
  no same-named general `slice_mesh` equivalent at this boundary.
- Strict facet-state extraction and model-volume traversal
  (`:1270-1278`) need an adapter from Ares `FacetsAnnotation`; the carrier
  exists, but the complete per-volume/per-color extraction contract does not
  exist as this function.
- `tile_expolygons`, bounding-box clipping, and the tiled
  `project_to_shells` algorithm (`:1455-1498`) need a coordinated port. Ares
  has `clip_clipper_polygons_with_subject_bbox` in
  `geometry/bbox_clip.rs`, but no same-named `tile_expolygons` helper.
- `resolve_outer_wall_line_width` (`:1221-1238`) and the exact
  `LayerColorStat` calculation (`:1411-1440`) need to be mapped to Ares region,
  nozzle, gap-infill, and flow configuration. The existing `top_bottom.rs`
  `stat` is not sufficient because it does not implement the penetration scalar
  rule or all Native outer-wall semantics.
- Ordered shell projection gathering and per-color/per-layer merge
  (`:1528-1582`) must be ported as one result-shape contract. Existing boolean
  helpers are reusable, but no Ares function currently provides this complete
  merge.
- The final `merge_segmented_layers` wiring at `:2455` must be mapped to
  Ares's side-painted region records; it is not part of the standalone
  top/bottom geometry result.

## Ordered implementation slices

The full function is large enough to split at natural internal boundaries. The
slices are sequential and together cover exactly `:1247-1582` plus the caller
wiring required to make the result observable.

### Slice 1: collect and normalize painted top/bottom surfaces

Port `:1247-1409`: penetration-depth resolution, layer-Z setup, model-volume
facet extraction, sinking/non-sinking slab slicing, top/bottom raw collection,
small-polygon filtering, and adjacent-layer occlusion trimming.

The first verifiable witness is a public runtime-derived painted MMU fixture
with one painted top or bottom patch and two controls: explicit positive
penetration and zero penetration falling back to the corresponding shell
count. The witness must compare the complete Native/Ares result in official and
seeded modes and show the projected color reaches the expected adjacent shell
layer without asserting stored Native values.

### Slice 2: color statistics and shell projection

Port `:1411-1526`: `LayerColorStat`, color-specific region matching, extrusion
width/spacing and small-region filtering, top/bottom penetration shell target
selection, and `project_to_shells` including its tiled clipping and early-stop
behavior.

This slice consumes Slice 1's raw surfaces and must preserve top-versus-bottom
direction and the Native nearest-first shell order.

### Slice 3: ordered merge and side-segmentation wiring

Port `:1528-1582` and wire the result into the `segmentation_by_painting`
call/merge wiring at `:2449-2455`, with the corresponding
`merge_segmented_layers` behavior at `:1999-2088`. This includes per-color
union, removal of shell areas already covered by painted regions, color-order
overlap trimming, and final combination with side-painted regions.

## Deferred behavior

This slice defers GUI controls and labels, new paint annotation formats,
transport changes, fuzzy-skin segmentation, unrelated segmentation modes,
unrelated interlocking-cut behavior, and performance changes that alter Native
source ordering or geometry semantics. `Preset.cpp` default-extension behavior
is also outside this slice. Existing Ares painted modules remain compatibility
shells only where they do not yet match the cited upstream boundary.

## Verification constraints

Implementation verification must retain the existing complete-output
comparator, public-only witnesses, official and seeded modes, and all output
residuals. A settings echo alone is not sufficient: the witness must enter the
painted segmentation path and observe the merged result after the equivalent of
Native `merge_segmented_layers`.
