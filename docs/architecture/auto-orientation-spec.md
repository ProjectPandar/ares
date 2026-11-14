# Automatic mesh orientation

## Source boundary

This is a Rust rewrite of OrcaSlicer 2.5.0 at
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, not an independently designed
orientation heuristic or slicing pipeline. Line references below use that pin.

| Upstream owner | Included behavior | Rust destination |
| --- | --- | --- |
| `src/libslic3r/Orient.cpp:78–572`, `orientation::AutoOrienter` | Preprocessing, candidate generation, projection, feature extraction, scoring, selection and cooling-direction search | `crates/ares-core/src/plater/auto_orient/` |
| `Orient.hpp:34–106`, `OrientMesh`, `OrientParams` | Mesh/context inputs, source parameter defaults and rotation results | Public types exported through `ares_core::plater` |
| `Orient.cpp:574–649`, `_orient` and `orient` overloads | Mesh/instance adaptation and source rotation application | The same plater operation and its project adapter |
| `TriangleMesh.cpp:515–522,1358–1447,1625–1632`, `TriangleMesh.hpp:349–354` | Three-dimensional hull construction, facet winding and face normals | Focused kernels under `ares-core/src/geometry/`, called by the plater operation |
| `Geometry.cpp:405–424` | Euler extraction and quaternion-based rotation between two vectors | Existing core matrix/transform boundary, extended only where needed |
| `Model.cpp:1675–1701`, `ModelObject::mesh/raw_mesh` | Transformed model-part volumes and instances used as the scoring mesh | Project-to-orientation adapter in plater |
| `deps_src/admesh/stl.h:50–56,171–218` and `Format/bbs_3mf.cpp` triangle/property import | Face-property enum decoding and triangle association | Existing project mesh data consumed by the operation |
| `PrintConfig.cpp:12972–12976`, `OrcaSlicer.cpp:4973–5003,5271–5288` | Integer `--orient` action, object eligibility and application before slicing | `ares-cli/src/args.rs` and its existing project-slice adapter |

The source facade `orient(ModelObject*)` also calls `ModelObject::rotate` and
`ensure_on_bed`; their existing model-transform behavior is part of CLI
integration, not a substitute orientation algorithm.

## Architectural contract

The operation belongs to `ares-core::plater`, with reusable numerical kernels
in geometry. It accepts mesh/instance data and orientation parameters and
returns rotations. It neither slices nor writes files. The CLI loads bytes and
presets, invokes this core operation, applies the result at the source model
boundary, then calls the existing slicing API.

Expose source-shaped `OrientMesh` and `OrientParams` inputs and an orientation
result containing the chosen direction and double-precision rotation matrix.
Retain axis/angle and the optional vertical cooling rotation where the source
adapter needs them. Inputs borrow validated indexed geometry and associated
face data; instance/volume transforms and their coordinate space are explicit.
Do not infer a mesh's placement from a camera, selection widget or printer UI.
A batch preserves input-to-result association independently of scheduling.

Keep scoring separate from applying rotations. The raw-mesh entry leaves its
input unchanged. Project/instance adaptation must preserve the source's
transformation order, part filtering, instance aggregation, rotation origin,
mirrors and bed placement. In particular, the CLI source scores
`ModelObject::mesh()`, including transformed instances; it does not orient each
part or instance independently. Applying a single-instance result must follow
the cited instance overload rather than silently adopting object semantics.

All computation must run in browser WASM, Windows, macOS and Linux. No native
Qhull/Eigen dependency, C++ execution, filesystem access, terminal output or
renderer dependency belongs in core. Implement deterministic Quickhull in
geometry without adding a dependency. Do not add a candidate crate or a second
orientation service. Native comparison belongs only to the test adapter.

Existing plater transforms remain the application boundary, not an orientation
scaffold to extend with a different heuristic. Existing
`geometry/convex_hull.rs` computes a two-dimensional hull and cannot stand in
for the required three-dimensional mesh. Opaque `ProjectMesh::face_properties`
remain available; their storage is a compatibility representation of upstream
`FaceProperty`, not evidence that orientation is already implemented.

## Algorithm contract

### Mesh preparation and candidates

Port both constructor contexts. The raw `TriangleMesh*` constructor uses
`OrientParams` defaults; the `OrientMesh*` constructor sets `ASCENT` from that
mesh's overhang angle and supplies cooling context. Do not conflate the
`OrientMesh` overhang default, the parameter member with that name, and the
raw constructor's `ASCENT` default.

`preprocess` at208–254 computes float face normals and geometric facet areas,
quantizes normal components with `floor(component * 1000) / 1000`, records
`eExteriorAppearance`, and builds the three-dimensional convex hull with its
own normals and areas. Preserve the source's normal normalization operations
and actual, rather than quantized, normals used for scoring.

Follow `TriangleMesh::convex_hull_3d` and `its_convex_hull`: the Native wrapper
calls Qhull with `Qt`, collects triangulated facets, maps vertices, and corrects
winding using the hull's outward normal. Implement deterministic Quickhull
with triangulated facets and outward normals in core geometry. Scoring uses
quantized normal-group area sums and the actual normal of the largest facet
in each group. Different triangulation of a coplanar face preserves these
quantities apart from floating-point noise; complete Native output witnesses
must establish observable agreement without relaxing the comparator. Preserve
facet winding and candidate selection effects. Do not substitute PCA, a bounding
box, a two-dimensional projection or Native execution.

`process`, `area_cumulation_accurate`, `add_supplements` and
`remove_duplicates` at134–205,281–344 must preserve:

- The initial downward candidate, up to ten mesh and fourteen hull normal-group
  candidates, and the eighteen supplemental directions in source order.
- Quantized grouping with accumulated geometric area, choosing the actual
  normal of the largest facet; equal facet areas do not replace the first
  representative.
- The source group-area ranking and candidate cutoffs before duplicate removal.
- Eigen's relative `isApprox` behavior at the source tolerance, removal of zero
  directions, retention of the first duplicate representative, and candidate
  sign inversion before projection.

Unused `area_cumulation`, `argsort`, and the disabled alternative contour code
are not additional active algorithm paths to resurrect.

### Features, scores and rotation selection

`project_vertices` at346–373 projects each triangle's vertices and derives the
maximum, median and mean height, plus hull maximum heights. Preserve the source
expression order and scalar widths.

`get_features` at402–489 includes both overhang modes, first-layer and half-layer
bottom masks, geometric bottom and hull-bottom area, contour `4 * sqrt(bottom)`,
low-angle faces, mesh statistics and optional cooling-face weights. The stored
`FaceProperty.area` is not the triangle's scoring area. Exterior appearance
multiplies that triangle's overhang contribution by
`1 + APPERANCE_FACE_SUPP` at427–428; it is not a new rejection or placement rule.
Empty properties retain the normal-face meaning. Decode valid property enums
without changing their triangle order or the public opaque strings.

Port `target_function` at491–510, including source constants, both `min_volume`
branches, low-angle selection and the bottom-area penalty. Do not substitute a
support-volume estimator or tune weights to a fixture.

Preserve the score ordering and upright-preference scan at172–202, including
its exact bounds and epsilon comparisons. Select cooling weights from the
orientation actually chosen, not necessarily the first ranked entry. Convert
the chosen float direction to double and use the source quaternion/axis-angle
rotation to positive Z, including parallel and antiparallel vector behavior.
Do not replace this with Euler-angle guesses or a minimum-angle heuristic.

### Cooling and operation modes

Include `AutoOrienter::find_cooling_direction2` at512–571: transform the mesh
with the selected Euler rotation, sample 180 horizontal directions, accumulate
weighted positive face projections in order, retain strict-improvement
selection, and preserve the source's near-symmetry preference for the machine
cooling direction. Keep this context explicit rather than deriving it from
available CPU cores.

The source `_orient` serial and parallel branches do not populate identical
auxiliary results: Euler and vertical cooling rotation are computed in the
parallel branch. Model that operation context explicitly. Changing the number
of execution workers must not enable cooling, change scoring parameters, or
change a requested result. Preserve each source adapter's meaningful result
fields; do not copy this semantic difference into a worker-count switch.
The separate free function `orient_for_cooling` at651–738 is adjacent behavior,
not the class method; its independent mesh-mutating action is deferred.

## Floating-point and ordering determinism

The same request must return the same rotation and lead to the same output
with one worker or multiple workers. Core count, hash randomization and task
completion order cannot select an orientation.

- Preserve float mesh coordinates, normals, areas, projections and costs,
  double parameters/conversions where used, source epsilon boundaries, and
  final double rotation construction. Do not globally promote the algorithm
  to double or introduce fast-math/FMA reassociation.
- Keep within-mesh accumulation and reduction order fixed. Reproduce relevant
  Eigen float reductions, rather than assuming an arbitrary parallel sum or
  Rust iterator sum has the same rounding. Parallelism may distribute
  independent requests without changing their local arithmetic or result order.
- Audit all order-sensitive stages: Qhull facet traversal, quantized normal
  buckets, area-ranked top-ten/top-fourteen cutoffs, duplicate retention,
  score-ranked selection, and cooling sampling. Fixed arithmetic alone is not
  sufficient.
- Native `unordered_map` traversal and equal-key `std::sort` ordering are
  library implementation details, not libslic3r semantics to reverse-engineer.
  Preserve first insertion order for normal groups and candidates, rank group
  areas stably, and sort candidate scores stably. Preserve the source's
  upright-preference scan separately. The decision is recorded in
  `ard-auto-orientation-determinism.md`.
- Native comparison meshes must be asymmetric with no ranking/selection tie.
  Symmetric and tied inputs assert only Ares reproducibility and worker-count
  invariance; they must not assert equality or inequality against Native.
  Near-epsilon non-tied witnesses retain complete output comparison.

Portable arithmetic must be checked on the four supported targets; differing
output on Native comparison witnesses requires diagnosis, not a wider
whole-output comparator tolerance.

## CLI action and saved transforms

Add `ares slice --orient <integer>` as an action, not a print-option override.
Use the source integer default of zero. Zero requests no orientation, one
requests orientation, and other integers leave the source automatic decision
unchanged; do not restrict this to a boolean or invent a new auto decision.
An absent action must preserve existing saved-transform slicing behavior.

Eligibility is keyed by current object ID. The apparent BBS-load insert(false)
at `OrcaSlicer.cpp:1769–1773` is inside a block comment and does not execute.
The map starts empty, so the action's insert at4983/4994 also works for BBS
projects: zero inserts false, one inserts true, other values leave it unchanged.
The apparent transfer to copied IDs at4039–4051 is commented out too. At5275,
`operator[]` inserts false for any current ID without an entry. Track actual
object IDs across copy/assembly rather than inheriting an orientation flag
without a source write. Assembly at4911–4972 can create a new object; copy at
5004–5026 delegates duplication/arrangement, so distinguish new objects from
new instances and check whether the action precedes or follows that change.
Native runtime behavior is authoritative for these action-order controls.
Actual application at5271–5288 calls `orientation::orient(ModelObject*)` for
true entries and then clears eligibility.

Perform action adaptation before the existing slice request consumes the
model. Retain Native volume/instance application and `ensure_on_bed`, without
adding arrangement, centering, scaling or a default rotation. Reuse existing
explicit catalog/preset routes for equivalent slicing settings. CLI filesystem
and argument handling stay outside core.

## Included and deferred

Included: the complete active AutoOrienter algorithm and both scoring modes;
its hull, normals and rotation dependencies; appearance-aware overhang costs;
class cooling-direction calculation; raw mesh and instance-facing core inputs
and rotation outputs; the Native CLI action and its default/object eligibility;
source-compatible application followed by the existing slicing API.

Deferred: browser buttons, web-state transport and viewer integration; GUI jobs,
selection menus, progress presentation and cancellation UI; the standalone
`orient_for_cooling` action; arrangement, nesting, new bed fitting and unrelated
model repair; new orientation heuristics or configurable weight presets;
malformed face-property compatibility; CLI switches and Native API comparisons
for nondefault orientation parameters and cooling modes, which Native CLI does
not expose. The core algorithm retains those paths, without claiming CLI
coverage of them. Existing saved-transform project slicing
is not redefined. No filesystem discovery or legacy fallback is added to core.

## Native comparison E2E plan

After spec review, use public project loading, the exported plater orientation
API and the real CLI. Write each failing public-route witness before its
implementation. Generate small, repeatable synthetic meshes with fixed vertex,
triangle and transform order; retain the generator and input hashes. Do not
change product code, fixtures or tests as part of this specification commit.

| Mesh/control family | Behavior it must distinguish |
| --- | --- |
| Asymmetric wedge and non-convex overhang assembly | A real non-identity choice, mesh versus hull normal candidates, and applied bed placement |
| Identical geometry without a property, with an empty property, and with normal-face data | Normal-face equivalence and retained face association |
| Same asymmetric mesh with selected triangles carrying valid `eExteriorAppearance` data | Appearance weighting; select a case where Native changes the chosen rotation, not merely a printed cost |
| Same appearance type with different valid serialized property-area values | Geometric facet area is used, rather than the stored annotation area |
| Symmetric solids and many equal-area face directions (Ares only) | Stable insertion-order ties, reproducibility and worker-count invariance; no Native equality or inequality assertion |
| Near quantization, duplicate-normal, bottom-height and scoring thresholds | Exact float casts, relative duplicate tests and epsilon comparisons |
| Multi-part objects, rotated/scaled/mirrored instances and several independent objects | Source part filtering, transform order, object aggregation and result association |
| Fresh import versus saved BBS project, with absent action and `--orient 0`, `1` and another integer | Actual Native action eligibility, disabled/automatic controls and saved-transform preservation |

For each CLI-reachable positive case, run the pinned Native CLI with `--orient
1` and an equivalent catalog-backed Ares invocation. Prove Native reached the
orientation operation using its retained orientation output and an
output-changing control; argument presence is insufficient. Synthetic BBS 3MFs
with and without appearance data are positive action witnesses, not assumed
no-ops. Record actual object IDs and outcomes for copy/assembly controls; an
unmapped new ID is a disabled control, not an appearance witness.

Compare the complete ordered G-code emitted after the action through Native,
core API and Ares CLI. Only the existing generated-by normalization is allowed.
Do not strip configuration blocks, progress commands, whitespace, line order,
or orientation-dependent geometry. Retain complete oriented model/transform
output when exposed by the Native CLI as well; compare the resulting geometry
and rotation, not only the rounded `best:` stdout vector. Native stdout is
supporting evidence and is not a matrix oracle.

Run both established Native modes with equivalent inputs and presets. Repeat
single-worker and four-worker execution inside CPUs16–19, keeping build jobs
and test threads at four or fewer. Compare each repeat's complete output and
public rotation result, including input/result association for a batch. Retain
commands, environment/producer identities, input hashes, all outputs, first
differences, exit status and a machine-verifiable manifest. Include Windows,
macOS, Linux and browser-WASM API checks without adding a browser button.

The Native CLI calls the raw-mesh/default-parameter overload, not the whole
`OrientMesh` API. Nondefault `min_volume`, per-mesh overhang settings and class
cooling context are deferred comparison modes; do not add a Native comparison
entry or CLI flags for them. CLI output comparisons do not establish coverage
of those parameters. First write the CLI default/zero/one failing witnesses,
then implement the source operation and expand the approved cases one at a time.

Implementation requires formatting, workspace all-target Clippy, WASM
compilation and Native E2E evidence. This specification makes no runtime or
orientation-parity claim.
