# PrintObject centering and bbs_3mf shared-mesh rewrite SPEC

C1 then C2, each with independent RED/GREEN. Public load/selection changes
need a new SPEC.

## Owning upstream boundaries

This is a Rust rewrite of Model volume preparation, the bbs_3mf importer,
and PrintObject's coordinate frame. It does not define a new Ares pipeline.
Use the OrcaSlicer 2.4.2 source (`8500fcdcc`). Paths and line numbers below
are relative to its `src/libslic3r` directory.

| Source / function | Behavior to port or preserve |
| --- | --- |
| `Model.cpp:1241-1265`, `ModelObject::add_volume` | A fresh mesh is centered once after creation. |
| `Model.cpp:1295-1307`, `add_volume_with_shared_mesh` | Reuse the already centered mesh; do not center it a second time. |
| `Model.cpp:2600-2617`, `ModelVolume::center_geometry_after_creation` | Bounding-box center, float mesh subtraction, stored `init_shift`, volume translation, source mesh offset. |
| `TriangleMesh.cpp:271-284`, `TriangleMesh::translate` | Vertex and bounds translations use float components. |
| `Model.cpp:1586-1599`, `ModelObject::raw_bounding_box` | Model parts only, first source instance's transform without translation. The first instance may be nonprintable. |
| `PrintApply.cpp:138-170`, `print_objects_from_model_object` | Printable instances grouped by the non-XY-translated transform; sorted groups and original within-group instance order. |
| `PrintObject.cpp:93-115`, `PrintObject::PrintObject` | Original raw bbox center; rotate that center by the first-source/first-group-member rotation difference around Z; scale it to the integer center offset. |
| `PrintObject.cpp:126-130`, `set_instances` | Add the same center offset to each member's separately scaled XY translation. |
| `Print.hpp:326-328`, `trafo_centered` | Pretranslate the group's slicing transform by minus its integer center offset. |
| `Geometry.cpp:384-402/420-426/471-474/754-777` | Upstream rotation extraction and signed axis-angle `rotation_diff_z`, rather than recomputing a snug bbox for each group. |
| `Point.hpp:204-206`, `Point::new_scale` | Scale and cast each coordinate before the integer shift addition. |
| `Format/bbs_3mf.cpp:1949-1957/2058-2082/2116-2168` | Read model-settings metadata, assemble in ordered object identity order, honor explicitly selected plate's loaded-object subset. |
| `Format/bbs_3mf.cpp:4868-4889`, `_generate_current_object_list` | Breadth-first component expansion with accumulated component transforms. |
| `Format/bbs_3mf.cpp:4894-4974`, `_generate_volumes_new` | Select volume metadata, default shared key to the subobject's integer id, override via `mesh_shared` and `atoi`, then look up in importer-local `map<int, ModelVolume*>`. |
| `Format/bbs_3mf.cpp:4977-5035`, `_generate_volumes_new` | First available mesh is prepared and registered; subsequent occurrences reuse it. Key `-1` looks up/registers the current subobject id instead. |
| `Format/bbs_3mf.cpp:5038-5052`, `_generate_volumes_new` | Source matrix is provenance; actual volume transform is component transform composed with the prepared volume transform. Reused mesh restores its stored initial shift. |
| `Model.cpp:1708-1720`, `center_around_origin` | Separate whole-object placement operation; not the PrintObject integer center or mesh-sharing algorithm. Preserve existing placement behavior. |

`OrcaSlicer.cpp:1464/1640` passes the requested slice plate into
`Model::read_from_file`. The selected-plate import distinction is real, not an
optimization assumption: the first available shared mesh can be on that plate
rather than another plate (`bbs_3mf.cpp:5025-5028`).

## Ares boundary and current scaffold

`project/load/mesh_prepare.rs::prepare` already implements fresh f32 centering,
triangle orientation and a volume transform carrying the widened center shift.
Keep that fresh path's numeric evaluation unchanged. The missing boundary is
shared preparation: `volume_metadata.rs:149` reduces `mesh_shared` to a bool;
`assemble.rs::collect_volumes` freshly prepares every occurrence. Consequently
this is not only an admission-guard deletion.

`project/effective_config/grouping.rs` already retains sorted transform groups
and instance membership. Preserve it and its source semantics. The temporary
compatibility shell in `project_slice/raw_intersections.rs:294-330` enforces one
matching group and rejects shared meshes or duplicate `(path,id)` occurrences.
Replace those two gates with the source-owned behavior below. Do not broaden
or remove the other raw-intersection gates.

Current `raw_center` computes only the original source-instance center.
Its consumers assume one PrintObject: raw slicing, support facet/annotation
projection, support bed shift, seam visibility, emitted instance shifts, skirt,
brim/footprint bounds and auto-lift caches. Each actual PrintObject consumer must
use its group's frame and members. A second group's paths cannot be paired with
another group's center or copies. Inspect all callers before GREEN; retain
source-object-wide data only where its upstream owner is ModelObject-wide.

The eager Ares loader and later `Project::select_plate` are compatibility shells
around the bbs importer. Sharing must be realizable for the effective loaded
subset, including repeated selection of different plates from one public
Project. Never keep a process-global mesh cache or use a previously selected
view's reused mesh as the next view's original geometry.

## Native preflight and reproducible evidence

All inputs are temporary authored 3MF packages based on
`tests/parity/orca_cli_ender3.3mf`, with no embedded Native G-code. The mesh is an
asymmetric 10x6x1 box centered locally at (17,-3,0); a distinct second mesh is
16x6x1. Use classic walls, no support/brim/skirt/tower, aligned seam, by-layer
printing. Component package relationships and plate instance records are
explicit.

All three rounds used seed0 for replay and a fresh per-case Native
cache/artifact directory. These are reachability and geometry witnesses, not strict parity PASS claims
or the later committed public E2E RED.

### All rounds retained; input construction changes explained

1. The second `<object>` in model_settings was appended after `<plate>`.
   Native official 9/9 and seeded 9/9 exited0 and emitted G-code.
   Ares accepted single/XY controls; three cases reached
   `print_object_centering`; four two-object cases stopped earlier with
   `duplicate field object`. Thus seven Ares rejections included four XML
   construction rejections, not seven centering guard witnesses.
2. The only authoring correction was placing the second `<object>` immediately
   after the first, before `<plate>`. Geometry, transforms, sharing keys and
   settings were unchanged. Native official 9/9 and seeded 9/9 again exited0.
   Ares admitted distinct and implicit-shared two-object packages. Three cases
   reached `print_object_centering`; explicit `mesh_shared=1/-1` still stopped
   at `duplicate field metadata`, because their metadata was appended after
   the part's `<mesh_stat>`. Five rejections here were three guards plus two
   XML construction rejections. Implicit sharing already showed wrong geometry.
3. (Authoritative guard matrix.) The only next correction placed `mesh_shared` alongside the part's other
   metadata, before `<mesh_stat>`. All logical case parameters remained the
   same. Native official 9/9 and seeded 9/9 exited0. Ares accepted four packages
   and rejected five via the two intended guards. No XML construction failure
   remains. This round isolates the feature boundary; it does not overwrite
   either earlier round or use repeated execution to explain away differences.

ZIP/input SHA differences between rounds are input-construction changes, not
Ares/Native output instability.

| Case | Native both modes | Current Ares | First-layer positive wall witness |
| --- | --- | --- | --- |
| single | exit0 | exit0 | X[122.31,131.69], Y[104.31,109.69] |
| XY copy, same group | exit0 | exit0 | X max176.69, 36 wall endpoints instead of18 |
| identity + 90-degree copy | exit0 | `print_object_centering` | X max160.69, Y max131.69 |
| identity + X-scale2 copy | exit0 | `print_object_centering` | X max198.69 |
| nonprintable scale2 first, printable identity next | exit0 | `print_object_centering` | Only one box, same world wall bounds as single |
| distinct second mesh id4 | exit0 | exit0 | X max179.69 (16mm source width) |
| explicit shared key1, second id4 | exit0 | `shared_mesh_centering` | X max176.69 (first mesh reused) |
| implicit shared id1 in another model file | exit0 | exit0, wrong geometry | Native X max176.69; Ares179.69 |
| explicit key-1, unseen second id4 | exit0 | `shared_mesh_centering` | Native fresh mesh X max179.69 |

For explicit vs implicit sharing, each mode's raw Native G-code SHA is identical
between the two cases in the authoritative round. The distinct control has
larger wall extent. The implicit case proves the upstream cache is keyed by
integer id, not Ares `(path,id)` identity. Current Ares has the exact same raw
G-code SHA for implicit-shared and distinct-mesh, incorrectly using the second
source's independent geometry. The 3mm extent difference is deterministic and
present in both oracles; it is not a stochastic Variant A attribution.

## Proposed implementation slices (separate RED, GREEN)

### C1: PrintObject group centering

Rust destination: a small internal centering module beside raw intersections,
with the original raw ModelObject center computation preserved and a group
center operation receiving the actual first member. Keep centers in floating
point through rotation, then apply the source integer scaling once. Pair the
same integer center with both mesh pretranslation and instance shift addition.
Do not recompute a snug per-group bbox or substitute the first printable source
instance for the upstream original first source instance.

Port the exact rotation extraction/difference needed by the accepted transform
model, including mirrored/scaled transforms, rather than assume atan2 of one
matrix column is generally equivalent to `computeRotationScaling`.
The upstream constructor rotates only by its signed Z-axis angle difference;
it does not introduce an independent scale correction to the original center.
Keep group transform, Z placement/shrink compensation and group sorting intact.

Replace `validate_centering` only once the existing physical group identity is
carried to each live frame consumer. For projected supports and emitted motion,
share that group's center calculation, not an independently rounded shift.
Update skirt/footprint membership where a prepared object is currently indexed
as if it were the source object. Update seam visibility's source-first transform
when called for another group. No new global transform registry or public API.

Included: multiple printable transform groups, same-group XY copies,
nonprintable original first instance, rotations/scales/mirrors already admitted
by the model loader, existing support/brim/skirt/visibility consumers' coordinate
frame and instance membership. Other feature guards remain independent.
Deferred: arranging/duplicating objects, object-origin recentering UI actions,
new support, brim or seam algorithms, SLA and contour-Z behavior. No new guard
or silent skip is introduced for a centering case.

### C2: bbs_3mf mesh reuse and one-time mesh centering

Rust destination: `project/load/{volume_metadata,assemble,mesh_prepare}.rs`,
internal ProjectVolume preparation provenance, and selected-view materialization
in `project/domain/plate_selection.rs` where necessary to mirror import scope.
Split `assemble.rs` before growing it beyond400LOC; it is already near that limit.

Replace presence-only `mesh_shared` metadata with the upstream signed key and
its default/subobject-id distinction. Match the source integer conversion.
Carry a load/selected-view-local ordered map of already prepared mesh handles
and initial shifts. First available occurrence fixes orientation and centers in
f32 once. Reused occurrences use that centered mesh and the first mesh's shift,
composed with their own accumulated component transform. The later occurrence's
raw mesh is not a second preparation source. Preserve per-volume name, type,
region overrides, source matrix and annotations rather than copy the first
volume's whole configuration.

Observe Native's sorted object identity import order and breadth-first component
order even if build-item output order differs. Preserve output instance order.
Key-1 performs lookup and insertion using the subobject id; an absent explicit
key prepares/registers the first available local mesh. Different paths with the
same integer key can share. Distinct keys must remain independent.

Keep original occurrence geometry/shift and component provenance sufficient to
materialize another selected plate from an immutable original Project. Sharing
must use the selected loaded subset, not a cache filled by excluded objects.
Use immutable prepared mesh handles where required; preserve the public borrowed
vertex/triangle API. Do not persist Native output or a cross-request cache.
A plate-local witness is mandatory before production: the all-plate Native cache
and selected-plate cache can deliberately choose different first meshes.
If realizing this import scope requires a broader public load/selection contract
change, report that dependency for a separate source-cited SPEC before coding it.

Remove `validate_shared_meshes` after reuse/centering semantics are implemented.
Replace the obsolete bool rather than retain it as a compatibility flag.
Included: explicit/default/-1 keys, duplicated component occurrences, cross-file
integer-id reuse, first available mesh and init-shift restoration, selected-plate
scope, volume metadata isolation. Deferred: mesh reload/export/backup, GUI undo,
SLA, native pointer identity itself; Rust immutable sharing is its representation.

## TDD and release gates

For each slice first commit authored inputs and public API/CLI Native RED
independently before production. Official and seeded
both use strict full ordered generated-output comparison. Reachability/extents
above explain the cases; they do not replace the strict comparator. All Native
G-code, diagnostics and execution evidence remain external artifacts, not Git.

C1 matrix: original controls, two rotation groups, scaling, a mirror, reversed
source/group order, nonprintable first, fractional coordinates for separately
quantized translation/center, Z/shrink, and representative existing support,
brim/skirt and seam settings that exercise group-frame consumers.
C2 matrix: distinct versus explicit and implicit sharing, repeated same-path
mesh, shared source with a different original shift/shape, transformed component,
key-1 lookup and fresh fallback, unseen explicit key, reversed build/object order,
selected plate with shared source excluded, repeated independent plate selection,
and per-volume metadata/annotation isolation. Commit immutable provenance/SHA.

Existing guard-only unit assertions need a separate retirement record listing
precisely the contradictory assertions/tests, with RED and source citations.
This SPEC does not delete them. Preserve all unrelated assertions.
No UnsupportedProjectFeature replacement, normalization, ignored case, probe,
Native-product commit, lint suppression or `regions[0]` shortcut is permitted.
Files stay <=400LOC. Fresh Native cache/artifact roots; strict workspace
clippy, fmt and WASM must be green.
