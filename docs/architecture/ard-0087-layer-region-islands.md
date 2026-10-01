# ARD0087 — Layer-region aggregate islands

Status: isolated prerequisite verified; final review pending, not modifier
output acceptance.

## Owner and destination

Rewrite OrcaSlicer `src/libslic3r/Layer.cpp:34-67::Layer::make_slices`
and `ClipperUtils.hpp:364::union_safety_offset_ex` into
`ares-core/src/project_slice/slice_ordering.rs::make_slices`.

## Decisions

- Aggregate every region's contours and holes with the native 10-scaled-unit
  safety-offset union, then chain contour first points into layer islands.
- Preserve region surfaces, metadata and identity. Islands do not replace
  region-owned options, perimeters or fills.
- Return geometry errors through the existing compensation error path.
- Keep the observable single-region surface ordering of the current adapter
  until region-owned perimeter inputs replace this temporary compatibility
  shell. Native make_slices itself does not reorder region surfaces.
- This prerequisite may merge independently only after clean-HEAD regression
  verification and six-axis review. It must not include unmerged Normal
  support fields, scheduling or new rejection guards.
- Normal support remains unmerged until Tree auto/manual/style output parity
  is complete. Existing modifier/Tree limitations are not acceptance outcomes.

## Included and deferred

Included: aggregate islands, contour/holes ordering, retained empty layers,
minimal caller/error propagation and supplemental geometry behavior tests.

Deferred to required follow-on slices: PrintObjectSlice.cpp multi-region
compensation, Layer.cpp compatible-region perimeter generation, Fill.cpp
region-owned fills, SupportMaterial.cpp all-region traversal, and the named
TreeSupport/TreeSupport3D/TreeModelVolumes rewrite. Their full public output
gates remain open; this slice does not enable modifier project slicing.

Execution evidence belongs in the plan, not this architecture record.
