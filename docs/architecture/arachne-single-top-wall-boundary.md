# Preserve Arachne outer walls outside the single-wall top region

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/PerimeterGenerator.cpp::process_arachne`, lines 2754–2835,
and its `widest_bead`, `length_off_reference`, and `walls_footprint` helpers
(lines 767–818). The single-inset solver can use two wide beads where the
ordinary inset count would generate an additional middle bead. These wide
outer paths belong on the exposed top, not on an adjacent wall that continues
upward. Leaving them in place can remove room for an inner wall and change
both wall roles' physical paths before ordering or extrusion.

## Rust destination

In `project_slice/perimeters/arachne/top_surface.rs`, share the contour-based
top-area calculation between the nominal and actual fill contours. If the
widest single-pass bead exceeds the outer spacing plus twice its one-tenth
spacing tolerance, test for uncovered nominal interior. Lazily generate the
ordinary wall count, using the same input as the single pass. Measure the
single-pass centerlines outside both the tolerance around the reference outer
walls and the expanded top-area exclusion. If that length exceeds the
internal perimeter width, keep the ordinary outer walls and subtract their
local-width footprint from the nominal interior with the upstream safety
offset. Continue with the actual top-area calculation and existing inner-wall
regeneration. Reuse an already generated ordinary result if the actual
interior is fully covered.

The existing shell-layer checks, single-inset invocation and inner-wall
regeneration remain the surrounding port. The separate positive-top-expansion
clip path and the inset depth renumbering have their own geometry and ordering
conditions; they are not part of the outer-wall replacement. Classic walls,
wall completion direction, and G-code scheduling are unchanged.

## Public behavior

Slice a compound synthetic coupon (rings, bridge, sloped projections, tapered
ribs and pads) with Flashforge Guider 2s stock and Prusa MK4 with
`only_one_wall_top` enabled. Confirm the effective Native settings and that
Native's wall segments at the affected height change relative to the same
project with the option off. Guider with the option off is also a parity
control. Compare complete emitted G-code using the existing ordered-byte
comparator against both Native producers.
