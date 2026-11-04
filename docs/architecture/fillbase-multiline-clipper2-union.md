# SPEC: FillBase multiline Clipper2 Positive union

Status: Accepted; numeric-only GREEN.
Upstream `8500fcdcc`.

## Owning upstream boundary

`libslic3r::multiline_fill` (`src/libslic3r/Fill/FillBase.cpp:2712-2784`)
expands sparse centerlines into round-ended bundles using Clipper2. Its offset
operation includes a closed-polygon union. This dependency belongs to every
caller of multiline_fill; it must not become a Stars-specific approximation.
The existing Ares round offset is a temporary compatibility shell around this
source operation until its union matches the cited Clipper2 implementation.

| Upstream source | Source contract (numeric slice inclusion below) |
| --- | --- |
| `Fill/FillBase.cpp:2712-2740` | Count<=1 returns; remove invalid centerlines; convert ordered source paths; odd count retains original centerlines. |
| `Fill/FillBase.cpp:2742-2784` | Ordered odd/even ring distances, float spacing boundary, one round offset group, ring order, skip empty/degenerate results, append closing point exactly once. |
| `deps_src/clipper2/Clipper2Lib/src/clipper.offset.cpp:137-159/:558-569` | Round-ended open path groups are not reversed; CheckReverseOrientation only considers polygon-ended groups. Therefore multiline_fill reaches the Positive union with ReverseSolution(false). |
| `clipper.offset.cpp:604-630` | PreserveCollinear(false), ordered AddSubject, ClipType::Union with FillRule::Positive, closed Paths64 result. This cleanup occurs inside Execute despite FillBase's comment saying no separate union is needed. |
| `include/clipper2/clipper.engine.h:59-146/:196/:470` | OutPt/OutRec, Active, LocalMinima and ClipperBase/Clipper64 data boundaries for the needed closed-path union. |
| `src/clipper.engine.cpp:120-138/:840-845` | Edge slopes, TopX rounding and input registration. Preserve upstream coordinate arithmetic; do not emulate Clipper2 by changing Clipper1 rounding globally. |
| `clipper.engine.cpp:912-986/:1015-1091/:2123-2152` | Positive winding/contribution, scanbeam/local-minima processing, horizontal edges, intersections and output joins needed for closed union. |
| `clipper.engine.cpp:1531-1587/:2341-2382/:2983-3020` | Collinear cleanup/self-intersection splitting, intersection ordering and BuildPaths64 traversal. Output order, start vertex, point count and orientation are part of byte parity. |

The Clipper2 paths above are relative to `OrcaSlicer/deps_src/clipper2/Clipper2Lib`;
Fill paths are relative to `OrcaSlicer/src/libslic3r`. Source files remain the
algorithm authority, including integer casts and floating rounding. Existing
Ares geometry is not a basis for independently redesigning union behavior.

## Existing evidence and what it does not prove

Read-only gdb captures on the Stars implementation show all13 S2 emitted layers have
exactly equal ordered integer rows and exactly equal raw round-offset points
before union. After union, Ares and Native differ: first phase13104/13104 points
with an integer-coordinate difference, other phases13444/13489 and13439/13492.
This proves a Stars-reached union mismatch, not a non-Stars Native byte RED on
main. No test, fixed array or Native capture is imported into this branch.

Current Ares `fill/multiline_offset/round.rs:57` calls
`geometry::union_polygons_paths`; `geometry/clipper/boolean_paths.rs:8-9/:93+`
uses the existing Clipper1 NonZero kernel. That helper serves other upstream
Clipper1 consumers. Changing it globally is outside scope.

## Rust destination

The initially proposed complete private union kernel is not implemented. After
the attributable C2 RED and the failed Positive-only trial, only a generic
numeric strategy is adopted:

- `fill/multiline_offset/numerics.rs`: private default Clipper2 segment arithmetic
  and scanbeam correction; each new file<=400 lines.
- `geometry/clipper/round_policy.rs`: immutable numeric edge inputs and a trait
  with the unchanged Clipper1 default; `engine.rs`/`intersections.rs` thread the
  type parameter. No runtime function pointer, state field or public API.
- `geometry/clipper/boolean_paths.rs`: one multiline-only Positive helper;
  `fill/multiline_offset/round.rs` selects its private numeric strategy, preserving
  raw round geometry and all other helpers/callers.
- `ares-cli/tests/orca_parity/option_coverage/multiline_union.rs` and its observer:
  whole-output runtime Native controls; one module registration only. No domain
  plan, inventory, producer config or comparator change.

The shared operation is selected by its multiline source boundary, not pattern.
Complete sweep, event order, joins, cleanup and traversal ports remain deferred
unless residual whole-byte evidence requires them.
The current numerical experiment does not redesign union as an Ares pipeline.

## Native-backed RED admission and attribution

First run main production. Single Ender-3 .4 nozzle/PLA, by-layer,
classic two walls, .2mm layers, positive30% sparse fill, stock positive anchors,
three solid bottom/top layers, no support/raft/brim/tower, arcs, scarf, fuzzy
skin, SAFC, adaptive PA, XY compensation or infill combination. Use authored
64x48x4mm centered prism and its8x8mm through-hole variant. The authored STL
sources may be copied from the Stars RED with their recorded provenance; no
Native output, array, receipt or hash table is committed. Native export resolves
profiles outside git and both slicers consume identical exported 3MF bytes.

Finite first controls (both seeded and official, fresh per-run cache/root):

| Controls | Pattern / count | Purpose |
| --- | --- | --- |
| R1/R2 | rectilinear /1,2 | First requested non-Stars comparison; verify actual source dispatch before assigning cause. |
| G1/G2 | grid /1,2 | Second requested comparison; mode0 generator is an explicit attribution concern. |
| C1/C2/C3 | cubic /1,2,3 | Existing ordinary multiline caller; even and odd bundle witnesses. |
| H1/H2/H3 | honeycomb /1,2,3 | Existing bent-centerline caller with round corners and overlapping offset edges. |
| HC2 | holed prism, cubic /2 | Hole/exterior clipping control after the shared offset dependency. |
| HH2 | holed prism, honeycomb /2 | Hole/exterior and bent-centerline control. |

Source dispatch cautions are already visible offline:

- Native Rectilinear count>1 takes fill_surface_by_multilines
  (`FillRectilinear.cpp:3387-3399`), while main Ares
  `fill_entities/monotonic.rs:97-107` selects fill_rectilinear_surface and its
  MonotonicFillParams does not carry multiline. A resulting dispatch mismatch
  is not proof of the shared union defect; do not fix that caller in this slice.
- Native Grid count>1 uses trapezoidal mode0 (`:3422-3446`); main Ares
  `fill_entities/grid.rs:62-86` still uses ordinary sweeps. A mode0 mismatch
  likewise cannot justify a union change.
- Native Cubic uses ordinary multiline sweeps (`:3506-3515`, with bundle
  expansion at `:3021`); Honeycomb calls multiline_fill after row rotation
  (`FillHoneycomb.cpp:70-77`). Ares cubic and honeycomb reach the shared offset.

Admission requires Native success, requested pattern/count retained, real
positive-XYE sparse paths, multiple sparse layers and reached offset operations.
Compare complete ordered output bytes and line count using the existing
single generated-by normalization; retain M73, travel, flow, repeated vertices,
footer and configuration. No polygon-only or filtered acceptance comparison.
The failing committed test still asserts whole-output PASS.

For each non-Stars DIVERGENT, retain the earliest full-output and sparse-path
first differences. Read-only gdb on existing Native and Ares binaries may
attribute the failure: compare centerline arrays, raw round-offset arrays and
post-union results outside git. Identical pre-union arrays plus differing union
results and a downstream output difference are the necessary shared-dependency
RED. No Native rebuild, retry hiding failures or source probe is permitted.
If differences begin at dispatch, spacing, angle, bbox, clipping or connection,
record them separately and stop that candidate's attribution; do not absorb
those ports to manufacture this RED.

If the finite non-Stars controls already match main, report NO RED FOUND and
stop before GREEN; the current proven mismatch remains Stars-local reachability.
If only unrelated dispatch differences appear,
report that no attributable shared-union RED was found rather than claiming
this prerequisite justified. Native rejection, rewrite or timeout is reported;
timeout is not evidence.

## Included / deferred behavior

Included: positive closed-path union required by positive, constant,
round-join/round-ended multiline offsets; upstream path order and numeric
semantics; portable pure Rust. Count<=1 bypass remains unchanged, and odd
bundles retain their unmodified centerline order before offset rings.

Deferred: Stars mode2 and anchoring wiring, Grid/Triangles mode
ports, Rectilinear dispatch correction, no-connect admission, negative/polygon
ended offsets, variable delta callbacks, Z data, PolyTree output, open-subject
Clipper2 boolean operations, Difference/Intersection/Xor, general NonZero or
EvenOdd kernels, a global Clipper1 replacement, new settings and geometry APIs.
Do not port UB or invent a simplified union that merely fits the finite inputs.

## Acceptance and regression evidence

After the spec and attributable RED, commit RED before shared GREEN.
Each directed execution gets a completely new Native cache and artifact root;
there is no cross-run cache reuse. Required dependencies
fail through native_environment; optional missing Native is visibly SKIPPED.
No ignore, suppression, guard, fallback or fixed Native product is added.

Run option_coverage entire group in both modes, including its pure tests, on
base and candidate with separate fresh roots/caches. Other callers' Native
requests and Ares raw bytes must be unchanged except explicitly proven RED
cases repaired by this union port. List every changed stream; unrelated byte
changes block acceptance even when both outcomes say PASS. Existing assertions
and domain expectations remain unchanged. Count1 controls must match entirely.
The caller inventory includes multiline, honeycomb, concentric_sparse,
plane_path, gyroid, tpms_d/fk, three_d_honeycomb, cross_hatch, adaptive/filler,
lightning and anchoring; report which are reached by existing controls, and any
coverage gap before claiming complete invariance.

Engineering gate: default-environment nextest workspace; strict clippy,
rustfmt and wasm32 check on the accepted tree. Production GREEN also requires a
fresh full run; each leg has its own fresh cache and artifact root. Candidate
failures must be a subset of the corresponding exact-main base failures.

## Attributable non-Stars RED

Fresh seeded and official runs each give4 PASS /8 DIVERGENT for the12 controls,
without rejection, dependency failure or timeout. Count1 R1/G1/C1/H1 all pass.
R2/G2 remain unrelated dispatch findings and do not justify this union port.

C2 is an attributable shared-union RED: read-only gdb of existing seeded Native
and unchanged Ares captures14 multiline calls (anchoring plus emission). Every
ordered centerline array and every ordered raw round-offset polygon array is
exactly equal. Each post-union array differs. Native gdb output is independently
whole generator-only equal to the runtime Native C2 output; Ares gdb output is
raw-byte equal to the runtime Ares output. Runtime C2's first whole difference is line204 M73 placement; its first sparse
path difference is layer4: Native line1425 `G1 X138.625 Y126.024 E.22346`, Ares
line1423 `G1 X140.369 Y119.517 E.22346`. The whole-output comparator requires
PASS throughout; these are diagnostics, not filtered acceptance comparisons.

First GREEN experiment: retain the existing Clipper1
kernel and change only the multiline round offset's union fill rule to
Positive. No private Clipper2 kernel or external crate may be introduced before
this experiment and a report of any remaining concrete difference. No claim
is made that selecting Positive reproduces Clipper2 numeric/order semantics.

## Positive-only experiment: not GREEN

The minimal trial selects existing Clipper1 Positive fill for the
round offset's final union. Its private geometry types cannot be imported by
fill, so the trial adds one crate-private Positive union helper with two thin
re-exports; the original NonZero helper and all its other callers are unchanged.
No external crate, new kernel, assertion change or other generator edit occurs.

A fresh seeded trial still yields 4 PASS /8 DIVERGENT. All12 Ares raw outputs
are byte-identical to RED; all Native generator-normalized outputs and all
corresponding 3MF member bytes are also equal. This rules out changing only the
winding fill selection as a sufficient repair.

Concrete C2 residual: whole first difference remains line204 M73 placement;
first sparse path remains Native layer4 line1425 X138.625 Y126.024 E.22346
versus Ares line1423 X140.369 Y119.517 E.22346. The RED stage captures show
identical pre-union arrays,93 result paths on each side, and first union
coordinate Native(2578819,-23832823) versus Ares(2578820,-23832823).
Clipper2 `clipper.engine.cpp:131-138` uses nearbyint in TopX and
`:2350-2382` uses its segment intersection/clamping pipeline; existing Clipper1
`geometry/clipper/predicates.rs:8-15/:33-104` has its own rounding/intersection
arithmetic. Both kernels also own output joining/cleanup/traversal. A fill-rule
flag does not port these contracts. This is a source-boundary explanation,
not a claim that one rounding edit alone fixes the complete difference.

The trial is not an accepted GREEN. A complete private union kernel is declined
pending numeric attribution. Exact round-input equality justifies retaining the
existing round generation and focusing on the union dependency; it does not
justify globally changing Clipper1. R2/G2 need separately recorded
dispatch findings rather than retained unrelated failing acceptance tests.

## First coordinate attributed to segment-intersection conversion

Read-only gdb of the existing Native binary captures the actual edge pair in
`ClipperBase::AddNewIntersectNode` and its resulting `IntersectEdges` argument.
The same pair is captured in Ares `predicates::intersect_point` and
`Clipper::intersect_edges`. Both have identical endpoints, slopes and current X:

| Edge | Bottom | Top | Current X |
| --- | --- | --- | --- |
| First | (2556061,-23829565) | (2581534,-23833212) | 2581115 |
| Second | (2585889,-23832438) | (2560194,-23833839) | 2572794 |

Native's actual SSE conversion operands are X=2578819.991888218 and
Y=-23832823.43219944. Both conversion instructions are `cvttsd2si`, producing
(2578819,-23832823). This implements the default
`clipper.core.h:898-917 GetSegmentIntersectPt`, specifically the truncating
`static_cast<T>` at lines914-915. It is not the alternate high-precision branch
and does not round X to nearest. Ares `predicates.rs:67-78` uses slope/intercept
arithmetic and `fixed_round` (`:8-15`), producing (2578820,-23832823).
Ares gdb also captures the Y quotient -23832823.432199441 before rounding.
The coordinate difference is therefore attributable to numeric semantics, not
an assumed input, slope or event-order difference.

The scanbeam bottom is -23832438 and top is -23833152. The intersection lies
inside it, so Native `clipper.engine.cpp:2358-2379` does not clamp through TopX.
This point does not justify changing TopX, event sorting, output traversal or
porting the complete sweep. It also does not prove that fixing this one numeric
routine will repair every result path; runtime whole-byte comparisons remain
the acceptance gate.

The completed Native diagnostic output is generator-only byte-equal to the
runtime C2 reference and the completed Ares diagnostic output is raw byte-equal
to runtime C2.

## Narrow numeric integration

The integration is numeric-only, replacing the proposed
runtime function pointer with a zero-cost generic trait strategy. There is no
complete Clipper2 sweep, output-coordinate postprocessing, new crate or change
to the comparator.

`fill/multiline_offset/numerics.rs` owns the default Clipper2 segment-intersection
arithmetic (`clipper.core.h:898-917`), the scanbeam correction
(`clipper.engine.cpp:2350-2382`), and the correction's numeric dependencies
TopX (`:131-138`) and integral closest-point projection
(`clipper.core.h:971-990`). The default branch casts doubles toward zero;
nearbyint in the correction uses ties-to-even under the default rounding mode.
This does not change shared Clipper1 TopX scanbeam sorting or event ordering.

`geometry/clipper/round_policy.rs` defines the crate-private `RoundPolicy` and
immutable numeric edge snapshot, plus the default `Clipper1Round` strategy.
The latter retains the existing slope/intercept/fixed_round calculation and
its final top_y clamp. `engine.rs` threads a type parameter through execution;
`intersections.rs` calls the statically selected strategy. Existing paths,
PolyTree and Z executions explicitly use the default. The multiline-only
Positive helper uses `Clipper2Truncate`. No runtime function pointer or policy
field is added to Clipper; its state layout is unchanged. The upstream numeric
implementation remains fill-private, and internal Edge/KernelPoint types are
not exposed outside geometry. Both new files stay below400 lines.

The initial12 complete-output controls were run in both modes with fresh
caches; the acceptance set now contains ten, with R2/G2 deferred.
Default-policy callers require raw-byte invariance proof and full
option_coverage in both modes, with strict/fmt/wasm32. Any residual sweep or output
traversal difference must be attributed and reported before widening scope.

## Numeric experiment result; unrelated dispatch controls still open

Fresh seeded and official runs each produce10 PASS /2 DIVERGENT, with no
rejection, skip or timeout. C2/C3/H2/H3/HC2/HH2 become whole-byte PASS in both
modes. R1/G1/C1/H1 remain PASS and each Ares raw output is byte-identical to the
Positive-only trial. All12 Native generator-normalized outputs in both modes
are also equal to that trial.

R2/G2 remain actual failing controls: Rectilinear's multiline dispatch and
Grid's mode0 dispatch are outside this union boundary. R2 Ares raw is unchanged;
G2 Ares raw changes because its anchoring stage already calls shared multiline
expansion, while its emission dispatch remains incomplete. Their first whole
Native/Ares differences are both line18 M73: R2 R38/R37 and G2 R45/R46.
These failures are not timeout, nondeterminism or accepted union GREEN.
R2/G2 are moved out of this union acceptance registration and preserved as
Deferred findings below. The ten remaining tests keep every existing assertion
and the comparator is unchanged.

## Deferred findings

R2 and G2 are real DIVERGENT findings, not accepted PASS or union RED. Their
registered test functions are removed by an explicit scope decision; their
authored input, runtime references and complete failed
comparisons remain reproducible outside git. They follow Stars mode2 as the
Rectilinear/Grid multiline dispatch rewrite slice.

Both use unchanged `tests/parity/multiline-union/prism.stl` and the fixed22
requests in `option_coverage/multiline_union.rs`: count2, 30% density, 45 degrees,
classic2 walls, three top/bottom layers, standard positive anchors, single
Ender3 PLA .4mm nozzle and .2mm layers. Pattern is respectively rectilinear/grid.
The mesh SHA is1d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e.
Effective requests were verified against the runtime Native exported project.

| Finding | Native/Ares first whole line | Upstream target | Ares target |
| --- | --- | --- | --- |
| R2 | line18 `M73 P0 R38` / `M73 P0 R37` | `FillRectilinear.cpp:3387-3399`, Rectilinear main multiline dispatch | `project_slice/fill_entities/monotonic.rs:97-107`, multiline missing from ordinary parameters |
| G2 | line18 `M73 P0 R45` / `M73 P0 R46` | `FillRectilinear.cpp:3422-3446`, Grid multiline>1 trapezoidal mode0 dispatch | `fill/grid.rs:62-86`, ordinary sweeps |

No Native artifact is committed. No tolerance, assertion change or production dispatch repair absorbs these
findings into the current union slice.
