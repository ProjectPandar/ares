# Turning infill bridge boundary sampling

## Source boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`PrintObject::bridge_over_infill` in `src/libslic3r/PrintObject.cpp`:

- Lines 3865–3867 classify the candidate region as turning for Hilbert Curve
  or Octagram Spiral. This classification is independent of whether the
  same-region lower-layer angle can be selected (3913–3942).
- Lines 3552–3749 and 3959–3991 construct the bridge boundary from real
  anchors. A turning pattern samples at `max(1, scaled_spacing / 4)`; other
  patterns advance by one extrusion spacing. Initial construction and
  collision reconstruction use the same step. Section centers and trace
  edge offsets depend on the step; the scan's longitudinal range and
  anchoring extension still depend on physical flow spacing and width.
- Lines 3994–4012 restore contact after opening and closing, before
  clipping. For a nonempty turning bridge, reconstruct the cleaned area
  at the chosen angle against the same boundaries and union the result
  with the cleaned area. In this pass, the anchor search includes one
  scaled flow width plus epsilon of existing overlap (3620–3641), so it
  does not skip an already contacted anchor in favor of the next cell.
  Do not open or close the repaired area again.

## Rust destination

Extend the existing `project_slice::prepare_infill::bridge_over_infill`
rewrite, which divides the upstream candidate loop and reconstruction
lambda into the following owners:

- `transaction/candidate_expansion/layers.rs`: derive the sampling policy
  from the candidate region, not the lower-region angle result.
- `candidate_anchored_bridge.rs` and `candidate_collision_reconstruction.rs`:
  apply that policy to the initial and collision reconstruction calls.
- `anchored_polygon.rs` and its `sections` and `tracing` submodules: keep
  physical flow and sampling arithmetic distinct; use the overlap
  allowance only during restoration.
- `candidate_bridge_postprocessing.rs`: restore contact between closing
  and clipping, retaining the selected angle and physical boundaries.

Non-turning candidates must retain their original calculation and operation
sequence. Bridge flow, extrusion, angle selection, sparse anchor generation,
and the surface rewrite at lines 4036–4096 are unchanged. In particular,
the upper bridge's ensuring ring already affects the lower layer's internal
solid and sparse areas; no changes to that propagation are needed. Ordinary
PlanePath bounds, path generation and connection remain outside this slice.

## Verification

Slice a synthetic stepped prism on the MK4 with the stock sparse pattern,
then with only that setting changed to Hilbert Curve or Octagram Spiral.
The recess activates an internal bridge and ensuring solid below it. Both
the public API and CLI must match the complete Native output using the
existing generator normalization; the stock case checks the unchanged
non-turning route. Run all `orca_parity` cases in both reference modes on
both revisions using identical settings, and compare outcomes by full
case identifier.
