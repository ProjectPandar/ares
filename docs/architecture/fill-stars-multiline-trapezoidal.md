# SPEC: FillStars sparse multiline trapezoidal branch

Status: Accepted spec; implementation resumed after the multiline union
prerequisite merged.

## Priority and ownership

Select the `tri-hexagon` / `FillStars` active `fill_multiline > 1` dependency
under the [roadmap's current full-acceptance priority](../roadmap.md#current-acceptance-priorities-full-acceptance-open).
The current Option plan already declares `fill_multiline` min1/max10/interior5
and the `sparse_infill_pattern` enum covered. That does not establish this
coupled consumer: the current Stars adapter explicitly defers the trapezoidal
branch and unconditionally uses the ordinary scanline-family generator.
This is not another inventory-count or domain-status promotion.

Exclude
brim, scarf/APA, G4/tool ordering, profile inheritance, CI and web. Do not change
Grid or Triangles dispatch, their generators, or their prior parity contracts.
Any conflicting assignment discovered before implementation stops this slice.

## Upstream ownership

Source tree: `OrcaSlicer`, revision `8500fcdcc`. Paths in this table are relative
to `src/libslic3r`; line references were checked against the local source.

| Source boundary | Contract to rewrite / preserve |
| --- | --- |
| `PrintConfig.cpp:238`, `:2996-3003` | `tri-hexagon` maps to `ipStars`; `fill_multiline` is coInt, default1, min1/max10. |
| `Fill/Fill.cpp:925-926`, `:1281-1288` | Only sparse `erInternalInfill` receives the configured bundle count; other roles receive1. Density converts from percent to float fraction; anchors pass to FillParams. |
| `Fill/FillRectilinear.hpp:105-115` | `FillStars::fill_surface`; its `_layer_angle` returns0. Do not add ordinary alternating scanline rotation. |
| `Fill/FillRectilinear.cpp:3487-3505` | Count>1 dispatches to `fill_surface_trapezoidal(..., Pattern_type=2)`; count1 retains `fill_surface_by_multilines`. |
| `Fill/FillRectilinear.cpp:3048-3085` | Shared trapezoidal setup: truncate scaled spacing before integer multiplication for d1, triangular period, float-origin infill direction and double base-angle addition, rotate the domain, extended object bbox, effective infill-layer id. |
| `Fill/FillRectilinear.cpp:3260-3336` | Mode2 star-row coordinates, integer halves/chamfers, aligned symmetric limits, normal/mirrored rows and alternating pair shifts. Apply `(layer_id / thickness_layers) % 3` orientation when thickness_layers>0; otherwise use layer_id. |
| `Fill/FillRectilinear.cpp:3344-3384` | Bundle expansion, half-spacing surface contraction, source fallback to the original domain if contraction is empty, ordered clipping, `< scale_(0.8*spacing)` short-line removal, connect-or-chain branch, rotation back. |
| `Fill/FillBase.cpp:275-329`, `:1569-1578`; `Print.hpp:347`; `Fill/Fill.cpp:1224` | Object-centered reference, source angle arithmetic and transpose-merged/sqrt2-scaled PrintObject bbox. Surface-local bounds must not replace the object-wide generation domain. |
| `Fill/FillBase.cpp:2712-2784`, `Fill/FillBase.hpp:52` | Existing `multiline_fill` bundle expansion; `dont_connect` is anchor_length_max<0.05f. |
| `Fill/FillBase.cpp:133-175` | Existing extrusion collection, flow/width/height and role materialization remain the consumer boundary; no new Ares extrusion pipeline. |

This slice implements mode2 and its necessary setup/finalization, not the
mode0 Grid or mode1 Triangles branches. Preserve the source integer casts,
integer division, float density/angle starting points, double calculations,
rotation center and clipping/connection order. The call to `multiline_fill`
passes spacing as float; preserve that narrowing at this source boundary.
Do not substitute rounded
coordinates, a per-surface grid or a synthesized approximate star pattern.

## Rust destination and existing scaffold

- `ares-core/src/project_slice/fill_entities/stars.rs::append`: conditional
  source dispatch. Preserve the count1 route and collection flow/role behavior.
- `ares-core/src/fill/stars_trapezoidal.rs` with private children only if needed
  to keep each file below400LOC: `FillRectilinear::fill_surface_trapezoidal`
  mode2 rows and finalization. Reuse the existing `fill::multiline_offset`,
  checked rotation, ordered open-path clipping and `fill::connect` primitives;
  do not create another geometry/cache framework or public API.
- Narrow object-once wiring in
  `project_slice/fill_entities.rs::prepare` and `fill_entities/stars_bounds.rs`:
  `Fill.cpp:1624/1637` set_bounding_box(object()->bounding_box()),
  `PrintObject.cpp:96-112`, `Model.cpp:1586-1599`,
  `TriangleMesh.cpp:440-446` and `Print.hpp:347` provide source model-part mesh
  dimensions. Compute once only for Stars count>1 and pass Option to the layers;
  count1 and all other patterns do not compute this box or change their X adapter.
- Minimal Stars-specific wiring in `project_slice/fill_entities/generate.rs`
  carries layer id, surface thickness_layers and the required object-wide
  extended bbox. Existing `ObjectFillGeometry`/X-bound adapters are temporary
  compatibility shells, not proof that all bbox provenance is ported. Derive
  both dimensions from the source-centered box; do not globally change the
  legacy bbox callers or reinterpret compensated slice bounds as mesh size.
  If correct provenance requires a wider traversal change, report that boundary
  before editing it. The finite inputs below disable XY compensation.
- Additional Stars consumer:
  `project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms_tail.rs:43-60`.
  Upstream `Fill.cpp:1394-1501` creates the same FillStars generator, sets the
  PrintObject bbox (:1398/:1443), forwards multiline (:1489), and invokes
  fill_surface (:1497). For Stars count>1 this caller must share the new mode2
  generator, layer phase and object box; count1 and other patterns retain their
  current dispatch. This wiring is included.
  `transaction/candidate_expansion/layers.rs::prepare_infill_lines` computes
  the source mesh box once per object only for Stars count>1, then threads it
  through private `sparse_anchoring::{mod,arms,arms_tail}` calls. Planned
  Layer.id and each fill's thickness_layers reach mode2. Reuse the same
  `fill_entities/stars_bounds.rs` helper; no traversal data model is expanded.
- `ares-cli/tests/orca_parity/option_coverage/tri_hexagon_multiline.rs` and
  dedicated helpers: runtime Native admission, complete-output RED and controls.
  A registration line is the only intended shared test-module edit. No domain
  plan, inventory count, generic activation or comparator change is proposed.

The current `fill::multiline::fill_surface` remains the source rewrite of
`fill_surface_by_multilines` for its existing callers. For Stars count>1 it is
an incorrect provisional dispatch, not a legacy fallback to retain after a
real RED justifies the new branch. Adjacent primitive mismatches discovered
by the RED require a separate bounded report rather than a broad refactor.

## Finite Native admission / RED design

Use one nozzle / one filament, by-layer classic walls, no support/raft/brim,
prime tower, arc fitting, scarf, fuzzy skin, SAFC or adaptive PA. Fixed .2mm
layer height, positive30% sparse density, rectilinear solid/bottom/top fill,
and stock templates/settings. Admit two provenance-recorded slicing inputs:
a centered64x48x4mm prism and the same prism with a centered8x8mm through-hole.
Inputs are authored geometry/project data; store no Native-generated output.
Native export may resolve settings outside git; Ares must consume those exact
exported project bytes. Keep input/archive-member hashes in runtime receipts.

| Case | Geometry | fill_multiline | Density | Infill direction | Connection |
| --- | --- | --- | --- | --- | --- |
| S1 | prism |1 |30% |45deg |stock anchors |
| S2 | prism |2 |30% |45deg |stock anchors |
| S3 | prism |3 |30% |45deg |stock anchors |
| S10 | prism |10 |30% |45deg |stock anchors |
| S2A | prism |2 |15% |0deg |stock anchors |
| H1 | holed prism |1 |30% |45deg |stock anchors |
| H2 | holed prism |2 |30% |45deg |stock anchors |

Freeze Native/Ares requests and all first outcomes. Require
successful Native slicing and unchanged effective requested values. Prove
actual positive-XYE sparse extrusion, at least three reached source infill-layer
phases and both odd/even bundle witnesses. Hole case must reach clipping on
both exterior and hole boundaries. S2N is excluded from this slice (see below). A Native rejection, timeout, rewritten request,
absent consumer, unstable Native output or unrelated first difference is not
this port's RED; report it without changing context to manufacture a PASS.

Use existing OrcaRunner `slice_case`, producer identity/lock/receipt and cache.
Compare the complete ordered bytes and line count with the existing generator-
only normalization. Preserve M73, travel, flow, footer, repeats and ordering.
Retain first differing line/byte, layer, role and complete outputs outside git;
confirm the first body difference is sparse-infill geometry/order, not admission
or metadata. S1/H1 are count1 controls. Do not infer active multiline coverage
from a settings echo, scalar range plan or filtered polygon comparison.

If the finite count>1 cases already match current main completely, report
NO RED FOUND with evidence and do not implement from source suspicion alone.
Otherwise commit the real Native-backed RED before any core GREEN.

## Included, deferred and acceptance

Included: sparse Stars mode2 row generation and source finalization for the
finite1/2/3/10, angle/density, hole and connected witnesses; unchanged
count1 behavior and role-specific count routing. Source connection primitives
remain the single implementation. Core stays byte/geometry-only and portable
to WASM, Windows, macOS and Linux; filesystem/Native work stays in CLI E2E.

Deferred: Grid/Triangles trapezoidal modes, other patterns, complete multiline
pattern exhaustion, infill combination>1, bridge/solid Stars selection,
object/region overrides, modifier/range settings, nonzero XY compensation,
non-axis-aligned model transforms, multinozzle/SEMM, towers and GUI consumers.
Source integer/floating arithmetic is included; no UB or nondeterminism
emulation, fallback, ignore or assertion tolerance is permitted.

Exit: Native-backed RED -> bounded core GREEN -> fresh
cold/hot directed witnesses in seeded/official and entire option_coverage group
(including non-Native tests), strict clippy/fmt/wasm32 and fresh default workspace.
Any original expectations updated must be itemized; unrelated assertions/plans
remain unchanged. A production GREEN then requires a fresh full run on latest
main whose candidate failures are a subset of the base failures (currently both
empty). No inherited print-statistics failure exception. Audit changed raw
streams and retain all failures; no automatic reruns.

## Native-backed RED observation

Both seeded and official fresh runs on the unchanged `c2de7a2e4` production
code give the same classification: S1/H1 whole-output PASS; S2/S3/S10/S2A/H2
whole-output DIVERGENT; S2N Native success but Ares admission error. All Native
requests retain the requested effective values, positive-E sparse extrusion
and all three layer phases. Hole controls reach both clipping boundary classes.
No timeout, rejection, ignore, filtered comparison or production edit occurred.

The earliest normalized difference in the five byte REDs is line18 M73 remaining
time (Native/Ares: S2 and H2 47/46, S3 44/41, S10 38/37, S2A 36/35). First
sparse XY+E commands prove the actual geometry difference on layer4 (Z0.8):

| Case | Native first sparse line | Ares first sparse line |
| --- | --- | --- |
| S2 |1419: X135.227 Y133.02 E.01663 |1418: X134.974 Y133.02 E.01315 |
| S3 |1418: X141.02 Y127.924 E.01708 |1420: X133.595 Y133.02 E.05401 |
| S10 |1416: X141.02 Y133.02 E.0035 |1424: X135.61 Y133.02 E.05401 |
| S2A |1425: X138.724 Y133.02 E.02779 |1425: X137.851 Y126.08 E.28846 |
| H2 |1705: X141.02 Y128.255 E.00723 |1705: X134.974 Y133.02 E.01315 |

The whole-output comparator still requires PASS; the XY+E table is diagnostic,
not another acceptance comparator. Initial S2N observation attempted
reading absent Ares output after the real error; the corrected RED records
ARES_ERROR explicitly and retains the PASS assertion. All eight cases were
then run with independent fresh caches, with identical classifications. No
Native artifact is tracked.

S2N is not silently removed: its effective absolute `infill_anchor_max=0`
projects to 0.0, rejected by
`prepare_infill/bridge_over_infill/transaction/anchor_projection.rs:102-104`
(`!anchor_max.is_finite() || anchor_max < 0.05`). Upstream `Fill.cpp:978-989`
projects/clamps sparse anchors and `:1286-1287` forwards them. Zero selects
`FillBase.hpp:52` dont_connect, so `FillRectilinear.cpp:3366-3373` chains/appends
instead of calling connect_infill; `FillBase.cpp:1820-1828` has the same split.
`FillBase.cpp:1580-1587` connection assertions apply to the connecting branch.
The cited `:2712-2784` only expands bundles, not anchors. Generator finalization
was included in this spec, but the bridge transaction admission edit is outside
its listed Rust targets. Decision: the zero-anchor admission is the general rectilinear-family
no-connect slice, not Stars mode2. S2N is removed from this branch's active
controls, preserving the five real byte REDs and two count1 controls.

### Deferred zero-anchor admission

S2N (`infill_anchor_max=0`, count2) is deferred to the next separately specified
rectilinear-family no-connect port. The existing guard is
`anchor_projection.rs:102-104`, rejecting finite projected values below0.05.
Upstream references: `Fill.cpp:978-989/:1286-1287`, `FillBase.hpp:52`,
`FillRectilinear.cpp:3366-3373`, `FillBase.cpp:1820-1828/:1580-1587`.
This is not `multiline_fill` (`FillBase.cpp:2712-2784`). Native accepts in both
modes; Ares reports UnsupportedProjectFeature. No guard
change, ignore, skip or retained failing S2N test belongs to this Stars branch.
The subsequent slice must reuse this real RED and port no-connect admission
before removing the guard; no scope expansion is implied here.

### Deferred shared union prerequisite

The first Stars mode2 implementation produces exact Native row arrays and exact round
offset polygons before union on all13 emitted S2 layers. Differences first
appear at union: Native uses Clipper2 Positive union
(`deps_src/clipper2/Clipper2Lib/src/clipper.offset.cpp:604-630`), while the shared
Ares `fill/multiline_offset/round.rs:57` uses the Clipper1 NonZero union helper.
This is a common `FillBase.cpp:2712-2784` dependency, not a Stars-local algorithm.
Decision: specify and prove a non-Stars Native-backed RED before any shared
union change. If those controls already match, report rather than widening
the Stars slice or silently changing Clipper1 semantics.

### Resumed trial after the union prerequisite

After the numeric union slice merged, seven runtime controls ran with fresh
cache/root per mode; both exited100, three PASS/four FAIL.
S1/H1/S2A passed; S2/S3/S10/H2 remain whole-byte DIVERGENT. First whole lines:
S2 line135 M73 placement, S3 line38 remaining time, S10 line131 M73 placement,
H2 line18 remaining time. First positive-extrusion differences: S2/H2 sparse layer5(Z1.0), S10
sparse layer4(Z0.8). S3's14457 sparse extrusion commands agree, but its first
machine-command difference is layer17 Outer wall retract/travel (Native
line16684 `G1 E-2.8 F3600`, Ares line16686 `G1 X140.608 Y133.201 F9000`).
Solid extrusion commands agree in all seven controls. This does not establish
anchoring as the cause; the anchoring mode2 caller is connected next
and must be tested separately without changing comparison or assertions.

### Anchoring follow-up and remaining union input mismatch

With anchoring connected, the same seven controls ran with fresh caches/roots.
Both modes exited100: S3 became whole-output PASS; S1/H1/S2A stayed PASS with
unchanged Ares raw. S2/H2/S10 remain DIVERGENT, with their first sparse
differences still at layers5/5/4 respectively.

Read-only gdb evidence compares14 S2 calls (one anchoring,13 emitted fills) against earlier initialized
Native captures with identical input members. All ordered rows and raw offset
polygons match; union phase0 matches, phase1/2 differ before clipping/connection
(`FillRectilinear.cpp:3345/3351/3357/3366-3373`). Phase1 Native13489 versus
Ares13459 points; phase2 Native13492 versus Ares13461. The union belongs to
`FillBase.cpp:2712-2784` through `clipper.offset.cpp:604-630`.

All30 phase1 Native-only points exist in the equal raw input. After adjacent
duplicate removal each is exactly collinear with its raw neighbors. Clipper2 `clipper.engine.cpp:615-661`
AddPaths_ skips duplicates but retains these vertices. Ares
`geometry/clipper/input/path.rs:114-127` clean_path removes them before union
when preserve_collinear is false. Native only cleans collinearity after union
(`clipper.engine.cpp:1531-1565`); intersections can change their neighbors by
integer truncation, making the retained vertex noncollinear. Example raw
point(36252936,31786081): raw cross0, Native final neighbors give cross-740788.
This is distinct from the already-ported intersection arithmetic.

Narrow follow-up: only the Clipper2Truncate/Positive
helper retains all input collinear vertices, including spikes; duplicate
removal remains. A compile-time RoundPolicy constant selects input cleanup,
with the existing default Clipper1 behavior intact. Output cleanup still uses
preserve_collinear=false. No runtime hooks, new sweep or external crate.
Revalidate ten union controls, seven Stars controls and whole option_coverage
in both modes. Fresh direct Native/Ares H2 capture also reproduces the same
14-call phase mismatch.

### Input-only-dedup trial: geometry restored, order remains different

Both modes exited100: ten union controls PASS, Stars S1/H1 PASS,
S2/S3/S10/S2A/H2 DIVERGENT (12/17). All five remaining first sparse differences
are now layer4(Z0.8). S3/S2A changed from PASS to FAIL; no result or assertion
is suppressed.

S2 read-only capture confirms all14
row arrays equal Native, phase0 union13104 points, phase1 N13489/A13489 and
phase2 N13492/A13492. Each phase has24 closed paths and identical point sets.
Diagnostic cyclic/path-order comparison finds all polygons geometrically
identical but different start vertices; phase1/2 additionally permute paths.
This diagnostic does not normalize or relax the whole-output comparator.

Remaining source boundaries: Clipper2 CleanCollinear (`clipper.engine.cpp:
1531-1565`) preserves outrec->pts unless that node is removed; Ares
`geometry/clipper/output/fixup.rs:8-43` assigns the final cleanup cursor as
the output front. Native BuildPath64 (`:2885-2921`) versus Ares collect_out_paths
(`fixup.rs:60-83`) makes the cursor observable. Native minima Reset (`:53-60/
:788-796`) stably sorts descendingY/ascendingX; Ares `minima.rs:7-12` uses the
Clipper1 unstableY-only sort and consumes from the vector back.
Proposed next scope, not implemented: first port only the cleanup/front cursor
contract for this private Positive policy, then diagnose the minima ordering
if permutations remain. Keep default Clipper1 output/minima behavior and strict
assertions intact; no whole sweep port or coordinate-based output sorting.

### Positive output order follow-up, step 1

Private CleanCollinear/front-cursor alignment comes first, then stable minima
source order only if permutations remain. Both are selected
by Clipper2 policy; default Clipper1 remains unchanged. Output-coordinate sorting
and comparator changes are excluded.

Step 1 keeps the outrec front unless
that node is removed. The arena's previous link corresponds to Clipper2 next,
so cleanup follows that link and updates a removed front to the other neighbor.
`output/clipper2_front.rs` cites CleanCollinear `:1531-1565`; existing output
collection follows BuildPath64 `:2885-2921` in the same mapped direction.

A fresh run gives exit100 in both modes, each 16/17:
all ten union controls PASS, Stars S1/H1/S2/S3/S2A/H2 PASS, S10 DIVERGENT.
S10 first difference is line 131, Native M73 P3 R36 before the extrusion versus
Ares after it. The unchanged strict comparator still rejects the whole file.

Existing-binary S10 captures contain all 14 calls.
Ordered input rows match. Phase 1 shows paired path permutations; phase 1/2
point counts are Native 5876/5878 versus Ares 5878/5879. Some paths therefore
still differ geometrically, rather than only in their front or permutation.
These observations do not justify a broader sweep or output normalization.
Step 2 implements only the source minima ordering.

Step 2 also gives 16/17 in both modes, with only S10 DIVERGENT. The step 1/2
raw audit confirms 34/34 Ares G-code outputs unchanged. The S10 stage audit
shows changed union ordering in phase 1 calls 2/5/8/11; all other calls are
unchanged. Every geometrically matching
polygon now has the same Native/Ares index, with no remaining matched-path
permutation. Post-clipping arrays are unchanged across all 14 calls. Thus
minima sorting has a source-stage effect, but has not repaired the S10 bytes.
This source-stage correction is retained; it is not claimed to repair the S10
bytes by itself.

### Shared multiline expression precision correction

FillBase.cpp:2739-2749 receives spacing as float: `i * spacing` remains f32
before conversion into the double offsets vector, while `0.5 * spacing` and
the even-offset sum use f64. These types are preserved
in the shared `fill/multiline_offset.rs::apply`; count 1 still returns early.

An existing Native capture contains all 14 S10 calls and five actual Execute deltas per call. Every value
matches the f32-product expression exactly. For captured spacing
0.40707963705062866, index 3 gives Native delta 1424778.6700725555 versus the
old f64-product delta 1424778.7296772003. The original missing-symbol capture
is excluded; successful spacing/delta capture uses x86_64 argument registers.
No Native rebuild or artifact is committed.

The correction keeps multiplication in f32 and then promotes to f64,
with an explicit upstream-line comment. It gives 17/17 PASS in each
mode, including S10 whole-output equality. Against Step 2, 16/17 raw outputs
are identical per mode, only S10 changes (line 131 M73 placement and
subsequent path differences repaired). Complete option_coverage passes in both
modes; against the pre-correction baseline only S10 changes. No non-RED raw
change is observed. The baseline official tree failure was a Native-only
line 1081 travel variation (113.388/113.135 versus 113.244/113.297) with the
same input and Ares raw SHA.
