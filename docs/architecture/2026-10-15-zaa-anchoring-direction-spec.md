# ZAA sparse-anchoring fill-direction SPEC

Status: SPEC-REVIEW-REQUEST; source/docs only. No Native/build/test run,
public RED or production edit in this submission. Run no discovery/RED
inside the coordinator's FULL14 exclusive window.

Base main caac760d464112b076821975b67232fdd0f14bf0 plus approved W2 chains
through58f963c54c2f2b5d2bdd46fc3eea45b8cde13833, queued for stack-14.
This separate branch preserves that approved candidate. Rebase onto its
merged main before the next submission when the coordinator broadcasts it.
Upstream inspected: OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Owning source boundary and current gap

Fill/Fill.cpp::Layer::generate_sparse_infill_polylines_for_anchoring:
1394-1503 generates analysis polylines rather than final extrusion entities.
It groups fills and keeps stInternal surfaces, constructs a filler for the
pattern, sets the object bounding box, and assigns layer ID relative to
the first object layer. At1440-1447 it derives the runtime flag from the
current SurfaceFill region:
`dont_alternate_fill_direction = rcfg.zaa_enabled &&
rcfg.zaa_dont_alternate_fill_direction`.

Fill/FillBase.cpp::Fill::_infill_direction:275-323, specifically311-313,
uses that flag to inhibit only the layer-angle alternation term. Bridge
angle precedence, fixed_angle, thickness-layer divisor, reference point
and final float pi/2 are independent source behavior and must remain intact.
FillBase.hpp:147 owns the default false; this slice ports the explicit
regional assignment at the anchoring call site rather than its default.

PrintObject.cpp::PrintObject::bridge_over_infill:2468-3553 consumes these
polylines. Lines2735-2754 generate infill_lines for layers immediately below
bridge candidates. The determine_bridging_angle lambda at2849 and its
calls at3244-3250 use the resulting anchors to select internal-bridge
orientation. That bridge output is the public activity witness required
for this slice; a different ordinary sparse-fill stream alone is inadequate.

Ares has the corresponding source boundary under
project_slice/prepare_infill/bridge_over_infill/sparse_anchoring:
- mod.rs::generate_sparse_infill_polylines_for_anchoring groups internal
  fills and dispatches with object_index/layer_index;
- arms.rs::dispatch's linear branch (Rectilinear, AlignedRectilinear,
  Monotonic, MonotonicLine, ZigZag, CrossZag, LockedZag) calls linear::append;
- linear.rs::append currently initializes
  MonotonicFillParams::dont_alternate_fill_direction to false.

The downstream candidate_expansion/layers.rs passes each candidate's lower
layer to this generator. The ordinary fill_entities path already carries
the regional conjunction in approved58f963c54; that repair intentionally
left this separate anchoring constructor at false. No new runtime witness
of the anchoring discrepancy is claimed by the previous ordinary-fill RED.

## Exact Rust destination and implementation boundary

Resolve the actual SurfaceFill::region_id for its printing object at the
existing sparse-anchoring dispatch boundary, using its existing object
identity/context and canonical region graph. Derive the same two-option
conjunction as Fill.cpp:1444-1446. Carry the scalar to linear.rs::append's
existing MonotonicFillParams, replacing the hard-coded false.

The expected production boundary is sparse_anchoring/arms.rs and linear.rs,
with mod.rs runtime-context plumbing only if necessary. Reuse the existing
SurfaceCall context if it is the smallest way to carry scale, layer ID and
the regional bit; do not introduce a sixth argument, new lint suppression,
parallel parameter framework or new per-point allocation/copy. The existing
rectilinear and multiline kernels already honor the bit after the ordinary
fill slice; no second direction algorithm belongs here.

Use the specific fill region, including already-resolved modifier-derived
regions. Do not select a first region, make the setting object-global, alter
region/group identity or add a capability guard/fallback. Keep layer ID,
angle/template and fixed_angle, sparse density/spacing, anchor lengths,
object bounds, overlap, dont_adjust, link length and loop handling unchanged.
No edits to bridge candidate admission/expansion, bridge-angle optimization,
ordinary fill generation, ray queries, motion or ZAA slice-plane sampling.

The existing sparse_anchoring adapters are temporary compatibility shells
around the named Layer generator, not an Ares-owned replacement pipeline.
This is its missing regional Fill runtime control, not a new anchoring API.

## Included and deferred

Included: the existing linear dispatch arm's seven patterns listed above,
including its current rectilinear_multiline call, solely for the regional
flag assignment shared by their current direction kernel. No new pattern
admission or subclass geometry is included. The first measured public RED
is Rectilinear on a single classic object/instance without raft.

Deferred: other anchoring pattern arms (Line, Grid/Triangles/Stars,
Honeycomb/lattice, cubic/adaptive/lightning, TPMS, concentric and plane
paths); any separate subclass-specific geometry defect; main object
locked_zag and other dedicated fill generators still lacking the regional
bit at Fill.cpp:1235-1242; ironing's separate Fill.cpp:1631 assignment;
support/raft/tower option domains, raft/physical-layer remapping changes,
multi-object/instance, Arachne and new modifier witnesses. Threading a bit
through this shared arm does not establish every ZAA-pattern combination's
whole-output parity. Preserve all existing controls and name residual gaps.

## Native admission/activity before the public RED

Only outside FULL14 and later exclusive windows, use CPU13 official and
CPU14 seeded, jobs2/threads2 and new Native root/cache/output per invocation.
Start from the existing admitted positive-density internal-ceiling input
setup in bridge_over_infill_zero_density.rs (cube10,20% Rectilinear sparse
fill, classic walls, support/tower disabled). Prepare only input bytes and
retain original mesh/member SHA. Enable ZAA with the existing valid minZ0.05.
Do not assume contoured XYZ extrusion must occur on this flat geometry:
the activity under test is the flag's effect on sparse anchors/bridges.

Probe ZAA1/flag0 and ZAA1/flag1 with both Native modes and fresh repeats.
Require successful Native export, complete footer with the exact requested
options, positive sparse and Internal Bridge extrusion roles, and a changed
Native Internal Bridge geometry/direction stream between the flag states.
Choose a ceiling whose supporting sparse layer can exercise the alternating
orientation; if the initial admitted ceiling has no such witness, report
inactivity and seek a bounded fixture plan rather than fabricate a RED.

The ordinary-fill flag already works at this base. Consequently a changed
Native ordinary infill stream or an M73 difference cannot by itself prove
that the anchoring call site is active. Require the bridge-role witness and
complete-output fail-on-head, with a passing false control. If Native rejects,
crashes, is unstable, or Ares already matches complete output, STOP and
report; do not alter numeric policy, comparator or expected results.

## Independent public E2E RED, after SPEC approval

Build an input-only fixture from the admitted ceiling/profile setup; do not
run another test's whole-output assertion or derive geometry from an Ares
slice result. Keep fixture construction separate from comparison. Verify
public loaded options and unchanged ZIP members. No committed Native
artifacts or private-stage assertions.

Include all three cases in the RED commit itself:
1. enabled ZAA / flag false, complete-output control;
2. enabled ZAA / flag true, complete-output PASS requirement expected to RED;
3. disabled ZAA / flag true, complete-output conjunction control.

Use OrcaRunner::from_env().unwrap() with None=>return by repository
convention; required-Native gates use ARES_REQUIRE_ORCA=1. Force fresh Native
reruns; repeat each case twice. Require Native/API/CLI success and API/CLI
raw equality, then the original compare_case and generator-only ordered
comparison. Every case asserts PASS, never expected DIVERGENT. Before its
final verdict, the true test must establish Native Internal Bridge activity
against its false control and stable normalized/bridge SHA across repeats.
Retain input/native/API/CLI SHA, exact first difference, roles, geometry and
cache paths in external repeatable artifacts. Record geometric discrepancy
even when progress accounting is the first whole-output difference.

Submit the test-only RED hash and both modes' admission/activity/failure
evidence for review before production GREEN. If minimal scoped wiring
cannot satisfy unchanged assertions later, STOP before broadening scope.

## Gates and scheduling

After accepted RED and separate GREEN authorization, require three public
cases both modes, ordinary ZAA direction and modifier/sloped-roof controls,
existing bridge/anchor and rotation-template/fixed-angle controls. False
and disabled controls retain their original raw Ares bytes. Run fmt, strict
core/CLI-test clippy and wasm32 check; every file <=400 LOC. No suppression,
ignore/debug macro, guard/fallback, regions[0] or weakened assertion.

Only directed work on CPU13/14 with outer taskset and jobs2/threads2;
full/workspace remain coordinator-only. Do not begin Native/cargo/nextest
inside FULL14. Abort/VOID owned runs on later EXCLUSIVE FULL START and wait
for END. No push. This SPEC submission runs no build, test or Native job.

## Authorized bounded ceiling amendment after the initial NO-WITNESS

The initial10mm cube has Native Internal Bridge on one-based layer46;
its supporting object layer is index44, an even layer. Both flag states
therefore use the same base direction for that support layer:
FillBase.hpp:211 alternates only odd indices; FillBase.cpp:311-313 can
suppress that alternation. This explains an inactive initial witness
without claiming the source mismatch has no reachable effect.

Probe only the following three fixed input variants, in this order, stopping
at the first stable bridge-geometry delta. All retain the admitted source
profile,20% Rectilinear sparse infill,0.2mm layers,5 top shell layers and
zero internal_bridge_angle override. Change only the named mesh coordinates
and the previously specified ZAA options; retain bottom z=-5 and instance
placement. Native actual bridge layers are observed, not assumed.

| Candidate ceiling | Exact mesh amendment | Source justification |
| --- | --- | --- |
| Parity cube |10x10 XY, top z5->5.2 (height10.2mm) | One extra0.2mm layer should move the first top ceiling and its supporting sparse layer to opposite parity. Fill.cpp:1442 and FillBase.hpp:211 / FillBase.cpp:311-313 govern that layer's direction; PrintObject.cpp:2735-2754 reads that supporting layer. |
| Parity rectangle |20x10 XY (x multiplied by2), same10.2mm height | The same parity change plus a non-square boundary changes the sampling weights in PrintObject.cpp:2863-2893 determine_bridging_angle, which scores nearby anchor directions. |
| Rotated parity rectangle |Previous20x10 rectangle rotated23 degrees about mesh XY origin, same10.2mm height | A fixed non-axis-aligned boundary changes sampled anchor distances/orientations in the same source lambda; PrintObject.cpp:3244-3250 distinguishes actual anchors from boundary-only fallback. |

For each attempted candidate, Native-only flag0/1 official+seeded with
fresh cache/output and two repeats per state. Require accepted output,
exact requested footer options, positive sparse/Internal Bridge roles,
stable per-state geometry and cross-mode agreement. Geometry observation
uses layer/start/end XYZ for positive Internal Bridge extrusion, excluding
comments, feedrate, extrusion amount and metadata. Ordinary sparse changes
are insufficient. No API/CLI run, test or production edit in this bounded
amendment. If all three lack a bridge delta, close this slice NO-WITNESS and
propose a different upstream-cited Native-RED slice; do not keep searching.
