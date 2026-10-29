# S-C0 revision: sparse-anchoring base fill entry inset

## Status and supersession

SPEC-REVIEW-REQUEST, docs only, on main3ee8f272c plus the approved S-C0 RED
c15b29c60 and localization documents. Production remains unchanged.
This revision makes the earlier anchoring entry operation the first boundary.
It supersedes the implementation order of SPEC37145ce12 and addendum a11e509bd:
fix/re-observe this input operation first. The hook extension is conditional
on a remaining residual after the inset; do not edit hooks in the initial patch.
The accepted STOP0fb0a0ac5 and its durable evidence establish a real missing
operation, rather than different-but-equivalent bbox conventions.

## Owning upstream and Rust destination

All upstream paths below are under `OrcaSlicer/src/libslic3r/`:

1. `PrintObject.cpp:2744-2754` generates preceding-layer sparse anchoring
   lines during bridge-over-infill preparation.
2. `Fill/Fill.cpp:1394-1405`, `Layer::generate_sparse_infill_polylines_for_anchoring`,
   groups and selects internal surfaces; 1440-1453 constructs the configured
   filler and sets layer id, Z, angle and adaptive/support octree.
3. `Fill/Fill.cpp:1492-1498` resets spacing for each expolygon and calls
   `f->fill_surface(&surface_fill.surface, params)` on the original domain.
4. `Fill/FillBase.hpp:181-187`, `Fill` constructor, sets overlap0; the cited
   anchoring setup leaves that default intact.
5. `Fill/FillBase.cpp:105-117`, `Fill::fill_surface`, calls
   `offset_ex(surface->expolygon, float(scale_(overlap - 0.5*spacing)))`,
   then dispatches every resulting expolygon to `_fill_surface_single`.
6. `Fill/FillAdaptive.cpp:1321-1326` receives the offset expolygon;
   1378 clips generated lines to it, 1401 applies hooks and 1410 connects.

Production destination is ONLY
`crates/ares-core/src/project_slice/fill_entities/adaptive.rs::anchoring_lines`
(current248-282): apply the missing base entry operation before calling
`crate::fill::adaptive::filler::fill_surface` for each resulting component.
This serves both AdaptiveCubic and SupportCubic dispatches from
`prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`.
The current helper is a temporary shell around the upstream base/pattern
entry boundary; replace its direct unmodified-domain forwarding. Do not add
an independent Ares geometry stage or a global anchoring wrapper.

W6 owns the same file's `build_object_octree` coordinate work and may use
`fill/adaptive/octree.rs` for that work. Neither is in this patch. The
functions do not overlap: whichever chain lands second rebases; report a
conflict before resolving any shared-file production hunk.

## Included operation and exact arithmetic

Use the Native anchoring default **overlap0**, not an arbitrary
`fill.params.overlap` copied from emission. Compute the completed scaled
DOUBLE expression `(0.0 - 0.5*spacing) / scale.factor()`, then cast ONCE to
f32, matching `float(scale_(...))`. Do not narrow spacing first, round the
scaled delta to an integer, or multiply by a reciprocal approximation.
Pass that f32 delta to the existing `offset_expolygon`, Miter join, limit3,
matching the default `offset_ex` parameters. The geometry kernel receives
the completed f32 value; retain its existing widening/coordinate rounding.
No offset-kernel change belongs to this slice.

For the observed spacing0.40707963705062866mm, the completed delta is
-203539.8125 lattice units; the actual square contour inset is203540 units
per side. These are evidence in the STOP document, not committed expected
Native output for a test. Preserve holes and iterate all returned components
in source order. An offset producing no components naturally contributes
no polylines, as upstream does; do not restore the original polygon or add
a special guard. Retain the helper's existing error propagation.

Do not change ordinary `adaptive::append`: its entry offset already matches
Native on the inspected Z0.8 control. Keep fill parameters, octree selection,
coordinates, hooks, connector/grid geometry, ordering and public contracts
unchanged in the initial inset patch. Rust files must remain <=400LOC.

## Other anchoring caller audit (source only)

Audited every branch reachable from `sparse_anchoring/arms.rs`, its
`arms_tail.rs`, `linear.rs` and `arms_tail/lightning.rs` at this head.
The table records entry-operation ownership; it does NOT claim runtime parity
for every pattern. Rust file names below are relative to `crates/ares-core/src/`.
Do not prepend a generic inset to the dispatch: several branches already own
one, while the rectilinear family has an upstream override rather than the
base entry (`FillRectilinear.hpp:18-158`).

| Anchoring family | Existing operation owner | Audit disposition |
| --- | --- | --- |
| AdaptiveCubic / SupportCubic | `project_slice/fill_entities/adaptive.rs::anchoring_lines` forwards directly to pattern filler | Proven missing inset; sole initial implementation scope |
| CrossHatch | `fill/cross_hatch.rs::fill_surface` offsets overlap-minus-half-spacing | Entry present; no extra dispatch offset |
| Gyroid | `fill/gyroid.rs::fill_surface` offsets before components | Entry present |
| 3DHoneycomb | `fill/three_d_honeycomb.rs::fill_surface` offsets before components | Entry present |
| Line | `fill/line.rs::fill_line` offsets then calls pattern single-domain function | Entry present |
| Concentric | `sparse_anchoring/arms.rs` explicitly offsets zero-overlap domains before `concentric_sparse` | Entry present |
| TPMS-FK | `sparse_anchoring/arms_tail.rs` explicitly offsets before `tpms_fk` | Entry present |
| Plane paths: Hilbert / Archimedean / Octagram | `fill/plane_path.rs::fill_surface` offsets before components | Entry present; no blanket extra inset |
| Rectilinear / ZigZag / Monotonic / MonotonicLine / AlignedRectilinear / CrossZag / LockedZag | `sparse_anchoring/linear.rs` routes to `rectilinear/surface.rs` or `rectilinear_multiline.rs`; surface contraction lives there or in `multiline.rs` | Pattern-specific entry present; do not replace with base wrapper |
| Grid / Triangles / Stars (TriHexagon), Cubic / QuarterCubic / LateralLattice / LateralHoneycomb | `fill/multiline.rs` contracts the domain; multiline trapezoidal paths use `fill/trapezoidal_tail.rs::finish` | Source-derived custom entry/tail contraction present; do not double-offset |
| Lightning | `sparse_anchoring/arms_tail/lightning.rs::append` offsets domains before tree conversion | Entry present |
| Honeycomb | `sparse_anchoring/arms.rs` directly calls `fill/honeycomb.rs::fill_honeycomb`; that function clips original input with no base inset | Additional source-level omission candidate, deferred; requires its own Native/public RED |
| TPMS-D | `sparse_anchoring/arms_tail.rs` directly calls `fill/tpms_d.rs::fill_tpmsd_surface`; base inset absent on that route | Additional source-level omission candidate, deferred; requires its own Native/public RED |

For the two additional candidates, `FillHoneycomb.hpp:12-25` and
`FillTpmsD.hpp:14-32` override `_fill_surface_single` but inherit
`Fill::fill_surface`; upstream anchoring therefore still takes the base inset.
Their ordinary Ares emission adapters (`fill_entities/honeycomb.rs` and
`fill_entities/tpms_d.rs`) explicitly offset, strengthening the call-path
finding. No new Native run, fixture or production fix for either is included.
Potential parameter/precision discrepancies inside the other audited entries
are not ruled out by this source-only presence audit and remain deferred.

## RED reuse, sequence and residual gate

Reuse EXACT approved S-C0 authored input SHA256
9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1
and its approved public test c15b29c60 unchanged in assertions/comparator.
It exercises runtime official/seeded Native, `slice_project_plate_validated`,
real CLI, strict whole-output comparison and raw API==CLI equality. Its
current panic is a real failure, never an accepted result or parity PASS.
No new isolated test or frozen Native output is needed for this revision.

After SPEC approval:

1. Implement only the anchoring inset, plus the already requested stable
   test-comment wording: describe CLI/API evidence order without current
   state/history. No assertion weakening or test retirement.
2. Re-observe the first anchoring filler/connector input against Native:
   matched layer45/Z9.2 domain must be the offset one. Check whole-output
   parity in both modes before attributing any residual to hooks.
3. If any earlier intermediate still differs, STOP at that first difference.
   Only if the entry inset is correct and a downstream residual remains,
   activate the approved a11e509bd hook-localization extension. Identify its
   actual limiting collision candidate/expression before editing hooks;
   no trial-and-error production changes or simultaneous speculative fixes.
4. GREEN acceptance remains full S-C0 strict equality plus unchanged B-S
   ten-case `runner/adaptive_cubic_precision.rs` matrix, existing connector
   controls, Adaptive controls and strict/fmt/wasm32. A residual means STOP,
   not normalized comparison, retries, skipped input or new rejection.

All runtime/build commands use outer taskset CPU12; directed jobs1/threads2,
canonical mode/common variables and fresh Native cache/artifact roots each
run, START/END to inbox. Full/workspace/CI/push/merge remain coordinator-only.
No unsafe, new allow/expect, probes, guards, fallback or panic swallowing.
The original grid assertions stay; no evidence currently justifies removal.

## Evidence and deferred work

Accepted first-unequal proof: `support-cubic-anchoring-inset-preflight-stop.md`
(0fb0a0ac5) and durable root
`/home/indexyz/ares-evidence/worker3/support-touching-grid-bbox-preflight/20261015-124632`.
No new runtime experiments were run for this docs-only revision.
Object-region spacing stays BLOCKED until S-C0 lands; A-C1/C2-C4 Inner-wall,
Honeycomb/TPMS-D candidate omissions and multiline>1 expansion remain deferred.
