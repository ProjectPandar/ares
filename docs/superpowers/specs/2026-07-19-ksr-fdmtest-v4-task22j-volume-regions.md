# Task 22J: Single-Range Volume Region Composition

## Status and objective

This specification is a draft. Production or tracked test implementation may
begin only after the exact specification and implementation-plan bytes receive
independent fixed-source/specification, independent current-Ares/plan, and
direct default-model approval.

Task 22J is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`eb3aa56118d75c970886d46952fdfde1f8b198b1` produces the exact ordered Task
22I post-simplification `ExPolygon` stream. Task 22J ports the accepted
single-layer-range part of OrcaSlicer's volume-region graph, occurrence-ID
`VolumeSlices` sidecar, `slices_to_regions`, and append-as-`stInternal` caller
boundary.

The stage consumes only the loaded 3MF project, its resolved typed Options,
the selected coordinate scale, and the released Task 22I geometry. It never
reads a filename, fixture digest, reference G-code, process-global default, or
out-of-band test parameter. `clip_multipart_objects=true` is fixed upstream
behavior, not a new Option.

Task 22J stops immediately after every composed ExPolygon has become a private
internal surface. It does not remove top empty layers, classify surfaces,
generate perimeters, infill, supports, extrusion paths, or G-code. The public
project API executes the new stage and continues to return
`SliceError::ProjectSlicingIncomplete`.

## Fixed Ares and upstream identity

The fixed Ares baseline is commit
`eb3aa56118d75c970886d46952fdfde1f8b198b1`, tree
`35b99bc2ad16abc4a37e09dd6d62b6494cafc075`. Exact-SHA Tier-1 run
`29676205957` passed Windows, Ubuntu/Linux, macOS, format, and WASM.

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored Orca checkout is
evidence only; tracked tests never inspect it.

Fixed source blobs are:

- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/Print.hpp`,
  `c69c5b6570a79cb750c08805e4907eeec5c834f5`;
- `src/libslic3r/PrintApply.cpp`,
  `a80ad6f7300b4a03fe9f5d492ecf49fb22b35d4a`;
- `src/libslic3r/PrintObject.cpp`,
  `925da0c5644e06b6813747ae35b371d1a1555fe1`;
- `src/libslic3r/PrintConfig.hpp`,
  `0a7b7ba36f87c3d4517daf96d7d8825812e66358`;
- `src/libslic3r/Model.hpp`,
  `d8697adb41307ac2cdb018c440f1afac75f01356`;
- `src/libslic3r/ObjectID.hpp`,
  `94fbb6a0abd57e27ab9d5a068bb8350868ff553f`;
- `src/libslic3r/ClipperUtils.hpp`,
  `9c2fa239263c0cb097a4b4c3db823821615bd7c7`;
- `src/libslic3r/ClipperUtils.cpp`,
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`;
- `src/libslic3r/Layer.hpp`,
  `cb2e6c7c1a166a028ac8fceffaf9f42f3c2426b0`;
- `src/libslic3r/Surface.hpp`,
  `b63c283251d63154a3a7071694c87b637ec7dff7`;
- `src/libslic3r/SurfaceCollection.hpp`,
  `1895516aa2eb1fa30be3cf63bb211f7db420f3af`;
- `src/libslic3r/ExPolygon.hpp`,
  `ce7ebe892f64b3d4e2e9fb0c85bd77b99e889d54`;
- `src/libslic3r/Format/bbs_3mf.cpp`,
  `31519e4fbed8427c115344ec124c6b8250db67c6`;
- `deps_src/clipper/clipper.hpp`,
  `06637effce040fa7d87c368437cb32398f19ee92`;
- `deps_src/clipper/clipper.cpp`,
  `1f16446ac8da1f0b9c802d8a9dee33f766919f6b`.

There is no fixed `PrintObjectSlice.hpp`; the relevant declarations live in
`Print.hpp`. No new third-party crate or second geometry engine is introduced.
Ares extends its released source-cited Clipper 6.4.2 rewrite under the existing
BSL-1.0 provenance.

## Exact upstream rewrite boundary

The accepted single-implicit-range layout boundary is:

- `Print.hpp:44-48,102-120,216-305,423-427,516-519,553-555` for
  `VolumeSlices`, `VolumeExtents`, `VolumeRegion`, region identity/config,
  shared ownership, the caller declaration, and their two distinct orders;
- the implicit `[0, DBL_MAX)` arm of `PrintApply.cpp:342-405` for one region
  layout with no loaded layer-range config;
- `PrintApply.cpp:542-545,548-553,582-592,887-910` for participating volume
  kinds, composed transforms, zeroed XY translation, `f32` bounds, referenced
  mesh vertices, and full-mesh extents;
- `PrintApply.cpp:699-724,958-1057` for runtime-`ObjectID` extent lookup, modifier
  ancestor bounds, first-created region identity, and source-order model-part,
  negative, and modifier records;
- `PrintObject.cpp:3555-3710` for model-part and modifier region Option
  resolution and normalized feature-filament fallback;
- `PrintApply.cpp:1727-1739` for the actual first resolved print-instance
  transform, layer ranges, filament count, painted compensation input, and
  fuzzy-skin-painted input passed to region generation;
- `Format/bbs_3mf.cpp:315-332,3804-3826,4867-5116,7315-7346,7815-7899`
  for fixed BBS painted-triangle vocabulary, breadth-first volume construction,
  import into facet/config storage, export, and absence of serialized
  `ModelMaterial` association.

The exact composition and caller boundary is:

- `PrintObjectSlice.cpp:21` and `Print.hpp:585-590` for the static true
  multipart clipping behavior;
- `PrintObjectSlice.cpp:231-241` for runtime-`ObjectID` slice lookup and
  inclusive XY overlap;
- `PrintObjectSlice.cpp:269-480` for the fast classifier, complex composition,
  modifier forwarding, negative/later-part subtraction, ordering, same-region
  merge, and epsilon closing;
- `PrintObjectSlice.cpp:1149-1192` for layer-region allocation, complete
  `VolumeSlices` sidecar copy, destructive composition, and append as
  `stInternal` before the top-trimming loop;
- `ClipperUtils.hpp:400-410` and `ClipperUtils.cpp:550-584` for the exact
  `closing_ex`/`offset2_ex` merge repair;
- `ClipperUtils.cpp:640-667,737-803` for Difference and Intersection Paths
  execution followed by fresh NonZero PolyTree union;
- `Surface.hpp:9-47`, `SurfaceCollection.hpp:65-81`, and
  `Layer.hpp:33-48,335-341`
  for `stInternal=4`, surface defaults, ordered append, and layer-region
  ownership, plus the exact `Layer::slice_z` to `float` activity vector;
- `ExPolygon.hpp:12-36`, `Model.hpp:341-348,901-910,1227-1236`, and
  `ObjectID.hpp:13-17,64-86` for owned geometry, exact volume-kind predicates,
  and generated runtime identity;
- `libslic3r.h:42-60,91-96,143-162` for `EPSILON`, scale selection, and
  ordered move conversion.

The unused `PrintConfig` and `PrintObject` parameters of `slices_to_regions`
do not become Rust dependencies. Cancellation and TBB scheduling are not
observable outputs and do not enter platform-neutral `ares-core`.

## Single implicit range boundary

Task 22J accepts exactly the domain that reaches released Ares Task 22I:

- one implicit `[0, DBL_MAX)` layer range;
- no layer-range configuration override;
- arbitrary model-part, negative, and parameter-modifier volumes;
- arbitrary private occurrence IDs and source order at the composition seam;
- all currently accepted model-part and modifier Region Options.

Released `raw_intersections` rejects when a loaded
`ProjectObject::layer_config_ranges()` vector is nonempty before mesh
projection. A nonempty XML document such as `<objects/>` may still load an
empty vector and is accepted. Task 22J preserves the exact
`UnsupportedProjectFeature("layer_config_ranges")` gate, and absent versus
loaded-empty range documents must produce identical I/J bytes. It does not add
a dormant range-layout vector or silently accept half of the upstream range
pipeline.

Production planning cannot escape the fixed range or create a zero-layer
print object. It starts with `[0, first_object_layer_height]`, that height is
positive and finite, later endpoints strictly advance, and `slice_z` is the
finite midpoint. Objects with no resolved print instance are omitted. Thus
every production `PlannedPrintObject` has at least one nonnegative `slice_z`;
Task 22J trusts that released invariant and adds no unreachable zero-layer
fallback.

The cohesive future range slice must port together:

- `PrintApply.cpp:342-405,595-662,911-947` range normalization and slab-clipped
  `f32` bounds;
- `PrintObjectSlice.cpp:98-136,162-164,212-223` selective range slicing;
- `PrintObjectSlice.cpp:244-267,289-406` shared-endpoint selection and range
  transitions;
- cross-range region registry and configuration behavior.

This deferral is Option-boundary specialization, not fixture specialization.
All accepted single-implicit-range multi-volume behavior remains mandatory.

## Option and capability ownership

Task 22J introduces no new parser, default, environment input, or public
Option. It consumes:

- kind, mesh, transform, raw resource ID, and source order from the 3MF;
- the released Task 22B nonzero occurrence ordinal carried by post-I geometry;
- existing resolved model-part `RegionOptions`;
- accepted parameter-modifier overrides through `RegionBase::Modifier`;
- resolved object `xy_contour_compensation` for volume bounds;
- `logical_filament_count` for existing Region Option normalization;
- the already selected `CoordinateScale` for epsilon closing.

The existing ten usage-affecting modifier rejection gates remain unchanged:

- `wall_loops`;
- `sparse_infill_density`;
- `top_shell_layers`;
- `bottom_shell_layers`;
- `sparse_infill_filament_id`;
- `internal_solid_filament_id`;
- `top_surface_filament_id`;
- `bottom_surface_filament_id`;
- `outer_wall_filament_id`;
- `inner_wall_filament_id`.

The first four may activate feature filament usage inherited by the six
selectors. Task 22J does not expand usage collection, so lifting only a subset
would make the existing typed usage contract incomplete.

Two other released pre-J gates remain unchanged. Any nonempty volume marked
mesh-shared, or any repeated raw resource ID among request-wide nonempty
volumes, returns `UnsupportedProjectFeature("shared_mesh_centering")`.
Distinct resolved print groups or transforms that cannot collapse to the
first transform without XY translation return
`UnsupportedProjectFeature("print_object_centering")`. These are accepted
boundary gates, not Task 22J identity providers.

BBS triangle attributes `paint_color` and `paint_fuzzy_skin` remain outside
Ares' accepted XML vocabulary before Task 22J. Complete archive mutations add
one unprefixed `paint_color` or `paint_fuzzy_skin` attribute to a triangle and
must prove both fail closed through `load_project` and the real project path
with exactly
`SliceError::InvalidInput("invalid project model XML: attribute namespace does not match its 3MF meaning")`
rather than being silently discarded. Fixed Orca reads and writes those
attributes, but its fixed BBS importer/exporter routes volume metadata into
volume config and does not serialize Orca `ModelVolume` material-config
association. `material=None` is therefore the exact accepted BBS 3MF boundary,
not an ignored loaded field. Core `pid` projection to an extruder is a
different typed behavior.

## Volume occurrence identity and retained sidecar

Fixed Orca uses generated process-global `ModelVolume::ObjectID`. Task 22J
observes only equality and relative ordering within one `PrintObject`. Ares'
`ProjectVolume::id()` is the raw 3MF resource ID and is not that runtime
identity. Task 22J therefore promotes released Task 22B `VolumeOrdinal` to a
private per-object `VolumeOccurrenceId`. It preserves flattened breadth-first
construction order and gaps from nonempty support volumes without claiming
numeric equality with Orca's process-global IDs. `source_volume_index` joins
geometry to source metadata and Options; neither it nor raw resource ID is the
composition sort key.

For each print object:

1. Consume every post-I volume, retain its nonzero occurrence ordinal, and use
   `source_volume_index` only to join the source volume.
2. Drop slicing mode and volume kind from the geometry carrier.
3. Build `VolumeSlices { volume_occurrence_id, layers }` and sort it by
   occurrence ID.
4. Clone the complete sorted geometry as the production sidecar corresponding
   to Orca's `firstLayerObjSliceByVolume = objSliceByVolume`.
5. Destructively compose separately owned slices into regions.
6. Preserve every planned layer in both representations, including empty
   middle and final layers.

The sidecar contains no label suggesting it is first-layer-only; fixed Orca
copies every layer. A direct algorithm vector uses occurrence IDs 90 then 10
to make source and identity order disagree; that vector tests the generic
composer and is not claimed as loader-produced order. A real loaded
`[3,1,2]` raw-ID project must instead yield occurrence IDs `[1,2,3]`, proving
raw resource IDs do not reorder carriers. A model/support/modifier vector must
retain the support gap as occurrence IDs `[1,3]`.

## Full-mesh volume bounds

Only model-part, negative, and parameter-modifier volumes receive Task 22J
bounds. Support enforcers and blockers remain outside this graph.

For each participating source volume:

1. Compose the resolved first print instance's transform and volume transform
   in `f64` in the same order as fixed Orca's
   `print_instances.front().trafo` caller input.
2. Zero the combined X and Y translation while retaining Z translation.
3. Narrow matrix coefficients and mesh vertices to `f32` before multiply-add.
4. Inspect only vertices referenced by triangles; unreferenced extremes do not
   affect the box.
5. Inflate X and Y by
   `max(0.0_f32, xy_contour_compensation as f32)`.
6. Inflate minimum and maximum Z by `EPSILON` in opposite directions.

Z activity is inclusive. XY bbox overlap is also inclusive: touching boxes
overlap because only strict separation returns false. Internal callers trust
that accepted project meshes contain referenced triangles.

## Region registry and source-order graph

Task 22J maintains two independent orders:

- volume bounds and `VolumeSlices` lookup are sorted by occurrence ID;
- `volume_regions` records remain in source `ProjectObject::volumes()` order.

`all_regions` IDs use first-encounter order. Exact-equal `RegionOptions` reuse
the existing ID. A linear equality lookup is sufficient and avoids a second
hash identity that could diverge from `PartialEq`.

For each source volume:

- A model part adds one record with its already resolved single-implicit-range
  `RegionOptions`, no parent, its occurrence ID, and its bounds.
- A negative adds one record with no region and no parent.
- A modifier scans preceding model-part/modifier records in reverse.

For each potential modifier parent:

1. Extend the parent's bounds through modifier ancestors to its top model
   part.
2. Require inclusive bbox intersection with the modifier.
3. Resolve from the parent's `RegionOptions`, the modifier volume overrides,
   `material=None`, and the existing filament normalization context.
4. Add a record for every intersecting parent whose resolved config differs.

If no intersecting parent changes, add exactly one no-op fallback for the last
intersecting model-part parent and reuse its region. If any changed parent was
added, do not add unchanged fallbacks for other parents. If no model-part
ancestor intersects, add nothing.

Records for one modifier are consecutive. This is required by the following
composition forwarding rule.

## Fast layer classification

The output is a dense matrix of every region and every planned layer. For each
`plan.layers[layer_index].slice_z as f32` (never `print_z`):

- no graph records leaves every region empty;
- one record moves slices only when that record is a model part;
- multiple records first find the first active model part in source order.

For each later active record, compare its inclusive XY bounds against every
active record from that first model part through its predecessor. Any overlap
makes the layer complex. If none overlap, move only the first active model
part's slices. This surprising omission of later disjoint model parts is fixed
source behavior and has a required synthetic regression.

An active negative before the first active model part does not affect that
later part. A single modifier or negative emits no printable region.

## Complex layer composition

For a complex layer, create temporary records in `volume_regions` source
order. Each record owns:

- its region ID or no region;
- its volume occurrence ID;
- the moved ExPolygons for that physical volume and layer.

Repeated records for one modifier initially observe the same source carrier;
only the first move is nonempty. Modifier handling therefore:

1. Moves the current modifier source into a local value.
2. Clears the modifier result when its parent is empty.
3. Otherwise assigns `intersection(parent, source)` to the modifier and
   `difference(parent, source)` to the parent.
4. If the immediately following record is the same physical modifier, forwards
   the untouched source into that next record.

For each later model part, because clipping is statically enabled, and for each
negative volume, subtract its slices from every preceding nonempty,
non-negative, XY-overlapping record. Later model parts win. A negative affects
only preceding records and never emits a region. The active fixed `#if 1`
Difference path is required; the disabled `trim_overlap` branch and safety
offset are not ported.

After composition:

- invalid or empty records sort to the tail;
- valid records sort lexicographically by
  `(region_id, volume_occurrence_id)`;
- negative and empty records are discarded;
- same-region ExPolygons append in that order;
- if at least two nonempty records actually merge, apply
  `closing_ex(expolygons, float(scale_(EPSILON)), Miter, 3)` once;
- store the resulting ordered ExPolygons in the dense region/layer slot.

Fixed C++ supplies no total order for comparator-equivalent duplicate
`(region_id, volume_occurrence_id)` records. Ares preserves their source-record
order deterministically and does not claim that tie order as cross-toolchain
Orca behavior. No tracked oracle depends on an equal-key tie.

Layers execute sequentially in ascending index. TBB scheduling is not
observable because each complex iteration writes a unique layer slot.

## Binary Boolean and closing contract

Task 22J adds private ExPolygon Difference and Intersection wrappers over the
released Clipper engine. Each wrapper executes exactly:

1. Difference or Intersection to `Paths`, with subject and clip both NonZero.
2. Return empty if that Paths result is empty.
3. Feed those exact paths into one fresh NonZero Union that outputs a
   `PolyTree`.
4. Convert the tree through the released ordered ownership traversal.

Calling the released `union_ex` after step 1 is wrong because it would add an
extra Paths union. No safety offset, epsilon, canonical sort, second geometry
engine, FFI, or platform-specific library is allowed.

Merged-region closing uses:

```text
delta = float(EPSILON / coordinate_scale.factor)
```

This is exactly `100.0` at Normal scale and `10.0` at LargeBed scale. It is a
floating scale operation, not integer `checked_scale`. It runs only after an
actual same-region append, never for a single record.

## Internal surface contract and stopping point

Task 22J creates a private project-slicing surface type rather than reusing the
dormant Ares `surface` or `pipeline` domains. Every composed ExPolygon becomes
one surface with:

- type `Internal`, encoded as fixed upstream discriminant `4`;
- `thickness=-1.0`;
- `thickness_layers=1`;
- `bridge_angle=-1.0`;
- `extra_perimeters=0`;
- owned ExPolygon geometry in existing order.

Empty region slots remain present with zero surfaces. Surface order is
ExPolygon order. Task 22J stops after this append and before
`PrintObjectSlice.cpp:1194` top-empty-layer removal. Consequently an empty
final planned layer is retained and remains addressable.

## Ares destination boundary

Task 22J remains private, byte-oriented, and platform-neutral:

- `geometry/clipper/boolean_ex.rs` owns exact Difference and Intersection
  ExPolygon wrappers;
- `project_slice/planning.rs` is a behavior-neutral real-module extraction of
  the existing planning seam from the 336-LOC root;
- `project_slice/volume_bounds.rs` owns the private occurrence identity,
  post-I source lookup, and accepted full-mesh transformed bounds;
- `project_slice/volume_regions.rs` owns the single implicit-range registry and
  source-order graph;
- `project_slice/region_slices.rs` owns the complete occurrence-keyed sidecar,
  dense output types, fast dispatch, and internal surfaces;
- `project_slice/region_slices/complex.rs` owns exact complex composition,
  sort/merge, and closing;
- `project_slice.rs` invokes Task 22J after Task 22I and feeds the public
  incomplete lifecycle;
- `project_slice/task22j_oracle.rs` emits the released complete stage with
  `ARES22J\0` magic.

The non-default browser feature becomes `task22j-browser-oracle`. It exposes
exactly a post-I input checkpoint and post-J output checkpoint through
`ares-core` and `ares-wasm`. The Task 22I browser feature and exports are
removed without aliases; native predecessor helpers remain under `cfg(test)`.
The feature controls visibility only, never algorithms or Options.

The old Ares `pipeline`, `print_apply`, `print`, and `surface` implementations
are not Task 22J dependencies or fallbacks.

## Invariants and errors

Task 22J adds no public error variant. Private geometry callers map
`ClipperError::CoordinateOutOfRange` to:

```text
SliceError::InvalidInput(
  "project region composition polygon coordinate is outside the supported Clipper range"
)
```

Internal graph and output invariants are trusted:

- every post-I print object has the matching source and resolved object;
- every production post-I print object has at least one planned layer and each
  planned `slice_z` is finite and nonnegative;
- every participating source volume has one post-I carrier;
- every sidecar volume has exactly the planned layer count;
- occurrence IDs are nonzero and unique within each print object by released
  Task 22B construction;
- every parent index refers to an earlier graph record;
- every region layer vector has the planned layer count;
- every dense slot's region ID equals its vector index.

Empty geometry and empty layers are normal outcomes, not errors. External ZIP,
XML, transform, mesh, and typed Option validation remain owned by existing
project code. No new internal defensive fallback is added.

## Exact ARES22J protocol

All numeric fields are little-endian. Counts and IDs are `u64`; coordinates
are signed `i64`.

```text
8 bytes  magic = "ARES22J\0"
u64      object_count

repeat object_count:
  u64    source_object_index
  u64    transform_index
  u64    planned_layer_count

  u64    sidecar_volume_count
  repeat sidecar volumes in volume_occurrence_id order:
    u64  volume_occurrence_id
    u64  layer_count
    repeat every layer, including empty layers:
      u64 layer_index
      u64 expolygon_count
      repeat ExPolygon

  u64    retained_layer_count
  repeat every retained layer, including an empty final layer:
    u64  layer_index
    u64  region_count
    repeat every region in region-ID order, including empty regions:
      u64 region_id
      u64 surface_count
      repeat surfaces:
        u8 surface_type = 4
        ExPolygon

ExPolygon:
  Polygon contour
  u64 hole_count
  repeat Polygon hole

Polygon:
  u64 point_count
  repeat points:
    i64 x
    i64 y
```

There is no padding, footer, slicing mode, raw resource ID, volume kind, or
serialized `RegionOptions`. Sidecar and retained layer counts equal
`planned_layer_count`; every retained layer contains every region slot; exact
EOF is mandatory.

## Fixed-source oracle protocol

The ignored C++20 probe mechanically isolates the fixed Boolean and
`slices_to_regions` behavior. Its seven declared source files have aggregate
filename-NUL-content SHA-256
`19b8760f9017b4f5a2ec84327b3f6b4a645f19e0a543ca135fd6905c6ddf4406`.
The exact framing order is `oracle.hpp`, `oracle_geometry.cpp`,
`oracle_regions.cpp`, `oracle_cases.cpp`, `oracle_reader.cpp`,
`oracle_io.cpp`, `main.cpp`; each UTF-8 basename is followed by one NUL byte
and then that file's raw bytes. No lexical reordering is allowed.
MSVC 19.44 builds it with `/std:c++20 /EHsc /O2 /fp:precise /W4 /WX
/DNDEBUG`; the 470,528-byte executable has SHA-256
`3ef9c7d98e92039c04afbac9f356877493b4935b77a0ac50623f2654e7cec3cf`.

The binary SHA-checks and fully parses the released fixed-source `ARES22I`
input, including every field and exact EOF. Stateful coordinate reads are
sequenced through explicit X and Y locals before constructing a point. It then
applies the same accepted single-part graph and Task 22J caller/encoder. Five
runs are byte-identical.

Its synthetic stream contains 10 objects covering eight families:

1. one implicit-range model part with nonempty, empty, and nonempty layers;
2. two active XY-disjoint model parts where only the first moves;
3. overlapping source-order occurrence IDs 90 then 10, proving identity
   lookup does not replace source clipping priority;
4. a negative after and before a model part;
5. a two-level modifier chain;
6. one modifier source forwarded across two parents;
7. same-region closing at Normal and LargeBed scales with close/control gaps;
8. a final layer fully removed by a negative but retained in sidecar/output.

The exact synthetic stream is 5,880 bytes with SHA-256
`cb681dd4761dc69482f626374079f851ace0b9ec8d02587300c4495d84e0f4aa`.
Its complete ordered coordinate rendering has SHA-256
`938c8bcb02449c0ea77617973aed9b907313a2b0e4d9bb526c73ce158ee59691`.
All five runs reach exact EOF and every surface type is 4.

The probe, fixed checkout, generated archives, executable, and outputs are
ignored evidence only. Tracked Rust tests freeze independent literal vectors;
they never execute the probe or inspect source paths, line numbers, commits,
or hashes.

## Complete modifier/control loaded-input contract

A deterministic archive pair binds real loaded Option ownership to the Ares
complex path. The planned native Rust and Chromium tests load these complete
archives through the real project loader and resolver; they receive no Option
out of band. The builder starts from the committed KSR archive, retains its
complete project settings, and replaces only the model graph, model-settings
object, and two leaf meshes. The normal part is a closed 20-by-2-by-0.4 mm box
with raw resource ID 1. The following parameter modifier is a closed
10-by-2-by-0.4 mm box with raw resource ID 3 spanning the central X band. Both
transforms are identity. The variant contains only
`<metadata key="bridge_angle" value="37"/>` on the modifier; the control omits
that metadata and uses the KSR base value 0.

The deterministic Rust ZIP and sorted uncompressed
filename-NUL-content identities are:

Rust fixture archives fix every central-directory creator system to ZIP
`System::Dos`, independent of the host platform. This is test-container
metadata only; it does not supply a slicing Option or alter any uncompressed
3MF entry. The fixed creator system, sorted entry order, locked compression
stack, and 1980 timestamp together define the native ZIP identities below on
Windows, macOS, and Linux.

- variant: 56,046 bytes, ZIP SHA-256
  `83ac43d83487ad5f63b7c4b8f8c88ef20bb75b286d09e329fe24c8abc08807ce`,
  semantic SHA-256
  `82a7bdd3571da52daf92ec11a7a243ec279e9f053542804e2dfc1e10365d6fa3`;
- control: 56,027 bytes, ZIP SHA-256
  `4e1847cf020e217f9b90bef61cdb06c8fc2a953ca9dce100a161d3bcb99eca69`,
  semantic SHA-256
  `e59b8041e64297f880e19ab42b51cbbac9f9394bd3f287ffe845edba595176e5`.

Variant and control produce identical released geometry checkpoints:
`ARES22H` is 478 bytes with SHA-256
`4bc72e587c1a7061624d6a20df20d1cb4482dcad84951152ad4640d622b11f7a`;
`ARES22I` is 478 bytes with SHA-256
`4b37ef7c7816a29076288647810bcfb6fe0b341785b5a4505f602ab72f69cb87`.
The complete H and I streams differ only in their eight-byte magic. Each has
one object with `source_object_index=0`, `transform_index=0`, two planned
layers, and two volumes. Volume 0 has `source_volume_index=0`, ordinal 1,
type 0 (ModelPart), and two layers. Volume 1 has `source_volume_index=1`,
ordinal 2, type 2 (ParameterModifier), and two layers. Each volume layer has
index 0 then 1, mode 0 (Regular), one ExPolygon, and zero holes. Volume 0's
contour on both layers is
`[(10000000,1000000),(-10000000,1000000),(-10000000,-1000000),(10000000,-1000000)]`;
volume 1's is
`[(5000000,1000000),(-5000000,1000000),(-5000000,-1000000),(5000000,-1000000)]`.
Both streams end immediately after the second volume's second ExPolygon.

The second ignored fixed-source probe does not load a 3MF or resolve an
Option. Its exact input adapter consumes the common `ARES22I` geometry and a
command-line variant/control selector manually supplies the corresponding
region graph. It is therefore a composition oracle only; the real loaded
Option provenance belongs exclusively to the Ares archive/loader/resolver
tests above. The probe uses the same fixed Boolean/composition sources. Its
seven-source aggregate, with the framing order above, is
`1041608fc5c0bd0109242183e64a4618dd7ad2ab7c654ab9038fe91795c73563`.
The strict 446,464-byte executable has SHA-256
`f46ee6a82ce36d95aa7f5f56d1191e610c14420ccddf8d701b18ef119978fe3b`.
Five runs per case are byte-identical.

The complete variant `ARES22J` is 1,054 bytes with SHA-256
`1b18edae9cfbb9cd405cb7d45b1bec1a26168fe12c28a16366da211a30eadc77`;
its complete ordered rendering SHA-256 is
`02078fb14801f33ae561793931aa56a24a1dcdb6c135f8be07753a45f897876a`.
It has one object with `source_object_index=0`, `transform_index=0`, two
planned layers, two sidecar volumes, and two retained layers. Sidecar
occurrence 1 and occurrence 2 each have two layers, indexed 0 then 1; every
sidecar layer has one ExPolygon. Retained layers are indexed 0 then 1 and each
has two dense regions, IDs 0 then 1. For each layer index, the exact nested
vector is:

```text
sidecar occurrence 1: [(10000000,1000000),(-10000000,1000000),
                       (-10000000,-1000000),(10000000,-1000000)]
sidecar occurrence 2: [(5000000,1000000),(-5000000,1000000),
                       (-5000000,-1000000),(5000000,-1000000)]
region 0, surface 0, type 4: [(-5000000,1000000),(-10000000,1000000),
                              (-10000000,-1000000),(-5000000,-1000000)]
region 0, surface 1, type 4: [(10000000,1000000),(5000000,1000000),
                              (5000000,-1000000),(10000000,-1000000)]
region 1, surface 0, type 4: [(5000000,1000000),(-5000000,1000000),
                              (-5000000,-1000000),(5000000,-1000000)]
```

Every contour has zero holes. The trace is two complex layers, two
Differences, two Intersections, and zero closings. The loader/resolver graph
must expose two regions and child `RegionOptions.bridge_angle == 37` before
these bytes are encoded.

The complete control `ARES22J` is 698 bytes with SHA-256
`f2185c996e62a897b6af721f043a8ac150df647780693e828845f594524fd3d4`;
its complete ordered rendering SHA-256 is
`cf4e1c4668caa3c3676dd84d35bbc00f3f44644e4b85f56a921fb5582d9d3bad`.
Its object, source/transform indices, planned layer count, sidecar count,
occurrence IDs, sidecar layer indices/counts, ExPolygons, and zero-hole fields
are exactly those above. It has two retained layers, indexed 0 then 1. Each
retained layer has one region, ID 0, with one type-4 surface equal to occurrence
1's full contour and zero holes. Its trace is two complex layers, two
Differences, two Intersections, and two closings. The graph reuses the parent
region because no modifier Option changes. Both streams end immediately after
the final retained surface and contain every object, volume, layer, region,
surface, coordinate, count, and empty vector required by the protocol. They are
required literally, not as representative subsets.

Native Rust and Chromium each build variant/control deterministically, verify
the sorted uncompressed semantic identities, run each twice, parse the
complete vectors above, and require byte-identical native/browser I and J
outputs. Browser ZIP headers may differ from Rust; semantic entry framing and
the resulting checkpoints may not.

## KSR acceptance at this boundary

The committed project and reference G-code fixture hashes remain respectively
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`
and `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`.
The reference G-code is integrity evidence only; Task 22J code and tests do not
open it.

The released `ARES22I` input is 999,721 bytes with SHA-256
`0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`.
The exact Task 22J output is 2,008,706 bytes with SHA-256
`2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d`.
It contains:

- 1 print object and 460 planned/sidecar/retained layers;
- one sidecar volume with occurrence ID 1;
- one dense region slot per retained layer;
- 2,890 sidecar ExPolygons and 2,890 `stInternal` surfaces;
- 395 holes and 58,902 points in each geometry copy;
- no trimmed layer and exact EOF.

Five fixed-probe runs and an independent schema transform are byte-identical.
Representative complete sidecar layer records are:

- layer 0: 11,680 bytes, SHA-256
  `bbc99a45cc9a566fefdbc4a7fa1ae80865858126f2ba0a9b9ee9c412f8414581`;
- layer 46: 24,216 bytes, SHA-256
  `47486ac767ceea0b822566a750abc913c326141ca91eef5b27cfc1b37d26de4d`;
- layer 49: 23,512 bytes, SHA-256
  `ec3c90e0e8d276b9995169285b5b5a939e60bbd7283e46d0fa2c299bd8756816`;
- layer 459: 736 bytes, SHA-256
  `fd1b4912b9472d854d664769d1d0e5c5ec49e9bb9efd67e43c5707bca9189d0a`.

Representative complete retained-layer records are:

- layer 0: 11,702 bytes, SHA-256
  `633fcb207ed0be4092a75c7ad6052fa68579c4ced58371afa8837cd99d65c21e`;
- layer 46: 24,248 bytes, SHA-256
  `486a43246ef4bc94b2119a4b5787662ff65162c416137caf5d131c1ea5d458ec`;
- layer 49: 23,544 bytes, SHA-256
  `59eaf433513f5c92203cbd58b10612fb7b3438c627666d6e7a5dae24711c86ea`;
- layer 459: 761 bytes, SHA-256
  `a19b98ff4513317e141d1dac1c7f978f60b50602210b7d1bd4afd94c9b4fe82d`.

The committed KSR takes the single-model-part fast path on all 460 layers. It
cannot validate complex composition, identity/source-order disagreement,
modifier forwarding, negative ordering, closing thresholds, or no-top-trim
after subtraction. The complete synthetic inventory is therefore a release
gate, not optional supplemental coverage.

## Planned test inventory

Geometry tests cover exact Difference and Intersection Paths-to-PolyTree
closure, ordered roots/holes/islands, first-pass empty results, coordinate
range errors, default Clipper regressions, and no extra union pass.

Bounds tests cover object-volume transform order, zeroed XY translation,
retained Z translation, `f64`-to-`f32` narrowing, triangle-referenced vertices,
an ignored unreferenced extreme, positive/zero/negative contour compensation,
Z epsilon inflation, and inclusive touching bounds.

Region-graph tests cover source versus occurrence-ID order, loaded raw IDs
`[3,1,2]` mapping to occurrences `[1,2,3]`, support gaps `[1,3]`, first-created
config deduplication, model parts, negatives, changed modifier parents,
multi-parent records, ancestor bounds, no-op fallback to the last model part,
no-parent omission, accepted modifier Option resolution, and all ten unchanged
gates.

Composition tests cover empty/single/multiple fast paths, the disjoint-first
quirk, a bbox boundary where `slice_z as f32` is active while `print_z` is not,
touching-to-complex classification, later model-part priority, negative
before/after ordering,
modifier intersection/difference, repeated-source forwarding, empty parent,
occurrence sort keys, same-region append, Normal/LargeBed closing thresholds,
internal defaults, complete sidecar retention, empty middle/final layers, and
no top trim.

Project tests cover the complete committed KSR checkpoint, exact input/output
EOF and hashes, counts, representative records, repeatability, occurrence ID
1, public incomplete lifecycle, loaded-empty/absent range identity, unchanged
loaded-range rejection, unchanged shared-mesh/raw-ID and print-centering gates,
the complete modifier/control vectors above, and exact painted-triangle errors.

Browser tests build fresh default and Task 22J feature bindings, audit exact
exports, run an independently hand-written nested/empty-vector parser and
WebCrypto KAT, execute the committed KSR and modifier/control archives twice,
and freeze exact hashes, complete modifier/control vectors, counts, record
digests, surface tags, EOF, native equality, and repeatability in Chromium.

## Included behavior

- Accepted single implicit `[0, DBL_MAX)` volume bounds and region registry.
- Existing typed model-part and accepted modifier Region Options.
- Occurrence-ID `VolumeSlices` conversion and complete retained sidecar.
- Exact fixed fast and complex `slices_to_regions` behavior.
- Exact Difference, Intersection, and same-region closing closure.
- Dense internal surfaces with fixed defaults and no top trim.
- Complete native and WASM/browser conformance checkpoints.

## Explicitly deferred behavior

- Nonempty loaded `ProjectObject::layer_config_ranges()` and the complete
  range-filtered slicing chain.
- Usage collection for the ten currently rejected modifier fields.
- Orca material association outside the accepted fixed BBS 3MF vocabulary.
- MM facet painting, fuzzy-skin facet painting, and painted-region
  segmentation.
- Support enforcer/blocker composition in this object-region graph.
- Top-empty-layer removal, conical overhang, XY contour/hole geometry
  compensation after region creation, elephant foot, and `make_slices`.
- Surface top/bottom/bridge classification, perimeters, fill, supports,
  toolpaths, G-code assembly, metadata, and normalized reference-G-code
  equality.
- Cancellation, TBB, GUI, filesystem, and native viewer mechanics.
- Dormant Ares pipeline domains not reached by this source boundary.

## Structural, hardcoding, and platform constraints

- Every Rust production and test file remains below 400 physical LOC; split
  before reaching the limit.
- The 336-LOC `project_slice.rs` planning seam is genuinely extracted into a
  real module before Task 22J behavior is added.
- Tests live in separate real `mod` files. `include!` and `include_bytes!` may
  not split Rust source or test modules.
- No unsafe, FFI, filesystem, process, thread, UI, terminal, OpenGL, native
  dependency, platform branch, or second geometry engine enters `ares-core`.
- No production fixture name/hash/count, reference-G-code read, Option literal
  override, coordinate table, stage bypass, or KSR-specific branch is allowed.
- Existing obsolete executable Orca source-pinning tests remain deleted; no
  source-path/line/hash test is added.
- No legacy browser feature alias, fallback region composer, or compatibility
  shell is retained.
- Tier-1 remains WASM browser, Windows, macOS, and Linux.

## Verification and review exit criteria

Implementation follows strict RED-GREEN-REFACTOR packages. Exact complete
checkpoint assertions and synthetic fixed-source vectors are registered before
their production behavior. Expected oracle constants cannot change without
new fixed-source evidence and independent approval.

Before release, all focused and predecessor Task 22 tests, workspace nextest,
workspace all-target/all-feature Clippy with warnings denied, rustfmt, native
checks, both wasm32 checks, isolated feature-export audits, and fresh
Playwright Chromium tests must pass. Structural, provenance, hardcoding, and
fixture-integrity audits must pass on the exact candidate manifest.

One independent read-only reviewer must assess requirement completeness,
logical correctness, boundary cases, code quality, test coverage, and actual
execution. It returns a repair list to the main thread and makes no edits. The
main thread fixes every finding, reruns affected and full verification, and
sends the same reviewer the new exact candidate. This loop continues until
literal approval with no unresolved P0-P3 finding.

After six-axis approval, independent specification, quality, default-model,
and documentation reviews must approve the same bytes. The exact commit is
pushed normally and its Tier-1 run must pass all five jobs. Only then may the
next source-cited slice begin. Task 22J approval does not claim complete G-code
parity or complete the persistent user goal.
