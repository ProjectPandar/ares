# Task 22B: Scaled Raw Mesh Intersections

## Status and objective

This specification is a draft. No implementation plan, test edit, or
production change may begin until these exact bytes receive independent Codex
and default-model OpenCode approvals.

Task 22B is the next small source-rewrite package in the persistent
`ksr_fdmtest_v4` parity program after released Task 22A commit
`91fc19f1dbfc85d21431791d2d5acb78af818671` and exact-SHA Tier 1 run
`29543841835`. Task 22A already owns one private plan for the committed print
object and its 460 complete `PlannedLayer` records. Task 22B must consume those
records' `slice_z` values and the project mesh to produce real, directed,
scaled triangle-plane intersection lines.

This package deliberately stops before line chaining. It introduces only the
Bambu import-time winding/centering preparation needed by slicing, integer
coordinate/point subset, shared mesh-edge identities, source-faithful facet
intersection, multi-plane dispatch, deterministic volume-occurrence identity,
and private project ownership needed to retain raw lines. It does not introduce
`Polyline`, `Polygon`, `ExPolygon`, Clipper, loop repair, closing,
simplification, region assignment, or G-code.

For the committed 3MF, Task 22B must retain one model-part volume with 460 raw
line slots and 116,472 total `IntersectionLine` records, all derived from the
3MF's 6,109 vertices, 12,234 triangles, transforms, printable area, and Task
22A layer plan. A valid project still returns
`SliceError::ProjectSlicingIncomplete`, but only after this private raw
intersection state has been built. No approximate or placeholder G-code is
observable.

### Why the package stops at raw lines

Orca's `slice_mesh` first creates `IntersectionLines`, then performs exact
edge/vertex chaining, open-chain repair, polygon construction, fill-rule
processing, Clipper union/offset, closing, and simplification. Those later
steps require the remaining Task 21A path domain plus Task 21B/21C Clipper
kernel. Combining them here would hide several independently testable numeric
and topology contracts in one review unit.

Raw lines are therefore an intentional source boundary, not a new Ares slicing
pipeline. Their endpoint direction and vertex/edge provenance are exactly the
inputs consumed by Orca's later `make_loops` path. Existing Ares STL
`planning.rs`, `segments.rs`, and `contours.rs` remain outside the project path
and may not be called as a fallback.

### Pre-implementation review contract

The independent approvals at the end of this document are design reviews.
Reviewers must judge source fidelity, completeness of the declared subset,
typed ownership, TDD observability, bounded public-input behavior, WASM
portability, and the honesty of every deferral. Missing Task 22B files and tests
in the current tree are the expected pre-implementation state, not a review
defect.

A `REVISE` verdict must identify a specification defect: an inaccurate source
claim, missing required behavior, unsafe or ambiguous ownership, an
unimplementable requirement, a hidden fallback, or acceptance criteria that
could not distinguish a wrong implementation.

## Fixed upstream rewrite boundary

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`.

- `src/libslic3r/libslic3r.h:38-70,92-96` defines `coord_t = int64_t`, the
  normal `0.000001` and large-bed `0.00001` scale factors, the 2,147 mm bed
  threshold, and the scale/unscale macros.
- `src/libslic3r/Point.hpp:187-206,262-265,643-704` defines integer `Point`,
  lexicographic X-then-Y point order, type-safe scaling, truncation toward zero
  for floating-to-integer scaling, and unscaling.
- `src/libslic3r/Format/bbs_3mf.cpp:500-505,645-661,766-775`,
  `bbs_3mf.cpp:3517-3521,3772-3781,5316-5320,5485-5494` parses Bambu 3MF vertex
  scalars as f32, multiplies by an f32 model-unit factor, and stores `Vec3f`
  mesh vertices. `deps_src/admesh/stl.h:42-45,219-235` fixes the retained
  `stl_vertex`/indexed-mesh vertex to three f32 components.
- `src/libslic3r/Format/bbs_3mf.cpp:744-778,4867-4892,4904-5045`,
  `src/libslic3r/TriangleMesh.cpp:37-74,230-234,361-365,431-437,1463-1482`,
  and Eigen 5.0.1 `Eigen/src/Core/Dot.h:92-102` define breadth-first component
  occurrence order, the exact f32 signed-volume test, import-time index swap,
  f32-statistics bounding box, and zero-vector normalization behavior.
- `src/libslic3r/Model.hpp:414-418,949`,
  `src/libslic3r/Model.cpp:1241-1260,1295-1306,2600-2616,2821-2824`, and
  `src/libslic3r/Format/bbs_3mf.cpp:1139,4927-5050` define fresh-volume
  geometry centering, the importer-global integer-keyed mesh cache, and the
  distinct later shared-mesh compensation path. Fresh meshes are centered in
  f32 and receive the f64 transform compensation before their accumulated
  component transform is applied.
- `src/libslic3r/ObjectID.hpp:20-37,42-87`,
  `src/libslic3r/Model.cpp:1241-1306`, `src/libslic3r/Print.hpp:44-48`, and
  `src/libslic3r/PrintObjectSlice.cpp:138-228` establish that slicing order is
  the unique, monotonically created runtime `ModelVolume::id()`, not a numeric
  3MF leaf object ID. Absolute `ObjectID` values also contain unrelated
  process allocation history and are not a portable project identity.
- `src/slic3r/GUI/Plater.cpp:11350-11366` is adjacent runtime wiring that
  selects the libslic3r scale factor from the printable-bed bounding-box span.
  Ares has no GUI-owned mutable global, so this input must be projected from the
  resolved 3MF `printable_area` into an explicit per-request value.
- `src/libslic3r/Model.cpp:1584-1600`,
  `src/libslic3r/Geometry.cpp:639-666`,
  `src/libslic3r/TriangleMesh.cpp:440-445`,
  `src/libslic3r/BoundingBox.hpp:261-266`,
  `src/libslic3r/BoundingBox.cpp:231-237`,
  `src/libslic3r/PrintObject.cpp:84-115`, and
  `src/libslic3r/Print.hpp:325-328` define the source-first-instance,
  all-translation-free model-part-only raw bounding box, f32-source-to-f64
  transform and center calculation, scaled center offset, and pretranslated
  object-local slicing transform.
- `src/libslic3r/Print.cpp:3849-3885`,
  `src/libslic3r/Geometry.cpp:653-666`, and
  `src/libslic3r/PrintApply.cpp:136-167` define shrink compensation and the
  separate print-object grouping transform that removes instance XY placement
  while preserving Z. Released Ares already admits only the identity-shrink
  subset before Task 22B.
- `src/libslic3r/TriangleMesh.cpp:520-618` creates shared edge IDs from
  normalized vertex pairs, preferring an oppositely oriented neighbor before a
  same-oriented fallback. Its key-only `std::sort` does not define equal-key
  ordering. `TriangleMesh.cpp:736-740` contains the index-swap helper but does
  not by itself define when import or slicing calls it.
- `src/libslic3r/TriangleMeshSlicer.cpp:43-52,58-147` defines the f32 equality
  context and the endpoint-provenance, line, flag, and facet-edge vocabulary.
  Task 22B ports only the fields needed before chaining; chaining flags remain
  deferred.
- `TriangleMeshSlicer.cpp:149-320` defines ordinary triangle-plane
  intersection, strict on-plane tests, top-edge ownership, endpoint order,
  interpolation, and vertex/edge provenance.
- `TriangleMeshSlicer.cpp:475-531` transforms each triangle once, finds the
  first and last eligible ordered slice planes by its f32 Z extent, and emits
  lines into the corresponding layer slots. Its TBB workers append under
  per-layer mutexes, so raw vector order is not a cross-platform source
  contract.
- `TriangleMeshSlicer.cpp:1826-1897` scales XY but not Z, casts the source mesh
  and transform path to f32, computes shared edges before slicing, and chooses
  the multi-plane vertex-copy path used by the 460-layer fixture.
- `src/libslic3r/PrintObjectSlice.cpp:76-95` composes centered object and volume
  transforms and tests `Eigen::Affine::rotation().determinant()`, not the raw
  linear determinant. `src/libslic3r/Point.hpp:80-85` fixes `Transform3d` as
  Eigen Affine; `deps/Eigen/Eigen.cmake:6-10` pins Eigen 5.0.1, whose
  SVD-derived Affine rotation is proper. Task 22B therefore preserves triangle
  indices even for a negative raw linear determinant. The reviewed dependency
  source is [Eigen 5.0.1 `Transform.h`](https://gitlab.com/libeigen/eigen/-/raw/5.0.1/Eigen/src/Geometry/Transform.h).
- `src/libslic3r/PrintApply.cpp:342-383,888-945`,
  `src/libslic3r/Print.hpp:271-288`, and
  `src/libslic3r/PrintObjectSlice.cpp:98-137,148-228` establish volume
  membership and lower-closed/upper-open Z-plane filtering before mesh slicing.
- `PrintObjectSlice.cpp:138-228` identifies the three sliceable volume kinds,
  sorts them by unique runtime `ModelVolume::id()`, and retains one layer vector
  per included volume.
- `PrintObjectSlice.cpp:1149-1174` obtains ordered `slice_z` values from the
  planned layers and invokes volume slicing immediately after layer creation.
- `src/libslic3r/Model.hpp:1227-1230` fixes ascending runtime-volume-ID order.

The Rust destination is `ares-core` only:

- `crates/ares-core/src/geometry.rs` and `geometry/coord.rs` own the explicit
  coordinate scale, `Coord`, and integer `Point` subset;
- `crates/ares-core/src/mesh_slicer.rs`, `mesh_slicer/topology.rs`, and
  `mesh_slicer/intersection.rs` own shared-edge indexing and raw intersection;
- `crates/ares-core/src/project_slice/raw_intersections.rs` maps already planned
  print objects to sorted per-volume raw line slots;
- `crates/ares-core/src/project/model_xml.rs` and
  `crates/ares-core/src/project/load/assemble.rs` narrow source vertex and
  model-unit materialization to the cited Bambu f32 path, omit empty geometry,
  and own request-wide bounded component expansion;
- `crates/ares-core/src/project/load/mesh_prepare.rs` owns the cited fresh-mesh
  f32 winding/centering preparation and its compensated transform while
  retaining the existing f64 `ProjectMesh` boundary;
- `crates/ares-core/src/project/load/volume_metadata.rs` and
  `crates/ares-core/src/project/domain.rs` retain only the explicit
  `mesh_shared` presence needed for the declared shared-centering capability
  gate; they do not implement a shared-mesh cache;
- `crates/ares-core/src/project_slice/state.rs` owns the resulting private
  objects in the existing single-load/single-resolve lifecycle;
- `crates/ares-core/src/project/transform.rs` exposes only the crate-private
  all-translation removal, pretranslation, f32 slicing-transform, and finite
  range seams needed by this source slice.

No new crate, native dependency, filesystem access, terminal behavior, C++
binding, global mutable scale, or non-WASM API is permitted.

The current f64 `ProjectMesh` is used only as a compatibility storage shell for
exactly promoted importer-centered f32 components; its existing f64 parse/unit
path is replaced, not retained as a fallback. `ProjectVolume::id()` remains the
3MF leaf provenance field and is not renamed into an Orc runtime ID.
`VolumeOrdinal` is private slicing state around the cited creation-order
concept. Existing STL `planning`/`segments`/`contours` scaffolds are neither
renamed nor called by this project path.

## Included supported subset

Task 22B supports the following behavior:

1. The request uses the existing single bounded project-load lifecycle, extended
   only by the declared import preparation and expanded-model budget. After
   loading it must pass the released effective-config, config-writing,
   capability, bounds, parameter, and Task 22A layer-plan gates.
2. Released effective-config validation has already required every logical
   `filament_shrink` and `filament_shrinkage_compensation_z` entry to equal
   `100%`. A nonidentity entry therefore returns the existing exact option-keyed
   `UnsupportedProjectFeature` before config writing, Task 22A, or Task 22B;
   no legacy STL shrink path is reused.
3. Every participating source object has no nonempty typed layer-configuration
   range. Missing or empty range resources are the supported single full-height
   range subset. A nonempty range is rejected request-wide after Task 22A
   planning and before Task 22B scale selection, raw-center construction,
   topology, coordinate conversion, or intersection. Load-time import
   preparation is range-independent and has already completed.
4. Every supported 3MF model unit is materialized through the Bambu importer
   numeric path: parse each source vertex component as f32, multiply by the f32
   unit-to-millimeter factor, perform the declared f32 import winding and
   geometry-centering preparation, then promote each centered f32 component
   exactly into the existing f64 `ProjectMesh` field. Parsing/multiplying in
   f64, omitting import centering, or narrowing only when slicing is not
   equivalent.
5. Shared-mesh centering is outside this package. Among all nonempty volume
   occurrences in the request, any explicit `mesh_shared` metadata or any
   repeated numeric leaf object ID returns exactly
   `UnsupportedProjectFeature("shared_mesh_centering")`. The repeated-ID check
   is request-wide and intentionally ignores package path, root object, and
   volume type because Orca's importer cache is keyed only by one integer and
   survives across object construction. Empty geometry never enters that cache.
6. Every resolved source object has exactly one unique print-object transform
   group, and that group equals the source object's first instance transform
   after XY translation removal under the identity-shrink invariant. Multiple
   physical instances that collapse into this one group remain supported.
7. The raw center is computed from every centered vertex of every nonempty,
   importer-materialized model-part volume under the source-first instance with
   X, Y, and Z translation all removed, followed by the compensated volume
   transform. Empty-triangle, negative, modifier, blocker, and enforcer volumes
   do not influence the center.
8. Each nonempty importer-materialized occurrence receives a checked typed
   `VolumeOrdinal` in breadth-first occurrence order before filtering by volume
   type. Empty geometry receives no ordinal. Nonempty support enforcer and
   blocker volumes consume ordinals but are not ordinary volume slices, so gaps
   are retained. Model-part, negative-volume, and parameter-modifier meshes are
   intersected in ascending ordinal order. Numeric 3MF leaf IDs are retained
   for source provenance and shared-mesh capability validation only; they never
   serve as the accepted volume identity or slicing sort key.
9. Empty-triangle sliceable volumes are omitted. A nonempty volume retains one
   line vector for every planned layer even when some or all vectors are empty.
10. All planned `slice_z` values are converted to f32 once in their existing
   order. `print_z` is never substituted for `slice_z`.
11. Arbitrary finite volume transforms, object Z translation, XY translation
   inside a volume matrix, scaling, rotation, and negative raw linear
   determinant are supported. After the separate import-time signed-volume
   normalization, triangle indices are not swapped again for a mirrored affine
   transform, matching the pinned Orca/Eigen predicate. Instance XY placement
   remains outside object-local raw slices.
12. A normalized mesh edge may have one directed use or exactly two uses. A
   two-use group is paired whether its orientations are opposite or equal. A
   group with more than two uses has upstream-undefined equal-key order and is
   explicitly rejected as `UnsupportedProjectFeature("mesh_topology")` before
   intersection; Task 22B does not repair or close open paths.
13. Normal and large-bed scaling are selected only from the resolved full
   `printable_area` carried in the 3MF. No fixture name, expected G-code, machine
   ID, external preset, or process-global state participates.

The distinct-transform centering case in which Orca rotates the first raw
bounding-box center into another print-object Z rotation is not silently
approximated. Until a later source-cited transformation package retains or
reconstructs the required structured rotations, more than one unique transform
group or a mismatch with the first source-instance transform returns
`UnsupportedProjectFeature("print_object_centering")` after Task 22A planning
and before raw intersection.

The layer-range gate is a single request-wide preflight, not a per-object late
check. After Task 22A has completed all planning, any participating source
object with a nonempty typed `layer_config_ranges()` returns exactly
`UnsupportedProjectFeature("layer_config_ranges")` before scale selection,
center calculation, transform, topology, coordinate conversion, or raw-budget
claims for any object. This prevents geometry that Orca would exclude before
`slice_mesh` from surfacing a coordinate or budget error. Full
`LayerRangeRegions` membership and plane filtering remain a later source slice;
unfiltered slicing followed by discarded output is forbidden.

The shared-mesh gate is likewise request-wide. It runs after the
`print_object_centering` gate but before volume ordinals, dense-slot budgeting,
scale selection, raw-center calculation, topology, or intersection. This is not
an allocation optimization gate: Orca's fresh branch composes
`C * T(center_shift)`, while its shared branch reuses a previously centered
mesh and adds that mesh's saved shift directly to the already composed
component offset. Treating every occurrence as a fresh mesh is observably
wrong. Retaining the cache key, first-seen shared state, and distinct transform
compensation is deferred as one later source slice; no fresh-mesh approximation
is allowed.

## Bambu fresh-mesh preparation and volume occurrence identity

### Import winding, empty geometry, and centering

Component expansion remains breadth-first in declared component order. For
each encountered leaf:

1. A mesh with no vertices or no triangles is omitted before volume-metadata
   index selection and before `ProjectVolume` construction. It receives no
   volume ordinal, contributes no raw-center vertex, and cannot enter the
   shared-mesh gate or cache-key set. This matches Orca's `Geometry::empty()`
   check before `_generate_volumes_new`; retaining an empty phantom volume is
   forbidden because it would shift metadata association for every later leaf.
2. Parse every source coordinate as f32, multiply it by the model's f32 unit
   factor with an f32 result, and retain the resulting finite f32 vertices and
   source face order.
3. Compute signed volume in source face order with an f32 accumulator. Let
   `p0` be the first vertex. For every face `(a,b,c)`, evaluate in f32 and in
   this order: `u = b-a`, `v = c-a`, `cross = u.cross(v)`,
   `normal = cross.normalized()`, `area = 0.5 * cross.norm()`,
   `height = normal.dot(a-p0)`, then
   `volume += area * height / 3.0`. Eigen 5.0.1 returns an exact zero vector
   unchanged from `normalized()`, so an exact zero-area face contributes zero.
   IEEE overflow/NaN follows this expression and is not replaced by a
   determinant/triple-product shortcut.
4. If and only if the final f32 accumulator is strictly `< 0.0`, swap face
   slots 1 and 2 for every triangle in place. Face order and vertices remain
   unchanged. Positive zero, negative zero, positive volume, and NaN do not
   flip and do not create a new validation error.
5. Compute componentwise f32 minima and maxima over all vertices, including
   unreferenced vertices. Promote each extremum exactly to f64 and compute
   `center_shift = (min + max) / 2.0` in f64. For a nonzero finite shift,
   update each f32 component as `component += -(center_shift_component as f32)`.
   Promote only this centered f32 result exactly into `ProjectMesh`; the
   original uncentered vertex is not retained as the slicing mesh.
6. Pair that centered mesh with the compensated f64 volume transform
   `accumulated_component_transform * T(center_shift)`. The translation is
   therefore `C.translation + C.linear * center_shift`. The 3MF part metadata
   `matrix` remains source provenance and does not replace this volume
   transform. The mesh and compensated transform are constructed together and
   cannot be recomputed or centered twice.

The centering subtraction and its f64 compensation may not be algebraically
cancelled. Casting the midpoint to f32 before subtracting, then retaining the
full f64 midpoint in the transform, makes an off-origin asymmetric mesh
observably different from applying `C` directly to the original vertices.

For KSR, the imported f32 signed volume is strictly positive (approximately
51,011), its f32 bounds are exactly
`[-37.5, -35.0, -46.0]..[37.5, 35.0, 46.0]`, its shift is exactly zero, and its
first face remains `[2, 0, 1]`. Import preparation must therefore leave every
existing KSR raw-state count and digest unchanged.

### Deterministic volume ordinal

Orca's absolute `ObjectID` value is process-global, non-contiguous, and affected
by unrelated `ObjectBase` allocations. Task 22B does not reproduce that
unobservable number. It preserves the ordering and lookup semantics consumed
by `slice_volumes_inner` with a private
`VolumeOrdinal(std::num::NonZeroU32)`:

- the ordinal is one-based within each source `ProjectObject`;
- it is the checked position in that object's nonempty breadth-first
  `ProjectVolume` occurrence vector after import omission and before volume-type
  filtering;
- nonempty blocker and enforcer occurrences consume an ordinal even though
  their raw slices are omitted;
- every physical instance of one source object reuses that object's ordinal
  sequence;
- the request-unique private identity is
  `(source_object_index, VolumeOrdinal)`; no cross-object numeric ordering is
  invented;
- accepted raw volumes are retained in ascending ordinal order, which is the
  normalized equivalent of Orca sorting that object's monotonically created
  runtime IDs.

The request-wide expanded-model budget below guarantees fewer than `u32::MAX`
materialized occurrences, so checked one-based conversion is an internal
invariant rather than a second public-input limit or later error. Numeric source
leaf ID, name, package path, or metadata order cannot replace the ordinal.

### Bounded component expansion

The bounded archive size does not itself bound a component DAG's expanded
occurrence count. Before occurrence expansion, project assembly performs one
iterative three-color DFS over component identities reachable from the unique
build roots. It follows declared component order, rejects a gray back-edge with
the existing exact
`InvalidInput("invalid project model graph: component graph contains a cycle")`,
and ignores cycles in objects unreachable from the build, matching the current
collection boundary. Component target/identity validation has already run.
The preflight visits each reachable graph node and edge once and uses an
explicit heap stack, not Rust recursion.

After that acyclicity proof, breadth-first occurrence expansion must remove the
existing per-pending-item cloned ancestor vector. Its pending item contains only
the path, object ID, and accumulated transform. Project assembly then owns one
request-wide `ExpandedModelBudget` with a limit of exactly 1,000,000 units:

- claim one unit before scheduling each unique-root or child object occurrence;
- for a nonempty leaf, additionally claim
  `vertices.len() + triangles.len()` before cloning/materializing its mesh;
- an empty leaf consumes only its already claimed occurrence unit and is then
  omitted;
- repeated physical build instances of one source object do not recollect its
  volume DAG;
- all additions are checked, and the claim that would exceed the limit or
  overflow returns exactly
  `InvalidInput("project expanded model item count exceeds supported limit of 1000000")`.

Claims occur before queue growth or mesh allocation, and one budget is shared
across every source object assembled for the request. This is a public-input
resource boundary, not a slicer option, and changes no accepted geometry. KSR
claims exactly 18,345 units: two object occurrences, 6,109 vertices, and 12,234
triangles. The finite graph preflight plus ancestry-free pending representation
make queue/path ownership linear in the claimed occurrence count; merely adding
a counter while retaining cloned ancestor paths is noncompliant. The budget
bounds loader expansion; the later ordinal type must not be presented as that
bound.

## Coordinate scale and point semantics

### Per-request scale selection

The coordinate scale is a small copyable enum or equivalent typed value with no
global mutation:

- compute the finite `printable_area` bounding box in unscaled millimeters;
- an empty point list has zero span, matching the default undefined-box/zero
  size effect at the cited caller;
- if `max(max_x - min_x, max_y - min_y) <= 2147.0`, select factor
  `0.000001`;
- otherwise select factor `0.00001`.

The same request-local value is passed to center quantization, the f32 slicing
transform, and checked coordinate conversion. Concurrent normal- and large-bed
requests must not affect each other.

### Checked boundary and trusted integer domain

`Coord` is exactly `i64`. `Point` contains exactly two `Coord` values and uses
full-point equality and lexicographic X-then-Y ordering. Integer geometry after
checked conversion trusts this invariant.

At 3MF vertex materialization, both the parsed f32 scalar and its f32
unit-to-millimeter product must be finite. A nonfinite result returns the
existing bounded `InvalidInput("project mesh vertices must be finite")` before
effective-config resolution. The exact f32 product is promoted into the f64
`ProjectMesh`; no later code may recover discarded precision or redo unit
conversion.

At the external project-to-scaled boundary, a coordinate is accepted only when
the quotient by the selected factor is finite and lies in the f64 interval
`[-2^63, 2^63)`. Accepted scalar scaling truncates toward zero, including
negative fractional values. Unscaling multiplies by the same selected factor.
Rust's saturating float-to-integer cast must not be used to hide an invalid
external coordinate.

The multi-plane mesh path has two distinct conversions that must not be
collapsed:

- a transformed vertex or an endpoint exactly inherited from a mesh vertex is
  converted to `Coord` by truncation toward zero;
- a strict interior edge intersection is converted with
  `floor(interpolated_scaled_coordinate + 0.5)`.

The latter is intentionally not Rust `round()`: for example, a negative half
coordinate rounds toward positive infinity under the cited expression.

## Object-local transform semantics

For each supported resolved object:

1. Before raw-center transformation, use each nonempty model-part's
   importer-centered f32 components exactly promoted in `ProjectMesh`. Start
   from the source object's first instance transform with X, Y, and Z
   translation all removed, compose it with the paired compensated volume
   transform `C * T(center_shift)`, and apply the combined f64 transform to
   those promoted vertices. Merge into an f64 bounding box. Direct use of the
   original uncentered mesh, a more precise f64 parse/unit product, or an f32
   transform/bounding box is not the Orca path.
2. Compute each box-center component as `(min + max) / 2.0` in f64. Quantize
   only center X and Y through the
   selected scale using truncation toward zero, then immediately unscale those
   integer offsets. This reproduces Orca's `Point::new_scale` followed by
   `unscale`; subtracting the exact unquantized center is incorrect.
3. Separately take the sole resolved print-object group transform. Its XY
   placement is zero, its Z translation is retained, and its shrink compensation
   is identity because the two shrink options passed their earlier gate.
4. Pretranslate that resolved group transform by the negative quantized X/Y
   center and zero Z. This translation acts after the object's linear
   transform.
5. Compose the centered object transform with each compensated sliceable volume
   transform in that order.
6. Preserve every importer-normalized triangle's index order when constructing
   shared edge IDs and intersections. A negative source signed volume may have
   swapped slots 1 and 2 once during import. After that, the pinned Eigen Affine
   `rotation()` returns the proper SVD rotation; Ares must not substitute the
   raw linear determinant and must not flip a mirrored triangle a second time.
   The retained prepared project mesh is never mutated during slicing.
7. Build the slicing transform in f64 by prescaling output X and Y by
   `1 / factor`, leave output Z unscaled, cast every matrix coefficient to f32,
   cast every exactly promoted centered project vertex component back to its
   identical f32 value, then apply the f32 transform.

Instance Z translation therefore has no effect on the raw-center box but does
shift slicing Z. The one-group/equality capability gate makes the supported
subset's source-first/group Z-rotation difference exactly zero; the distinct
rotation-center path remains explicitly unsupported.

All transformed components must remain finite. Any X/Y value that later cannot
be represented as a checked `Coord` returns the bounded error
`InvalidInput("project mesh slicing coordinate is nonfinite or outside the scaled coordinate range")`.
The error does not echo untrusted numeric text.

## Shared edge identity

For each local triangle in face order, create three directed edge uses. Store
the lower and higher vertex IDs as the normalized key and retain whether the
face edge was reversed relative to that key. Sort groups by normalized key.
Orca's comparator intentionally says nothing about order inside an equal-key
group. Ares uses ascending face index and then local edge index only as an
explicit cross-platform normalization of that unspecified tie; it is not
claimed as an upstream ordering guarantee.

Within each equal-key group:

1. reject a group of more than two uses with exactly
   `UnsupportedProjectFeature("mesh_topology")`;
2. assign one increasing edge ID to a one-use boundary group;
3. assign one increasing shared edge ID to a two-use group, with the
   oppositely oriented case being the ordinary manifold path and the
   same-oriented case being Orca's unambiguous fallback.

Every face edge receives exactly one nonnegative ID. The implementation uses
checked conversion for the ID range and returns
`InvalidInput("project mesh edge count exceeds supported range")` rather than
wrapping. The KSR topology must produce 18,351 unique edge IDs, with every edge
used by exactly two oppositely oriented triangle edges. Because each KSR group
has exactly two uses, unspecified equal-key ordering cannot change its pairing
or edge ID.

## Facet intersection semantics

For a triangle already transformed into `[f32; 3]` vertices and a sorted f32
slice plane:

1. Compute strict f32 minimum and maximum Z. Fully horizontal triangles are
   ignored by ordinary slicing.
2. Select the lowest vertex exactly as Orca: vertex 1 wins an equality with the
   minimum, otherwise vertex 2 wins, otherwise vertex 0.
3. Visit the three directed edges starting from that index. Plane comparisons
   use strict f32 equality and ordering, never an epsilon.
4. If two edge vertices lie on the plane, return immediately. When the third
   vertex is below, the edge is an owned `Top` slicing line and its endpoints
   and vertex references are reversed. When the third vertex is above, it is a
   non-owned `Bottom` cutting edge and is not retained in raw slicing output.
5. A single on-plane vertex is retained once by its vertex ID. A strictly
   crossing edge is first ordered by ascending endpoint vertex ID, then
   interpolated in f64 from the already f32 vertex values. Interior
   intersections retain the shared edge ID.
6. Exactly two intersection points produce a `General` line from collected
   point 1 to collected point 0. This direction preserves Orca's “external on
   the right” convention. Endpoint sorting by coordinate is forbidden.
7. Rounding may produce a zero-length line; Task 22B preserves it exactly for a
   later source-cited repair decision rather than silently deleting it.

Rust should model valid endpoint provenance as an exhaustive enum such as
`Vertex(u32)` or `Edge(u32)`, not as two independently invalid sentinel fields.
The retained `FacetEdgeType` subset is `General` and `Top`; pure tests may keep
`Bottom`/`Horizontal` classification private to distinguish non-retained
branches. Chaining flags, `TopBottom`, and `Slab` are not invented early.

## Multi-plane dispatch and bounded ownership

Each nonempty sliceable volume is transformed once, indexed once, and visited
triangle by triangle. For each triangle, binary-search the ordered f32 plane
array for the first plane `>= min_z` and the first plane `> max_z`; only that
half-open iterator span calls facet intersection. Under the supported single
full-height range, the array contains every planned plane. Allocating or
scanning a triangle-by-layer matrix is forbidden.

Orca's multi-plane TBB workers append under per-layer mutexes, so their raw
container order depends on scheduling. Ares deliberately normalizes this
unspecified detail by visiting source faces in ascending index order and, for
each face, eligible plane indices in ascending order. Each layer vector is
therefore ordered by ascending source face index. This preserves directed
endpoints, provenance, and edge type but does not claim to reproduce one Orca
TBB schedule. Later chaining and observable G-code order require their own
parity gates; content-sorting production lines is forbidden.

A request-wide dense-slot preflight runs before any raw layer vector is
allocated. Its count is
`sum(plan.layers.len() * retained_nonempty_sliceable_volume_count)` across all
planned objects and transform groups. Model-part, negative, and parameter
modifier volumes with at least one triangle are counted; empty-triangle,
blocker, and enforcer volumes are not. Checked multiplication and addition are
mandatory. The request permits exactly 1,000,000 layer slots; a larger total or
arithmetic overflow returns
`InvalidInput("project raw intersection layer slot count exceeds supported limit of 1000000")`.
The whole-request sum is accepted before creating any
`Vec<Vec<IntersectionLine>>`, so a later object's excess cannot follow partial
allocation or lose to an earlier object's coordinate/topology work. KSR claims
exactly 460 slots. This cap bounds only Task 22B's new dense slot ownership; it
is a public-input resource boundary rather than a slicer option, does not alter
accepted geometry, and does not claim to replace existing loader/archive
limits.

Separately, the request-wide `RawIntersectionBudget` counts retained slicing
lines. It permits exactly 1,000,000 lines and rejects the next with
`InvalidInput("project raw intersection count exceeds supported limit of 1000000")`.
This public-input resource bound is not a slicer option and does not alter any
accepted geometry. It must be shared across all objects, transforms, volumes,
and layers in one request. The dense-slot and retained-line counters are
independent even though their numeric limits are equal.

Private ownership is equivalent to:

```rust
struct IntersectedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<RawVolumeIntersections>,
}

struct RawVolumeIntersections {
    volume_ordinal: VolumeOrdinal,
    volume_type: ProjectVolumeType,
    layers: Vec<Vec<IntersectionLine>>,
}
```

Names may follow local style, but the ownership may not be split into parallel
top-level vectors. For every retained volume,
`layers.len() == object.plan.layers.len()`. Each object still records the
stable `source_object_index` and `transform_index` owned by Task 22A. The
enclosing `source_object_index` plus `volume_ordinal` is the full private volume
identity; the raw state must not expose or sort by `ProjectVolume::id()`.

`ProjectSliceState` owns the loaded `Project`, bounded resolved configuration,
optional exact config block, and `Vec<IntersectedPrintObject>`. It does not
clone or reload the project, re-resolve configuration, reread the archive, or
reconstruct options from profile labels.

## Lifecycle and error precedence

The request lifecycle remains:

1. bounded 3MF load, build-reachable iterative component-cycle preflight,
   ancestry-free request-wide expanded-model budgeting, empty-geometry
   omission, f32 source-vertex/unit materialization, fresh-mesh winding and
   centering preparation, and typed document validation;
2. bounded effective-config resolution, including the existing exact
   `filament_shrink` then `filament_shrinkage_compensation_z` identity gates;
3. exact Bambu config-block generation when applicable;
4. Task 22A capability validation and fixed layer planning;
5. Task 22B request-wide nonempty-layer-range gate;
6. Task 22B supported-centering validation;
7. Task 22B request-wide shared-mesh-centering validation;
8. Task 22B per-source-object typed volume-ordinal projection;
9. Task 22B request-wide dense raw-layer-slot preflight;
10. Task 22B scale selection, raw center and
   transform construction, topology indexing, and intersection generation;
11. later-stage `ProjectSlicingIncomplete`.

An earlier error always wins. In particular, malformed archive, component
identity/cycle, nonfinite f32 vertex/unit materialization, expanded-model
budget, and typed-document failures are load errors and precede effective
config. Effective-config, shrink-option,
and config-writer failures precede every post-load Task 22B error; Task 22A
unsupported/planning errors, including the existing range-owned `layer_height`
error, precede the Task 22B capability preflights. The request-wide
`layer_config_ranges` gate precedes centering and every later failure; the
centering gate precedes `shared_mesh_centering`; the shared-mesh gate precedes
ordinal projection and the dense layer-slot limit; and that whole-request slot
limit precedes scale-dependent center/coordinate work, topology, retained-line
budget claims, and raw-state allocation. A valid supported request reaches
`ProjectSlicingIncomplete` only after retaining its raw state.

`GenerationMetadata` remains unconsumed because no G-code serialization occurs.
The existing exact 49,004-byte config block and its SHA-256 must remain
unchanged.

## Exact production and test scope

Expected production changes are limited to:

- create `crates/ares-core/src/geometry.rs`;
- create `crates/ares-core/src/geometry/coord.rs`;
- create `crates/ares-core/src/geometry/tests.rs` and focused children as needed;
- create `crates/ares-core/src/mesh_slicer.rs`;
- create `crates/ares-core/src/mesh_slicer/topology.rs`;
- create `crates/ares-core/src/mesh_slicer/intersection.rs`;
- create focused `mesh_slicer/tests` modules, split before 400 physical LOC;
- create `crates/ares-core/src/project_slice/raw_intersections.rs`;
- create `crates/ares-core/src/project/load/mesh_prepare.rs` and focused tests
  for the cited f32 signed-volume, index normalization, geometry centering, and
  transform compensation;
- modify `crates/ares-core/src/project/model_xml.rs`,
  `crates/ares-core/src/project/load/assemble.rs`, and their focused tests for
  cited f32 source-vertex/unit materialization, empty-geometry omission before
  metadata association, build-reachable iterative component-cycle preflight,
  ancestry-free BFS occurrence expansion, and the request-wide expanded-model
  budget;
- modify `crates/ares-core/src/project/load/volume_metadata.rs` and
  `crates/ares-core/src/project/domain.rs` only to retain explicit
  `mesh_shared` presence for the declared capability gate;
- modify `crates/ares-core/src/project/transform.rs` only for the cited
  crate-private numeric seams;
- modify `crates/ares-core/src/project_slice/state.rs`, `project_slice.rs`, and
  their focused tests to own the new state;
- modify `crates/ares-core/src/lib.rs` only to register private modules;
- after whole-implementation approval only, update
  `docs/architecture/option-parity-v4.md` and `docs/roadmap.md`.

Every changed production line must serve this raw-intersection boundary. No
existing STL geometry type is renamed or generalized. No public geometry API is
added. No option registry/default/export shape changes. No generated fixture,
precomputed path, expected G-code fragment, or filename branch may appear in
production.

All Rust files must remain below 400 physical lines. Core code remains safe
Rust, deterministic, no-filesystem, and compatible with Windows, Linux, macOS,
and `wasm32-unknown-unknown`. No `unsafe`, target-specific arithmetic, Rayon or
native threading dependency, generic JSON value, dynamic option map, or legacy
fallback is authorized.

The obsolete Orca source-pinning audit is currently clean. Task 22B must not
add a test that opens `/OrcaSlicer`, the temporary pinned worktree, or upstream
source files. Source citations belong in this specification; committed tests
assert behavior only. Existing reference-G-code and option-inventory harnesses
outside this package remain untouched.

## TDD acceptance

### Required REDs

The independently approved implementation plan must establish genuine failing
tests before each production package. Every new or renamed test begins with
`task22b_`. At minimum:

1. **Scale selection and isolation:** printable spans 2,147 and just above it
   select the normal and large factors respectively; the KSR 256 mm area selects
   normal; empty area selects normal; concurrent/repeated requests with
   different areas do not leak scale state.
2. **Checked scale semantics:** `1.9e-6 -> 1` and `-1.9e-6 -> -1` under normal
   scale; exact zero and integer boundaries round-trip; nonfinite and quotient
   values outside `[-2^63,2^63)` are rejected before cast.
3. **Point domain:** equality compares both coordinates, lexicographic order is
   X then Y, and the type contains no floating coordinate.
4. **Source vertex/unit materialization:** a table names and exercises every
   supported f32 unit factor—micron `0.001`, millimeter `1.0`, centimeter
   `10.0`, inch `25.4`, foot `304.8`, and meter `1000.0`—and proves each source
   scalar is parsed to f32 and multiplied by that f32 factor before exact
   centering/promotion. Source scalar `0.3` therefore yields the exact
   f32 products `0.0003`, `0.3`, `3.0`, `7.6200004`, `91.44`, and `300.0` in
   that order. The inch X range `[0, 0.3]` is the precision
   discriminator: it produces f32 max `7.6200004` and scaled center `3_810_000`,
   not the late-f32 result `3_809_999`. Nonfinite parsed/product values return
   the bounded vertex error before effective-config resolution.
5. **Fresh-mesh winding and centering:** a closed tetrahedron and its fully
   reversed copy prove the exact f32 face-order volume expression and strict
   `< 0` test; the reversed copy swaps only face slots 1/2 and produces the same
   importer-normalized directed fingerprint. Positive, positive-zero,
   negative-zero, planar, and exact zero-area cases preserve face order without
   error. A mirrored component proves negative source winding is normalized
   once during import and is not flipped again by the later affine predicate.
   For f32 X bounds `128.0..128.00001525878906`, require f64 shift
   `128.00000762939453`, f32 shift `128.0`, centered bounds
   `0.0..0.0000152587890625`, reconstructed raw bounds
   `128.00000762939453..128.0000228881836`, and raw-center coordinate
   `128_000_015`; directly transforming the uncentered mesh yields the wrong
   `128_000_007`. A nonidentity linear component transform proves compensation
   is `C * T(shift)`, not `T(shift) * C`, and part metadata `matrix` remains
   provenance only. KSR asserts positive volume, exact zero shift, and unchanged
   first face `[2,0,1]`.
6. **Empty geometry and bounded graph expansion:** an empty leaf preceding a
   nonempty leaf with the same numeric ID is omitted before metadata selection.
   With two repeated part rows named `First` then `Later`, the nonempty leaf is
   the sole loaded volume and selects `First`. Pure budget claims accept exactly
   1,000,000 combined occurrence/vertex/triangle units, reject the next claim,
   map checked-add overflow to the same exact expanded-model error, and do not
   allocate a million test objects. Reachable self/two-node cycles return the
   exact existing cycle error before any mesh materialization, while an
   unreachable cycle remains irrelevant. A depth-32 chain whose last node fans
   out to 32 distinct leaves proves the iterative preflight visits finite graph
   nodes/edges once, the later BFS pending record carries no ancestor path,
   claims occur before queue growth, and budget usage is only declared
   occurrences plus nonempty leaf vertices/triangles—not depth times width.
   Budgets are request-wide across source objects, repeated physical instances
   do not recollect one DAG, and KSR claims exactly 18,345.
7. **Raw center and slicing transform:** under normal scale and identity
   transforms, importer-prepared model-part X bounds `[0, 0.000004]` produce
   center coordinate 1, and `[-0.000004, 0]` produce -1; a direct-f64 parse
   produces 2 and -2. With f32-exact local bounds `[0, 4]` and f64 volume X
   scale `1e-6`, the center coordinate is 2; incorrectly narrowing the
   transform/bbox to f32 produces 1. An asymmetric box proves quantization
   before unscale; unreferenced vertices contribute to both import centering and
   the later raw box; empty, negative, and modifier volumes do not affect the
   raw center. The first source instance has all translation removed only for
   center calculation; object Z does not change that center but shifts slicing
   Z. Object-then-compensated-volume composition, volume XY translation,
   removed instance XY, f32 slicing-matrix casting, and pretranslation order are
   discriminated. A negative raw linear determinant preserves the
   importer-normalized indices and directed output.
8. **Volume ordinal and shared-centering gate:** one object with breadth-first
   leaves `[empty id9, blocker id8, model id7, modifier id6, enforcer id5,
   negative id4]` materializes IDs `[8,7,6,5,4]` with ordinals `[1,2,3,4,5]`
   and retains raw `(ordinal, source_id, type)` values
   `[(2,7,ModelPart),(3,6,ParameterModifier),(5,4,NegativeVolume)]`. To
   distinguish BFS from DFS, a root declares internal group `A` first and leaf
   ID 3 second, while `A` declares leaf IDs 1 then 2; admitted leaf order and
   ordinals are `[3,1,2] -> [1,2,3]`, whereas DFS would incorrectly produce
   `[1,2,3]`. Thus both DFS and source-ID sorting fail.
   Two source objects may each use ordinal 1 because their outer
   `source_object_index` differs; repeated instances reuse one sequence; repeated
   and concurrent requests restart at 1. Any two nonempty occurrences with the
   same numeric leaf ID, including across package paths, root objects, or types,
   and any nonempty occurrence with explicit `mesh_shared`, return exactly
   `shared_mesh_centering`. An empty duplicate or empty explicit-shared leaf is
   ignored. The request-wide gate wins over dense-slot and coordinate errors.
9. **Identity-shrink prerequisite:** the KSR project resolves both logical
   entries of `filament_shrink` and `filament_shrinkage_compensation_z` to
   `100%`. Mutating either option to a nonidentity value returns the existing
   exact option-keyed unsupported error during effective-config resolution and
   creates no Task 22A or Task 22B state.
10. **Centering capability:** multiple unique transform groups and a first-source
   transform mismatch each return exactly `print_object_centering`; multiple XY
   instances collapsed into one equal group remain supported.
11. **Layer-range preflight:** a benign nonempty typed range returns exactly
   `layer_config_ranges` after Task 22A planning and before all Task 22B raw
   intersection geometry. The gate is request-wide: a ranged later object wins
   over an earlier object's otherwise-invalid scaled coordinate, and an excluded
   negative/modifier volume cannot surface a coordinate or budget error. A
   range-owned `layer_height` failure retains its earlier Task 22A precedence;
   missing and empty range resources remain supported.
12. **Topology:** opposite-oriented manifold neighbors share an ID; a boundary
   edge receives a unique ID; exactly two same-oriented uses share the
   unambiguous fallback ID; more than two equal-key uses return exactly
   `mesh_topology` before intersection; repeated indexing is equal.
13. **Ordinary facet crossing:** a sloped triangle produces the complete directed
   line, both endpoint references, and `General` type with the exterior on the
   right. A coordinate-sorting implementation must fail this test.
14. **Conversion distinction:** positive and negative fractional vertex
   inheritance proves truncation, while positive and negative half-valued
   interior intersections prove `floor(value + 0.5)` rather than Rust
   `round()`.
15. **Plane degeneracies:** one vertex exactly on plane is deduplicated by vertex
   ID; two vertices on plane with third below produce the reversed owned `Top`
   edge; third above produces no raw line; a fully horizontal triangle produces
   no ordinary slicing line; strict f32 equality is distinguished from a nearby
   value.
16. **Multi-plane dispatch and normalized order:** the private slicer trusts the
    Task 22A invariant that planes are sorted; sorted planes retain input slot
    order, min/max equality boundaries, empty slots, and duplicate f32 planes.
    A triangle is visited only across its lower/upper-bound span, and a test
    with distinct `slice_z`/`print_z` proves the project adapter uses `slice_z`.
    Multiple faces prove ascending importer-normalized face order in each layer
    and ascending eligible-plane traversal within a face; content-sorting
    production output must fail.
17. **Raw ownership budgets:** dense-slot preflight accepts
    `(40_000 layers * 10 volumes) + (60_000 * 10) == 1_000_000`, rejects
    `(50_000 * 10) + (50_000 * 11) == 1_050_000`, and maps
    `100_000 * usize::MAX` overflow to the same exact slot-limit error without
    constructing those layers or volumes. A small two-object projection proves
    that the three nonempty sliceable kinds count, empty meshes and
    blocker/enforcer do not, and the preflight is shared request-wide. The
    retained-line counter independently allows exactly 1,000,000 claims and
    rejects the next without allocating one million test lines; coordinate and
    edge-range errors remain bounded and deterministic.
18. **Volume projection:** sliceable volume kinds are included in ordinal order;
    blocker/enforcer and empty geometry are omitted from raw slots without
    renumbering; each retained volume owns exactly the plan's number of layer
    slots; source-object, transform, and ordinal identities cannot cross-wire
    across two project objects.
19. **Lifecycle:** malformed archive, vertex/unit materialization,
    expanded-model budget, effective-config, config-writer, and Task 22A
    failures retain their declared precedence. Request-wide range, centering,
    shared-mesh, then dense-slot failures precede coordinate, topology, and
    retained-line failures exactly as declared. A later object's shared key or
    slot excess wins an earlier object's otherwise-invalid coordinate before
    raw allocation; a valid supported project still reaches
    `ProjectSlicingIncomplete`.
20. **Real KSR fixture:** using only committed 3MF bytes, assert one intersected
    object, one model-part volume ordinal 1, 460 slots, a request-wide dense-slot
    count of 460, an expanded-model count of 18,345, normal scale, 6,109 input
    vertices, 12,234 triangles, 18,351 opposite-paired edge IDs, and 116,472
    total raw lines. Assert maximum 3,011 lines at layer 46 (`slice_z` f32 9.3),
    plus representative `(line_count, closed endpoint components)` pairs:
    layer 0 `(1046,12)`, layer 2 `(932,12)`, layer 12 `(1265,12)`, layer 17
    `(1138,12)`, layer 37 `(880,15)`, layer 230 `(38,1)`, and layer 459
    `(72,9)`. Layer 46 has 41 closed components and f32 value
    `9.300000190734863`. Layers 2, 12, 17, and 37 exercise exact
    vertex/coplanar ownership. Assert empty typed ranges, the unique normalized
    `(0.0, f64::MAX, None)` candidate, both two-entry 100% shrink vectors, the
    exact semantic and face-order digests below, and the representative records
    below. Require no zero-length line in this fixture, unchanged 49,004-byte
    config block/hash, complete repeated-state equality, and the unchanged
    public incomplete result.
21. **Anti-hardcoding mutations:** changing one mesh vertex changes the fixed
    semantic raw-state digest; changing only `printable_area` across the 2,147
    threshold changes scaled coordinates without changing fixture-name logic;
    neither test reads the reference G-code or `options-v242.json`.

Fixture counts and expected values belong only in tests. The real-fixture RED
must fail because raw intersection ownership does not yet exist, not because a
test is manually forced to fail.

### Exact KSR raw-state oracles

Tests use two SHA-256 oracles over one explicit fixed-width binary encoding. All
multibyte integers are big-endian, all tags are literal bytes, and no Rust enum
layout, pointer address, allocation capacity, or platform-dependent structure
layout is serialized:

```text
ASCII "ares-task22b-raw-state-v1\0"
u32 object_count

per object:
  u64 source_object_index
  u64 transform_index
  u32 volume_count

per volume:
  u32 volume_ordinal
  u8  volume_type       // 0 ModelPart, 1 Negative, 2 ParameterModifier
  u32 layer_count

per layer:
  u32 layer_index
  u32 line_count

per line:
  i64 a.x
  i64 a.y
  u8  a.reference_kind // 0 Vertex, 1 Edge
  u32 a.reference_id
  i64 b.x
  i64 b.y
  u8  b.reference_kind // 0 Vertex, 1 Edge
  u32 b.reference_id
  u8  edge_type        // 0 General, 1 Top
```

The prefix is exactly 26 bytes, ending in one NUL byte, with hex
`617265732d7461736b3232622d7261772d73746174652d763100`. The KSR stream length
is therefore exactly `26 + 33 + (460 * 8) + (116_472 * 43) = 5_012_035` bytes.
For the source-semantic oracle, sort
a copy of each layer's records lexicographically by
`(a.x, a.y, a.reference_kind, a.reference_id, b.x, b.y,
b.reference_kind, b.reference_id, edge_type)` without swapping A/B. Its exact
SHA-256 is
`a82b2d193c23c8ba499c7abd56e21cb9956f5444e9b51b1b261a7e9b67d26d21`.
This detects XY offset/scaling, endpoint direction, provenance, edge type, and
object/volume/layer cross-wiring while intentionally ignoring Orca's TBB append
schedule.

The encoded ordinal is the nonzero one-based value inside its enclosing source
object. KSR's sole value remains `1`, so renaming the field from the incorrect
3MF/runtime-ID claim does not change its bytes, stream length, or hashes.

For the Ares-order oracle, encode the retained layer vectors without sorting.
Its exact SHA-256 is
`1a6e83f2d5f53b73fa7ba9cb6444909816276496361f7fb9f9305412d2045e79`.
This freezes the declared ascending-face production normalization separately
from source-semantic geometry.

Representative exact records are:

```text
layer 0, first face-order line:
  (17_530_508, -25_999_317, Edge(0))
    -> (17_983_121, -25_954_736, Edge(1)), General

layer 0, first content-sorted line:
  (-37_500_000, -33_000_000, Edge(6691))
    -> (-37_469_924, -33_343_825, Edge(6982)), General

layer 2, owned top edge:
  (17_043_610, -26_369_232, Vertex(4))
    -> (17_652_542, -26_396_576, Vertex(0)), Top

layer 37, owned top edge:
  (17_043_610, -26_369_232, Vertex(11))
    -> (17_652_542, -26_396_576, Vertex(5)), Top

layer 459, first content-sorted line:
  (2_196_466, -30_303_541, Edge(11738))
    -> (2_201_466, -30_303_541, Edge(11741)), General
```

The test-only closed-component count uses the complete endpoint key
`(point.x, point.y, reference_kind, reference_id)`. Every line adds one directed
multigraph edge A to B with parallel multiplicity retained. Components are
found through the underlying undirected incidence graph; a nonempty component
is closed only when every node has indegree 1 and outdegree 1. A self-edge adds
one incoming and one outgoing incidence, although KSR separately requires no
zero-length line. Joining by coordinates alone or provenance alone is
forbidden.

### GREEN and regression gates

After every approved implementation package and again on the frozen whole
implementation:

```powershell
cargo +1.91.0 nextest run -p ares-core geometry
cargo +1.91.0 nextest run -p ares-core mesh_slicer
cargo +1.91.0 nextest run -p ares-core project_slice
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
```

The whole release gate additionally runs workspace nextest, CLI/WASM checks,
release browser bindings, the real-3MF browser test, fixture/hash guards,
forbidden-pattern scans, diff/whitespace checks, and the physical-LOC audit.
The implementation plan must freeze exact commands and counts from its current
baseline rather than relying only on this abbreviated list.

## Explicit deferrals

Task 22B defers all of the following without authorizing a fallback:

- distinct print-object transform-group center rotation and any structured
  transformation decomposition needed to support it; the explicit centering
  gate is included and may not be deferred;
- nonidentity `filament_shrink` and `filament_shrinkage_compensation_z`; both
  remain rejected by their existing exact option-keyed effective-config gates;
- full `LayerRangeRegions` volume membership, f32 slab-bbox projection, and
  lower-closed/upper-open plane filtering; nonempty typed ranges are explicitly
  gated before Task 22B geometry, with no unfiltered fallback;
- importer-global `mesh_shared` cache-key/value retention, first-seen centered
  mesh reuse, saved `init_shift`, the `-1` remap branch, and the distinct shared
  transform compensation; explicit sharing and repeated numeric nonempty leaf
  IDs are gated as `shared_mesh_centering`, with no fresh-mesh fallback;
- Orca's process-global absolute `ObjectID` numbers and unrelated allocation
  gaps; Task 22B retains only the per-source-object creation-order semantics it
  consumes through typed `VolumeOrdinal`;
- normalized edge groups with more than two uses and their upstream-undefined
  equal-key pairing order; the explicit `mesh_topology` gate is included;
- `Line`, `Polyline`, `Polygon`, `ExPolygon`, bounding-box, area, containment,
  orientation, and non-clipping path-domain behavior beyond `Coord`/`Point`;
- segment chaining by edge/vertex identity, greedy seed flags, exact open-chain
  joining, gap repair, loop construction, and path ordering;
- Clipper 6 boolean, PolyTree, fill rules, union, offset, simplification,
  closing, contour/hole construction, and deterministic polygon ordering;
- consuming `slicing_mode`, `slice_closing_radius`, `resolution`, or XY
  compensation in geometry; these remain typed in the resolved 3MF and are
  consumed only by their later source slices;
- negative/modifier boolean application, range/region assignment, painted
  segmentation, fuzzy skin, interlocking, conical overhang, slicing-error
  repair, and final layer cleanup;
- reproduction of any one Orca TBB raw-line append schedule; Ares face-order is
  an explicit deterministic normalization, while later chaining and observable
  ordering retain independent parity gates;
- surfaces, elephant-foot compensation, perimeters, fill, brim, supports,
  toolpaths, motion, G-code assembly, generated-by metadata, time estimation,
  and post-processing;
- embedded preset extraction, external preset discovery/management, CLI
  overrides, UI behavior, and any Ares-owned alternative slicing pipeline;
- successful normalized `ksr_fdmtest_v4` G-code parity.

The persistent full parity goal remains active after this task.

## Independent review, documentation, and release gates

1. Freeze this spec by path, byte count, and SHA-256.
2. Obtain literal `VERDICT: APPROVE` from an independent Codex reviewer and a
   default-model OpenCode reviewer for the exact frozen bytes. Any edit
   invalidates both approvals.
3. Author and freeze a Subagent-Driven TDD implementation plan only after both
   spec approvals.
4. Obtain literal plan `VERDICT: APPROVE` from a different independent Codex
   reviewer and the default-model OpenCode reviewer. Any edit invalidates both.
5. Execute the approved plan with bounded implementer subagents. Each package
   requires independent specification-compliance and code-quality `APPROVE`
   verdicts before the next dependent package begins.
6. Freeze the whole implementation and obtain independent whole
   specification-compliance, whole code-quality, and default-model OpenCode
   implementation approvals. Repeat correction and review until all three say
   `APPROVE`.
7. Only then update `docs/architecture/option-parity-v4.md` and
   `docs/roadmap.md`. Correct their stale Task 22A release record to commit
   `91fc19f1dbfc85d21431791d2d5acb78af818671` and Tier 1 run `29543841835`,
   document only approved Task 22B behavior/deferrals, and obtain an independent
   documentation `APPROVE`.
8. Run the complete post-documentation local release matrix, freeze exact
   tracked bytes, and create a reviewed Conventional Commit.
9. Push normally, verify local/tracking/direct-remote SHA equality, and require
   exact-SHA Tier 1 success across format, Ubuntu/Linux, WASM, macOS, and
   Windows before recording Task 22B as released.

No implementation, documentation-completion claim, commit, push, or Task 22B
release claim may bypass these gates.
