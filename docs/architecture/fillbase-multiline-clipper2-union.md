# SPEC: FillBase multiline Clipper2 Positive union

Status: SPEC-REVIEW-REQUEST. Baseline `48821ce89` (upstream `8500fcdcc`).
This branch contains only this spec. Native execution, tests and production
changes begin after approval. Stars mode2 WIP is saved locally as `6f3066116`
on `worker6/stars-mode2-wip`, not pushed or stashed.

## Owning upstream boundary

`libslic3r::multiline_fill` (`src/libslic3r/Fill/FillBase.cpp:2712-2784`)
expands sparse centerlines into round-ended bundles using Clipper2. Its offset
operation includes a closed-polygon union. This dependency belongs to every
caller of multiline_fill; it must not become a Stars-specific approximation.
The existing Ares round offset is a temporary compatibility shell around this
source operation until its union matches the cited Clipper2 implementation.

| Upstream source | Included contract |
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

Read-only gdb captures on the Stars WIP show all13 S2 emitted layers have
exactly equal ordered integer rows and exactly equal raw round-offset points
before union. After union, Ares and Native differ: first phase13104/13104 points
with an integer-coordinate difference, other phases13444/13489 and13439/13492.
The stage audit is `/tmp/worker6-stars-union-stage-audit.json`; its referenced
raw arrays and `/tmp/worker6-stars-union-boundary.md` are outside the repository.
This proves a Stars-reached union mismatch, not a non-Stars Native byte RED on
main. No test, fixed array or Native capture is imported into this branch.

Current Ares `fill/multiline_offset/round.rs:57` calls
`geometry::union_polygons_paths`; `geometry/clipper/boolean_paths.rs:8-9/:93+`
uses the existing Clipper1 NonZero kernel. That helper serves other upstream
Clipper1 consumers. Changing it globally is outside scope.

## Rust destination

Included only after an attributable non-Stars RED:

- `crates/ares-core/src/fill/multiline_offset/union.rs` and private children,
  each<=400 lines: source-cited Rust rewrite of the closed Paths64 Positive
  union used inside ClipperOffset::ExecuteInternal. Retain upstream ordering,
  winding, collinear cleanup and intersection semantics. No public API,
  filesystem, Native process, C++/FFI dependency or platform-only implementation.
- `fill/multiline_offset.rs`: one private module registration if needed.
- `fill/multiline_offset/round.rs::offset`: replace only its final Clipper1
  union call with the new Clipper2 operation. Preserve round generation and
  bundle caller behavior; do not globally replace geometry::Clipper.
- `ares-cli/tests/orca_parity/option_coverage/multiline_union.rs` and dedicated
  helpers: whole-output runtime Native controls and stage-attribution evidence.
  `option_coverage.rs` gains only the module registration. No domain plan,
  inventory count, producer config or comparator modification.

The new module is a bounded Clipper2 rewrite, not a second slicing pipeline or
an adapter selecting union semantics by pattern. All multiline callers share
the same source operation. If proving or implementing the boundary requires
broader changes, report before editing them.

## Native-backed RED admission and attribution

First run main production at48821ce89. Single Ender-3 .4 nozzle/PLA, by-layer,
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
  likewise cannot authorize a union change.
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
RED. No Native rebuild, retry hiding failures or source probe is authorized.
If differences begin at dispatch, spacing, angle, bbox, clipping or connection,
record them separately and stop that candidate's attribution; do not absorb
those ports to manufacture this RED.

If the finite non-Stars controls already match main, report NO RED FOUND and
stop before GREEN; the current proven mismatch remains Stars-local reachability
for coordinator adjudication. If only unrelated dispatch differences appear,
report that no attributable shared-union RED was found rather than claiming
this prerequisite justified. Native rejection, rewrite or timeout is reported;
timeout is not evidence and awaits a lower-load rerun with new cache/root.

## Included / deferred behavior

Included: positive closed-path union required by positive, constant,
round-join/round-ended multiline offsets; upstream path order and numeric
semantics; portable pure Rust. Count<=1 bypass remains unchanged, and odd
bundles retain their unmodified centerline order before offset rings.

Deferred: Stars mode2 and anchoring wiring (local WIP), Grid/Triangles mode
ports, Rectilinear dispatch correction, no-connect admission, negative/polygon
ended offsets, variable delta callbacks, Z data, PolyTree output, open-subject
Clipper2 boolean operations, Difference/Intersection/Xor, general NonZero or
EvenOdd kernels, a global Clipper1 replacement, new settings and geometry APIs.
Do not port UB or invent a simplified union that merely fits the finite inputs.

## Acceptance and regression evidence

After approved spec and attributable RED, commit RED before shared GREEN.
Each directed execution gets a completely new Native cache and artifact root;
there is no cross-run cache reuse. Preserve all first outcomes and request,
archive member, producer and output hashes outside git. Required dependencies
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
rustfmt and wasm32 check on the accepted tree. Directed load<60/jobs2/threads2;
workspace/strict load<30/jobs2. Production GREEN also queues fresh full according
to coordinator slots and shared-base policy; each leg has its own fresh cache
and artifact root. Require cand.fail subset of corresponding exact-main base.
Report spec/RED/GREEN commits and all evidence via REVIEW-REQUEST before push.
