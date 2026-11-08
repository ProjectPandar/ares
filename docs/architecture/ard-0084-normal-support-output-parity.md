# ARD0084 — Normal support complete-output parity

Status: Normal and Tree integration implemented; final combined merge verification open.

## Upstream owner and Rust destination

Rewrite `PrintObject.cpp::generate_support_material` and
`PrintObject::_generate_support_material`,
`Support/SupportMaterial.cpp::PrintObjectSupportMaterial::generate`,
`Support/SupportLayer.hpp::SupportGeneratorLayer`, and the interface, layer
and toolpath generation in `Support/SupportCommon.cpp` into
`ares-core/src/project_slice/support_material/`.
The input adapter runs after object fills are prepared and before extrusion
island ownership moves. Generated layers remain owned by the prepared object;
generation errors use the existing predecessor-disposal path.

`GCode.hpp::LayerToPrint`, `GCode.cpp::collect_layers_to_print` and
`GCode::extrude_support` own the rewrite into `gcode_emit/layers/` and
`gcode_emit/motion/support.rs`. This is not an Ares-designed support pipeline.

## Non-negotiable decisions

- Generate actual support paths; headers alone cannot establish parity.
- Pair object/support layers at upstream EPSILON, average paired Z, and merge
  plate groups at their minimum/maximum midpoint. Retain actual print Z.
  Raw ToolOrdering counts use exact unique Z followed by adjacent EPSILON
  differences, not the paired-group algorithm.
- Retain installed support layers even when they have no toolpaths. Empty
  tool ordering returns empty G-code before mutating emission state
  (`GCode.cpp:4580-4582`), with the raw last-layer cooling-flush flag. The processor
  header counts emitted layer tags; progress and tail retain raw counts.
- Keep active native layer IDs separate from merged chunk indices. Retain
  Path, NoSort, MultiPath, role, orientation, joined wipe and ironing-last
  behavior; support is not object infill.
- Keep active-support travel containment separate from planner geometry
  (`GCode.cpp:5339-5345,5386-5440,7618-7632`); emit paired labels once.
- Simplify retained support entities according to `Layer.cpp:329-375` and
  `PrintObject.cpp:951-970`. Arc fitting uses upstream 0.0375 mm support
  resolution; linear simplification uses configured resolution.
- Preserve native undefined bounding-box semantics for empty polygons, not
  MAX/MIN sentinels or saturating arithmetic. Preserve scaled-width integer
  truncation and native Flow narrowing before downstream calculations.
- Support connection cost thresholds come from the current line spacing on
  every call (`Fill/FillBase.cpp:2697-2701` at `345efec64`). Neither they nor
  the trim/cap/minimum-length values are cached.
- The public project-input/complete-G-code-output seam is the acceptance
  interface. Compare ordered bytes excluding only the generator line; retain
  input, both streams, error and producer manifest before assertions.
  No source-structure pins, tolerance, silent omission or unsupported-feature
  classification count as output parity.
- Normal support must not merge before Tree support is complete. Nonzero
  support angle, enforcers/blockers and all modifier regions require upstream
  behavior, not newly committed guards that reject valid projects.

## Source-owned prerequisite geometry

`SupportMaterial.cpp:739-820::SupportGridPattern::extract_support` and
`ExPolygon.hpp:415-422::polygons_append` retain each selected island's
contour followed by every hole when moving into flat Polygons. The Rust
destination `geometry/support_grid.rs::extract_support` must move both;
dropping holes changes projected support volumes before toolpath generation.
Included: selected-island flattening and orientation/order preservation.
Deferred: adjacent rasterization beyond the accumulation repair below,
sampling and style algorithms, Tree
dispatch and modifier execution. The current grid shell implements the
cited upstream concept, not an independently designed support volume.

`deps_src/agg/agg_rasterizer_cells_aa.h::set_curr_cell/line` and
`agg_rasterizer_scanline_aa.h::sweep_scanline` own support-grid raster cell
contributions. Upstream flushes a local cell and resets it on coordinate
changes, then sums repeated coordinates during the sweep. The Rust
`geometry/agg_raster/cells.rs::Outline` map already aggregates by coordinate;
every edge contribution, including vertical intermediate cells, must add
to the aggregate rather than overwrite previous visits. Included: preserve
this additive contract in the existing map shell. Deferred: AGG storage
layout/performance redesign, clipping and adjacent raster algorithms. Output
acceptance remains the unchanged public landed-support complete stream.

`GCode.cpp:5659-5669::set_origin` owns integer translation of the local
position and wipe path. Writer absolute coordinates and global wipe start
remain unchanged. Do not freeze cursors or compensate individual fixtures.

`Print.cpp:2669-2696,2794-2803,2842-2845,2973-2985::_make_skirt` owns
per-object local hulls, per-instance integer translation and the combined
plate hull. Store skirt points in plate scaled integers; emit after
`set_origin(0,0)` (`GCode.cpp:4411,5249`). Object paths remain local.
`Print.cpp:2995-3019,3154-3165` owns first-layer bounds composed from translated
instance contours, plate skirt bounds and the translated existing brim shell.

## Layer-region ownership prerequisite

`SupportMaterial.cpp:592/599/1164/1430/1613/1786` owns all-region surface,
bridge, contact and average-height traversal. Keep each region's options,
raw/typed slices, perimeters and fills associated with its actual LayerRegion;
never replicate a whole-layer fill buffer across region views. Preserve
`Fill.cpp:1216-1367::Layer::make_fills` destination `surface_fill.region_id`
and source perimeter-record region/planned-layer identity where the current
layer-only adapter discards them. Two explicit upstream front-region width
uses (`SupportMaterial.cpp:1422,1527`) remain front-region uses, including
their upstream limitations. This is an ownership repair around native
Layer/LayerRegion, not a new Ares slicing abstraction.
Earlier single-region scaffold gates require source-owned
`PrintObjectSlice.cpp:1262-1365` region compensation and
`Layer.cpp:185::make_perimeters` compatibility/flow traversal before removal;
guard deletion alone cannot establish modifier output parity.

The support input adapter stores a region vector per physical layer and
looks up raw/typed records by actual RegionLayerIndex. Surface collection
(`SupportMaterial.cpp:592-604`), bridge detection (`1162-1171`), contact
generation (`1430,1613`) and trim (`3177-3190`) consume all regions with
their own options and borrowed perimeter/fill owners. When any region has
bridging extrusions, average bridging_height_avg over every layer region,
including regions without bridging (`1784-1788`), in source order. Do not
average only matching regions, take a maximum or repeat aggregate fills.
Preserve the native front-region width choices at1422/1527; the remaining
overhang kernel already iterates regions and retains the last region's
lower_layer_offset for cantilever classification. This input repair does
not establish modifier output while compatible perimeter/island/tool
execution is incomplete, and does not authorize Normal submission before Tree.

`SupportParameters.hpp:85-97` owns the bottom-interface flow ratio. Its
printing_region/num_printing_regions map to Print.hpp423-424 all_regions,
not just this layer's nonempty regions. Average bridge_flow over the object's
complete region configurations in source order and use the same average for
with_flow_ratio and bridging_flow. No front-region ratio or slice-area weighting.
The public bridge-flow modifier witness uses support landing on a lower model
platform so the bottom-interface height actually affects exported extrusion;
preserve the existing modifier and KSR fixtures unchanged.
`Layer.cpp:34-67::make_slices` builds aggregate islands with
`ClipperUtils.hpp:364::union_safety_offset_ex` (10 scaled units), then chains
contour first points. Multi-region aggregation must not replace region-owned
surfaces or metadata. The existing single-region adapter currently also
orders its surfaces; retain that observable behavior until the region-owned
perimeter prerequisite replaces this temporary compatibility shell. This is
not a claim that upstream `make_slices` itself mutates region surfaces.

`PrintObjectSlice.cpp:1314-1365` owns positive-growth region priority and
negative-growth/elephant-foot clipping against the merged outline. Port into
`compensation/multi_region.rs`; do not compensate each region independently
or shrink internal modifier boundaries. `Layer.cpp:119-138::merged` excludes
regions with no walls, shells or sparse fill from the trimming outline.
The native function-static closing epsilon initializes on the first entered
multi-region negative/elephant-foot branch, even without elephant foot.
Retain one lazy first-branch value across the objects of a Rust project
compensation invocation, not a new value for every layer/object or global
state shared across browser jobs. This single-project sequential port does
not prove native cross-project static lifetime or parallel first-arrival
parity; reference scheduling/cache changes remain unauthorized and unresolved.

## Included and deferred behavior

Included: ordinary support generation/optimization, real retained layers,
paired scheduling/emission, raw skirt eligibility, support-aware combined
skirt hulls and object-origin transitions.

Remaining acceptance work includes Tree auto/manual and style dispatch,
support-angle behavior, enforcers/blockers and multi-region inputs. These are
merge requirements, not permanent exclusions. Tree has its own source-cited
[ARD0086](ard-0086-tree-support-output-parity.md).

Adjacent deferred scope: painted brim-point ingestion, complete multi-object
brim, wipe-tower hulls, per-object skirt grouping, sequential/multi-tool
dispatch, MultiPath adaptive PA and skirt/brim external avoid-wall routing.
Existing raft-only and first-object brim adapters are temporary compatibility
shells, not full upstream ports or new fallback designs. Installed support
layers must not regenerate raft-only shell output.

## Evidence location

Execution records belong in the
plan and
historical checkpoints.
Finite test success does not close the all-printer/default/all-option matrix
or approve the containing Normal support batch.
