# Arachne inner–outer–inner proximity ordering

## Source boundary

Port OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/PerimeterGenerator.cpp`, into Ares
`crates/ares-core/src/project_slice/perimeters/arachne/sandwich.rs`. The
caller in `materialize.rs` retains the topological ordering, first-layer
and minimum-wall-count conditions, and flow-spacing thresholds
(`:2999–3029`; precise outer wall applies to Inner/Outer only, `:2718`).

| OrcaSlicer lines | Ares rewrite |
| --- | --- |
| `:2634–2638` | Retain the stable partition that moves inset-0 contours to the front before the proximity pass. |
| `:2357–2377`, `:2486–2508` | Replace nominal-only, rounded polyline distance with the symmetric junction-to-segment predicate. Clamp the projection, interpolate the segment bead width locally, and use the greater of the reference spacing threshold and half the combined bead width. Include equality and give a degenerate segment projection parameter zero. Search only the requested inner inset. |
| `:2523–2545` | Starting from each unassigned inset-0 seed, discover inset-1 neighbors and admit unassigned inset-0 walls touching those neighbors. Visit the added outers too, so the group can grow transitively. |
| `:2546–2606` | Stably move closed holes ahead of the other group outers; append all outers, then inset-1 with longest last, and walk one deeper inset at a time with longest first. Each line is emitted at most once. |
| `:2608–2623` | Visit seeds in input order and append untouched lines in input order. |
| `:3035–3092` | Keep the sandwich scan downstream of grouping: reverse the deeper part of a qualifying block, then emit its outer and first-inner part in proximity order. Advance even when a block has no second-inner wall. |

## Scope and exclusions

The existing sandwich adapter is only the call boundary for the upstream
ordering pass, not a separate wall planner. Do not change toolpath generation,
seam placement, wall direction, overhang traversal (`:3096–3108`), other
wall sequences, or G-code scheduling. The separate single-top-wall generation
and its depth renumbering (`:2387–2474`, `:2754–2873`) are outside this port.
Platform-specific `unordered_set` bucket iteration beyond the observed
adjacent-wall order is not a portable upstream contract; do not reproduce
allocator or undefined behavior.

## Observable contract

Compare complete ordered G-code from the public slicer CLI against both
Official and Seeded OrcaSlicer, normalizing only the generator line. Use a
synthetic MK4 plate with holes and narrow bridges between wall runs. Check
Inner/Outer and Outer/Inner on the same geometry as controls. This exercises
wall entry order as well as its downstream effects on time estimates, without
masking either.
