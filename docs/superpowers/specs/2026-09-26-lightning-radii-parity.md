# Lightning generator radii and surface dispatch slice

## Boundary

Upstream `Fill/Lightning/Generator.cpp::Generator` (77, 92–97) retains
`scaled<double>(layer_height)` until multiplying by the angle tangent and
casting each radius to `coord_t`. Rust destination:
`fill/lightning/generator.rs::Generator::new`.

Replace the prematurely truncated layer-height intermediate. Keep the
existing Rust tree generator as the upstream compatibility shell; do not
introduce a new branching strategy. Included: wall-support, pruning and
straightening radii. Deferred: other generator/tree/rasterization differences,
random child ordering, general option inventory and printer coverage.

A second included upstream boundary is `FillBase.cpp::Fill::fill_surface`
(105–118) before `FillLightning.cpp::_fill_surface_single`: offset each
surface by overlap minus half spacing before tree clipping and connection.
Apply at both Rust emission (`fill_entities/lightning.rs`) and bridge
anchoring (`sparse_anchoring/arms_tail.rs`) boundaries, handling every domain
returned by the offset without changing the generator's internal outlines.

## Diagnosis

Current strict lightning output is DIVERGENT, missing sparse paths
from Z1.2. A real upstream probe shows radius=3000000, prune=200000 and three
roots at layer42 with a surviving branch at layer5. The Ares debug trace
has prune/wall/straightening=199999 and four roots,
with different initial sampled X coordinates. The scaled-double fraction
in `0.2 / 1e-6` is significant when multiplying by tan(45°) before truncation.

## Plan

1. Add a separate constructor regression asserting all three scaled radii
   at 0.2 mm/45°; record failure at 199999 vs 200000.
2. Preserve scaled double, verify radii and repeat actual lightning oracle
   comparison without changing output tolerance or tree heuristics.
3. Run workspace/KSR and fmt/clippy. Record any remaining divergence
   instead of declaring full-domain parity.
4. Commit code and this spec/plan together.

The radius-only change restores the low-layer branch but does not yet pass
strict output: the shortened tree endpoint
is X114.056 instead of being clipped to X114.020 and connected to its contour
anchor. The missing FillBase offset explains this second discrepancy.
A red test reproduces raw X4055922 vs 4019693 at the real
append seam, using the upstream layer42 outlines propagated to layer5.
Also verify the resulting contour anchor, not only the output bound.

## Scoped verification and residual

10 focused tests pass, including constructor radii, fractional
heights/large-bed scale, and real append clipping/anchor. No extra source
instrumentation remains in core.

The live comparison is still DIVERGENT after three strict attempts. The
closest paired output restores the formerly missing low-layer paths and their contour anchors;
its remaining geometric mismatch starts with a junction path split near
Z5.8 and later branch-order differences, plus M73 placement. The source
`TreeNode::convertToPolylines` uses process-global rand during parallel
layer filling; this is a candidate, not established explanation for this
remaining mismatch. Do not report lightning as a passing option domain.
Further diagnosis must distinguish random traversal order from deterministic
junction trimming/connection errors; this scoped correction is not full
Lightning parity or whole-goal acceptance.

## Sparse-anchoring connection correction

The sparse-anchoring branch still appended raw
clipped lines instead of `FillLightning.cpp:25`'s `chain_or_connect_infill`.
This pre-existing omission becomes observable once endpoints are clipped
to the corrected contour. Add the same nonempty guard, connection parameters
and offset-domain connection as emission. A new regression calls the actual
`dispatch_tail` with a real prepared traversal and shows the missing three-point contour anchor before the fix. Flatten the
absent-generator branch to remove the newly introduced excessive-nesting
warning; prior clippy warnings were therefore not all pre-existing.
The pattern-specific body is extracted into `arms_tail/lightning.rs` so
its nonempty guard does not reintroduce excessive nesting in the match arm.

11 focused tests pass; no excessive-nesting warning remains in the changed
sparse-anchoring files. The live comparison still reports DIVERGENT, as expected
for the unresolved branch/junction and progress residual. The tests do not
reinterpret that failure as parity success.

## Junction endpoint conversion follow-up

Source: `TreeNode.cpp::Node::removeJunctionOverlap:397–400` casts the
scaled displacement to `coord_t` before adding it to endpoint `a`.
Destination: `fill/lightning/tree_geom.rs::Node::remove_junction_overlap`.
Replace rounding the final floating coordinate with componentwise-truncated
displacement plus integer endpoint. This is a deterministic arithmetic
correction, not a random-order workaround or a new tree heuristic.

Plan: red tests on positive/negative endpoints, signed overlap reduction,
and reduction carried across a consumed segment; apply source-equivalent
conversion; run focused/workspace/KSR/live oracle and commit docs with code.
Complete-line removal and random traversal policy are unchanged. Live option
acceptance must still require strict equality.

Evidence: both regressions fail before the fix (e.g. (9,9) versus the
required (10,10)); 13 focused tests pass after it. The live strict
comparison remains DIVERGENT; this correction alone does not establish whole Lightning parity.
