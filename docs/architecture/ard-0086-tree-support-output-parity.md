# ARD0086 — Tree support source-owned rewrite

Status: runtime integrated; finite single-tool merge verification complete.

## Upstream owner and destination

Rewrite `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp/.hpp::TreeSupport`,
`TreeSupport3D.cpp/.hpp::generate_tree_support_3D/generate_support_areas`,
`TreeModelVolumes.cpp/.hpp::TreeModelVolumes` and
`TreeSupportCommon.hpp::TreeSupportMeshGroupSettings/TreeSupportSettings`
under `ares-core/src/project_slice/support_material/tree/`.
The public project adapter remains `support_material/project.rs`; it installs
actual layers and toolpaths rather than an Ares-designed support pipeline.
Shared Normal support belongs to ARD0084; canonical LayerRegion ownership
belongs to ARD0089.

`SupportParameters.hpp:190-204` normalizes incompatible styles, then resolves
Tree Default to Organic and Normal Default to Grid. Preserve the raw requested
configuration for serialization and use normalized style for generation.
`TreeSupport.cpp::generate:1724-1781` dispatches Organic to TreeSupport3D;
Slim/Strong/Hybrid use detection, raft/contact, height planning, node dropping,
smoothing, drawing and toolpath generation in `tree/non_organic/`.
Both Auto and Manual reach those real generators. Normal Snug separately uses
`SupportMaterial.cpp::SupportGridPattern`, not a Tree or Grid fallback.

## Generator and geometry contracts

- Organic influence/state, radius growth, initial tips, placement, merging,
  layer pathing, positioning, collision smoothing, branch meshes and drawing
  remain owned by TreeSupport3D and its SupportElement/SupportElementState.
  Reuse a geometry primitive only where its numeric and ordering contracts
  match; circle columns are not an Organic implementation.
- TreeModelVolumes owns collision/avoidance/placeable/wall caches. Keep radius
  keys, zero-radius behavior, signed layer indices, raft prefixes, top/bottom
  spans, tapered XY clearance, ignored radius buckets and offset joins source
  equivalent. Lazy caches and settings are job-owned; Rust exclusive mutation
  replaces native cache mutexes without global mutable state or query clones.
- Preserve enabled upstream machine-border behavior (currently empty), rather
  than adopting a disabled alternative. Blockers subtract from model landing
  areas at their original indices. Cache insertion retains existing entries;
  highest-layer queries exclude layer zero alone.
- Preserve float narrowing, scaled integer truncation, source traversal/ties,
  equivalent-key ordering and GCC reference sorting where they affect output.
  Native zero-angle clamp behavior is matched to the selected official release;
  this is not a claim of universal C++ undefined-bound portability.
- Non-Organic roots consume all-region Top/Bottom surfaces, real mesh bounds,
  physical support/nozzle flows, interface settings, raft/gap prefixes and
  print-wide skirt/brim inputs. Contact/body indices differ from the complete
  installed support sequence. Cleanup retains source adjacency order.
- `TreeSupport.cpp::_make_loops:1244-1282` preserves repeated front insertion
  and zero-wall initial loops. `ExtrusionEntity.hpp:638-651` owns the single
  closed-path SupportExtrusionEntity::Loop. Do not downgrade it to Path or
  ordinary chaining; collection reversal skips the Loop itself.
- `TreeSupport.cpp::generate_toolpaths:1355-1693` retains Fill bounding-box
  conventions, Grid layer-id reversal, group dont_sort and float spacing/
  angle operands. Shared Fill kernels are consumers, not an alternate support
  algorithm.
- `FillBase.cpp::could_take_*/take_limited/take_next`, overlapping marking and
  cap emission consume `libslic3r.h::EPSILON/SCALED_EPSILON` through
  `fill/base_support/{connect,link,take,mark,caps}.rs`. Pass epsilon scaled into
  the selected coordinate lattice (normal 100, large-bed 10), not fixed 16 or
  a normal-only constant. Availability uses the current arc endpoint before
  resolving the neighboring endpoint passed to take_next.

## Painted support and canonical regions

`Format/bbs_3mf.cpp` imports triangle paint_supports through
`Model.cpp::FacetsAnnotation::set_triangle_from_string`.
`TriangleSelector.cpp::deserialize/get_facets_strict` owns reverse-hex
serialization, sparse original triangle identity, split-child order, midpoint
neighbor reuse and strict T-junction selection. Rust ownership is
`project/model_xml.rs` and `project/facets_annotation/`; annotations remain
attached to their original transformed volume, not merged-volume indices.

`PrintObject.cpp::project_and_append_custom_facets:4646-4676` and
`TriangleMeshSlicer.cpp::slice_mesh_slabs` own enforcer/blocker layer polygons,
bottom slabs and vertical points. Destinations are `mesh_slicer/slabs/` and
`support_material/facet_projection/`, consuming the shared mesh face-neighbor
view on demand. Preserve composed transforms, float coordinates, source
connectivity and scaled XY projection, without another slicing pipeline.

`TreeSupport.cpp::generate_contact_points` bins vertical centroids against
`Layer.hpp::zs_from_layers` using **slice_z**, not extrusion print_z, inserts
the sharp-tail contact in the previous contact layer, and retains scaled XY
normal as skin direction. Leaf, split, nested vertical and partial blocker
inputs enter the actual generator.

`SupportMaterial.cpp` traverses every printing region for source-owned
support surfaces/flows. `SupportParameters.hpp:85-97` averages bridge_flow
across all object printing regions before bridging/flow-ratio construction.
Modifier configuration and physical-layer identity come from ARD0089; front
region reads are retained only where the upstream explicitly uses front().

## Installation, motion and shared prerequisite contracts

`Layer.hpp::lslices`, `SupportLayer::support_islands/base_areas` and
`PrintObject.cpp::add_tree_support_layer` stay distinct in install_layers.
Non-Organic lslices includes the skirt extent; actual containment islands and
base areas retain their source provenance. Normal/Organic common generation
keeps its actual containment islands. Travel `GCode.cpp:7620-7632` uses the
source whole-polyline/one-polygon containment, not the skirt footprint.

`GCode.cpp:5752-5805/5979-5990` owns nearest projection, first ties, integer
casts, 0.001mm snapping, seam clipping and forward loop wipe. Preserve wall
direction under global spiral configuration and native Clipper orientation;
reverse cloned fitting/path state, not the source Loop. Seam-placement's
different snap/bridge preference is not a substitute.

`TreeSupport.cpp` top-gap calculations retain source float narrowing.
`ClipperUtils.hpp/Clipper.cpp` PolyTree-to-ExPolygon conversion reverses holes
in the source order before Concentric traversal. `Fill/Lightning` retains
DistanceField Miter opening, unsupported-cell ordering, locator equivalent-key
traversal and realignment/intersection visitor state. These shared geometry/
Fill corrections must preserve existing non-support behavior.

The support-aware GCode scheduler, skirt origins, cooling-role transitions,
layer labels and physical-layer vertical-shell caches follow their named
GCode/Print/PrintObject owners, not region storage slot numbers. Spiral-vase
num_layers limits the complete vertical-shell operation; the original spiral
regression and four current-spacing assertions remain acceptance controls.

## Reproducible execution and acceptance

`TreeSupport3D.cpp::merge_influence_areas` reads native arena concurrency to
choose its first bucket size. The single-core replay restricts both the
actual Ares test process and immutable Orca runtime to the same domain.
Production worker derivation remains intact. Keep official default multiworker
results and first substantive differences in the plan; do not choose a
convenient variant or claim default-worker byte determinism.

SlicerContext owns source-required randomness and first-call connection costs;
no global RNG/process-static mutable cache, filesystem, terminal or native UI
belongs in ares-core. The selected official build remains immutable. A separate
seeded/initialized build supplies reproducible evidence alongside it.

Acceptance is public project bytes to complete G-code, excluding only the
generator line. Preserve the original KSR width fixture/assertion, full-output
comparator, four spacing assertions and spiral regression. Real toolpaths,
not header-only output, unsupported guards, silent omission or tolerance,
establish parity. Final nextest/clippy/fmt/WASM and independent six-axis evidence
live in the plan. Tree dead_code must be zero without suppression.

## Included and deferred

Included: Auto/Manual; normalized Default/Organic/Slim/Strong/Hybrid; Normal
Grid/Snug; active region flows, angle and volume/painted enforcers/blockers;
branch/collision geometry, roofs/interfaces, heights, toolpaths, raft and
scheduler installation. Existing official and seeded public complete-output
controls define verified inputs, not proof of every possible configuration.

Expanded inherited-second-filament validation is RED for both baseline and
modifier inputs. The GCode adapter does not yet implement GCode.cpp4971-5075
region/role-to-extruder buckets or5241/5308 per-tool traversal/set_extruder;
its multi-tool header also formats a count as an ID list. This is a known
implementation gap, not merely a missing fixture. Whether to include its full
source-owned port in this batch or a separate multi-tool milestone requires
the explicit user scope decision recorded in the plan. Single-tool support
evidence must not be called multi-tool output parity.

Deferred follow-up coverage: remaining split orientations, transformed/mirrored
nested painting, degenerate/coplanar facets, wider printer/option/tool matrices
and default multiworker scheduling equivalence. Native Fill exception-class
mapping and complete cross-platform runtime/cancellation evidence remain
explicit adjacent parity work, not accepted algorithmic substitutes.

Excluded: painting brush/UI, progress/debug SVG, OpenGL preview and viewer-only
caches. Shared serialized paint decoding required for support is not excluded.

Detailed slice history and superseded implementation-state notes are archived
in the plan.
The [spec](../superpowers/specs/2026-10-01-tree-support-output-parity.md) and
ARD0084/0089 define the adjacent source boundaries.
