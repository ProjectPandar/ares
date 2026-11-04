# SPEC: Classic physical-layer wall ordering for compatible-region owners

Brim is explicitly deferred because no separate Native-backed public brim RED
has been established.

## Upstream rewrite boundary

Pinned Orca source `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`src/libslic3r/PerimeterGenerator.cpp:1463–1471`, within Classic processing:
OuterInner or actual `layer_id==0` plus OuterOnly/nonzero brim reverses the
collection; InnerOuterInner sandwich ordering only runs when actual
`layer_id>0`. The upstream owner is PerimeterGenerator's layer identity and
post-traversal collection ordering, NOT counterbore Filled restoration.

Exact Rust destination: `crates/ares-core/src/project_slice/perimeters/classic/
perimeter_append.rs::transform_object`, its operand handoff to
`transform_record` → `transform_surface` → `reorder_walls`. Preserve upstream
reorder algorithm; replace the flattened-record-as-layer compatibility scaffold
with the existing actual layer identity (`ClassicTraversalRecord::layer_id`,
retained from perimeter input). Do not redesign an Ares ordering pipeline.
See [ARD](ard-classic-physical-layer-wall-order.md).

## Established RED and localization

Only acceptance REDs: `m-later-owner-classic-sacrificiallayer` (Filled) and
`m-later-owner-classic-none`. Frozen public `slice_with_profiles` comparisons
have Native exit0 and true full-stream divergence in both official
and seeded modes. Both first differ at byte2283/line106: Native travel
`X138.164 Y143.539 Z.72`, Ares `X142.418 Y141.501 Z.72`.
Native first extrusion loop is Inner hole radius9.681317; Ares first is
Outer contour radius10.750010. Startup/skirt are unchanged.

Ares-only public replay and two temporary in-memory probe
producers all preserve BOTH complete Ares streams byte-for-byte against frozen
Ares output; no new Native run is claimed. Seed tree is
contour0→contour1→hole1→hole0; first points respectively
(10750010,10750010), (10318683,10318683), (-9681317,-9681317),
(-9249990,-9249990), scaled LOCAL coordinates. Every nonempty frame chains
[(0,false)]; thin-hole reversal=false. Traversal emits hole1 first, consistent
with Native's public first-loop geometry. Actual APPEND records show flattened
arguments0/1/2/3 versus physical branch layers0/1/0/1, all InnerOuterInner.
Record2 incorrectly activates later-layer sandwich on physical layer0. There
is no inset2 in that layer's tree, so sandwich retains reversed collection,
predicting the observed first contour. Function-level discrepancy is
`transform_object` using enumeration `index` as layer identity.

## Record-index use audit and scope

Bounded grep/read audit:
`ffgrep index perimeter_append.rs`, `ffgrep enumerate classic/`,
`ffgrep 'layer_id.*index|layer_id.*record_index|index == 0|record_index == 0'
perimeters/`, and `ffgrep lower_slices perimeters/`, followed by full relevant
callee reads. This is not an assertion that the entire repository is defect-free.

| Consumer | Disposition |
| --- | --- |
| `transform_object:81` index==0 for first-layer OuterOnly brim | DEFERRED: no Native-backed public RED specific to this consumer; leave predicate unchanged in this slice. |
| `transform_object:99` index passed as layer_id; `reorder_walls:222–223` first-layer sandwich gate | INCLUDED: principal two-RED correction. |
| `transform_object:92` `traversal.lower_slices(index)` | DEFERRED, explicit adjacent identity misuse: `traversal.rs:166–191` expects physical/planned layer index, subtracts1 and steps region-major slots by physical layer count. Do not silently repair in this slice. Need its own overhang-reversal/lower-layer public RED and ownership amendment; actual layer ID and planned layer index must not be conflated, including raft offsets. |
| Record lookup `traversal.records[index]`, `region_options(traversal,index)`, input records[record_index] | RETAIN record index: aligned region-major record slots, not layer IDs. |
| Surface enumeration indexing `reorient.record.surfaces[index].roots` | RETAIN surface index, not layer ID. |
| Outer object enumeration indexing predecessor.objects[index] | RETAIN object index, not layer ID. |
| `classify_inactive` and `ReorientPlan` layer identity | Already sourced from traversal branch; no change. |
| `traversal::prepare_object` enum record_index; infill-boundary and chained/entity collection alignment | Retain slot lookup; traversal explicitly copies input.layer_id/planned_layer_index. No additional proven physical-ID substitution identified in this bounded audit. |

Deferred: N-split Arachne Filled, grouping/owner
selection, imports/centering, classifier/fill, Arachne ordering, loop nesting,
chaining algorithms, sandwich permutation, lower-slice handoff and raft policy.
No Native patch/debugging, fallback, oracle changes or comparator weakening.

## Test-first implementation and exit gate

1. Freeze exact two valid M-v3 inputs/options/transforms and complete ordered
   public REDs in a separate test commit. Do not substitute geometry or
   archive parts.
2. Run fresh official then seeded directed RED with fresh caches before
   production edits. Existing assertions stay intact.
3. Correct ONLY actual layer-ID operand passed to sandwich ordering.
   Leave index==0 brim predicate and lower_slices(index) unchanged and explicitly
   deferred. Retain record-slot lookups and upstream reorder math/geometry.
4. Require both full ordered public streams PASS under established generator-only
   normalization, including travel, arcs, M73, statistics/footer and role order.
   Verify later-owner physical layer0 and actual later layer; first-owner and
   incompatible controls; existing26 counterbore controls. First-layer brim
   is NOT an implemented consumer in this slice: any future inclusion requires
   a separate Native-backed public RED demonstrating its own divergence and
   a scope amendment before code.
5. Directed core/CLI tests via cargo nextest; cargo fmt --check, cargo clippy,
   WASM check and platform-neutral byte API invariance.
