# ARD: source-owned classic extra-overhang perimeters

Non-negotiable owner: `PerimeterGenerator.cpp:677-1109`, classic integration:1672.
The Rust destination is the existing classic infill-boundary seam plus a small
`extra_perimeters` module split by generation/ordering/integration (each<400LOC).
The inactive guard is temporary scaffolding, not the intended feature contract.
It can be removed only with real upstream behavior wired through materialization.

Preserve source geometry and path ordering. Extra paths precede existing island
perimeters; covered fill is subtracted from accumulated fill surfaces; no-overlap
fill geometry stays as the original source computes it. Keep the existing
classic stage ownership and move semantics, not a parallel Ares pipeline.

The local BridgeDetector.hpp:122-126 adapter is in scope;
shared `geometry::detect_bridging_direction` is read-only
for this slice. Reuse geometry helpers and signed lower-contour distance semantics.
Use classic-prelude cached expanded lower polygons, not raw slices (cpp:1177).
Move the existing convex-hull helper unchanged to shared geometry rather than
copying it or creating a perimeter→G-code dependency. Preserve existing tests.
Port integer/float conversions and ordering explicitly; do not repair source
quirks speculatively. Arachne integration is outside the current classic slice.

Public complete3MF→G-code comparison is the regression boundary. Active extra-wall
geometry and disabled controls must exist before production edits. Frozen stock
pairs stay read-only. Keep all failed runs; no filtered timing acceptance, guard
bypass or post-hoc deleted assertions. The
Qidi prerequisite integrated as e865eb2a before implementation resumed.
