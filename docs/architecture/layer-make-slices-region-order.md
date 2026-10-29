# Layer::make_slices region ownership

## Upstream boundary

This is the single-region rewrite slice of OrcaSlicer `Layer.cpp:38-67`,
`Layer::make_slices`, owned by `ares-core::project_slice::slice_ordering`.
It is not a new Ares ordering policy.

## Decision

Region-owned surfaces retain their input ordering. Physical-layer `lslices`
are a separately chained geometry copy. Upstream copies the region's
ExPolygons into a local vector, computes `chain_points` from contour first
points, and moves only that local geometry into `lslices`; it never reorders
`m_regions.front()->slices`.

The former Rust single-region compatibility shell reassigned ordered surfaces
back into the region. That is removed now that perimeter inputs are region-owned.
The existing `order_expolygons` operation chains the copied geometry without
changing source geometry, surface metadata, or region ordering.

## Included and unchanged behavior

- Copy each single-region ExPolygon once, matching upstream's local copy.
- Keep the existing contour-first-point chaining and physical-layer output.
- Leave zero-region and multi-region behavior unchanged.
- Do not change perimeter, vertical-shell, Clipper, Voronoi, or numeric behavior.

Region order is observable: `PerimeterGenerator.cpp:2128` visits islands in
region order, `Layer.cpp:225` preserves generated fill-domain order, and
`PrintObject.cpp:2141-2143,2210` consumes that order in vertical-shell projection.
The regression seam is the frozen X1C project's complete public API/CLI G-code
compared with qualified Native output, not an internal ordering snapshot.

Adjacent upstream behavior and unrelated ordering differences are outside this
slice. No configuration switch, compatibility fallback, or fixture-specific
sorting rule is introduced.
