# Task 22O.77 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Port the first dependency slice of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s rectilinear family:
`src/libslic3r/Fill/FillRectilinear.cpp:357-496,759-993`.

The Rust destination is `fill::rectilinear::segments`. It builds the source
outer/inner offset contour inventory for one ExPolygon, creates equally spaced
vertical lines, computes contour intersections with source rational arithmetic,
sorts and classifies them as outer/inner low/high, removes source duplicate
vertex intersections, validates the source alternating shape, and returns owned
ordered sections. It is a private prerequisite for Monotonic and MonotonicLine;
it is not itself a filler or lifecycle stage.

## Included behavior

- optional fixed-coordinate rotation with source rounding;
- Miter-3 outer and inner offsets from the original ExPolygon;
- contour-before-hole and outer-before-inner order;
- source integer vertical-line bounds and exact rational intersection position;
- strict vertical-segment ignore and contour-touch filtering;
- rational sort, low/high classification from source segment direction, and
  duplicate endpoint elimination;
- direct Clipper/range errors, determinism, and input immutability.

## Deferred behavior

`FillRectilinear.cpp:994-2738` link construction, graph traversal, monotonic
regions and chaining; `2751-2924` complete line filling; public Rectilinear,
Monotonic, and MonotonicLine dispatch; entity conversion; adjusted solid Flow;
`Layer::make_fills` activation; motion and G-code are later slices.

The implementation is safe, platform-neutral, filesystem-free, source-cited,
and contains no legacy `infills` fallback or fixture branch. Three focused tests
pass for rectangle order/kinds, hole plus outer/inner identities, rational
rounding, rotation, range error, repeatability, and immutability. Strict core
all-target/all-feature Clippy, rustfmt, and diff checks pass. The production
shard is 220 LOC and the test file 121 LOC; all are ordinary modules.
