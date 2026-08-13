# Task 22O.77 — rectilinear vertical segmentation

Port pinned `FillRectilinear.cpp:357-496,759-993` into private
`fill::rectilinear::segments`.

The seam accepts one ExPolygon, rotation angle, two source offset deltas,
vertical-line count/origin/spacing, and coordinate scale. It returns ordered
vertical sections containing exact source-rounded points and outer/inner
low/high classifications plus contour/segment identity needed by later link
construction.

Preserve contour order, segment order, source rational numerator/denominator,
endpoint/tangent filtering, rational comparison, duplicate endpoint rules,
low/high direction classification, and validation. Do not replace source
rational arithmetic with floating intersections or reuse the old top-level
`infills` scanline scaffold.

TDD tests cover rectangle, hole, vertex touch, diagonal rational rounding,
rotation, outer+inner offsets, duplicate elimination, empty result, range error,
repeatability, and input immutability. Tests live in separate modules and every
Rust file stays below 400 LOC without source-splitting macros.

Deferred: links, traversal, monotonic regions, complete fillers, entities,
lifecycle, motion, and G-code.
