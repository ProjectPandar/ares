# ARD0088 — Layer-region XY and elephant-foot compensation

Status: independent prerequisite under validation; not Normal/Tree acceptance.

## Source owner and destination

Port `PrintObjectSlice.cpp:1314-1365` (multi-region branch of
`PrintObject::slice`) and `Layer.cpp:119-138::Layer::merged` into
`ares-core/src/project_slice/compensation/multi_region.rs`.
Retain `_shrink_contour_holes` (`PrintObjectSlice.cpp:1512-1544`) in the existing
`compensation/xy.rs` geometry boundary. The existing single-region adapter
remains a temporary compatibility shell around the single-region source
branch, not a replacement Ares slicing algorithm.

## Non-negotiable decisions

- Positive growth operates on the merged outline, then assigns intersected
  grown surfaces in original region order, subtracting previously assigned
  regions. The first region wins overlap; region identities/options remain.
- Negative growth and elephant-foot compensation operate on the merged
  outline and clip the result back to every region. Do not independently
  shrink shared region boundaries or replace region geometry by plate islands.
- `Layer::merged` excludes regions having zero bottom/top shells, zero sparse
  infill and zero walls, matching the native subtractor-volume convention.
- Elephant-foot uses the first region's external-perimeter flow and retains
  uncompensated aggregate islands. It does not use another region's flow or
  union the raw elephant-foot trimming result before intersection.
- Preserve native f32 narrowing, scaled EPSILON and the special unscaled
  EPSILON substitution/undo when `Layer::merged` is called with zero offset.
- Closing epsilon initializes at the first multi-region negative-growth or
  elephant-foot branch, even if that branch has no elephant-foot compensation.
  Share that value across objects in one Rust project compensation invocation.
  This bounded job policy does not claim equivalence to native process-static
  persistence across jobs or parallel first arrival. Those domains stay open;
  do not change the official reference or select a favorable reference variant.
- The public project-bytes/complete-G-code seam remains the acceptance gate.
  Supplemental stage geometry examples do not close modifier output parity.

## Included and deferred

Included: source-owned positive priority, active-region outline merging,
negative/elephant-foot clipping, first-region flow, aggregate backup and job
epsilon initialization; retain existing single-region observable behavior.

Deferred: native cross-job/parallel first-arrival equivalence, region-owned
perimeter grouping and downstream fills, support-region iteration and Tree
support. Existing downstream single-region limitations remain unaccepted
rewrite debt, not successful support output. This prerequisite contains no
Normal `raw_slices` storage, support scheduling, support-type guard changes or
new unsupported-feature guards. The Normal batch still cannot merge until
Tree output parity is complete.
