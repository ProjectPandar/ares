# Order regenerated Arachne top-wall paths by depth from the outer wall

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/PerimeterGenerator.cpp:2857–2869` in `process_arachne`, calls
`renumber_inner_walls_by_depth` after offsetting the regenerated inner path
inset indices and before merging them into the outer groups. This applies to
an uncovered top with the non-clipping regeneration path and exactly one
outer inset group. The two helpers at lines 2357–2470 determine contact and
assign depth. A wall alongside a top boundary can be generated at inset one
without touching any real outer wall. If it touches a wall already anchored
to the outside, its effective depth can be greater than one even with only
two configured walls. The following inner-to-outer traversal uses that depth
for its seed and adjacent-inset dependencies without changing wall geometry.

## Rust destination and invariants

Wire the helper into `project_slice/perimeters/arachne/top_surface.rs::regenerate`
with a private module for the contact and depth passes. This replaces the
existing increment-and-merge compatibility shell only at the merge point:

- Flatten nonempty regenerated lines in their existing order. Inflate each
  bounding box by its widest bead. Use symmetric junction-to-segment contact,
  with clamped projection and linearly interpolated segment width. Use the
  greater of the configured threshold and the local half-sum of bead widths.
- Anchor original inset-one lines to the outer paths. The threshold is
  `ext_spacing + inner_spacing / 2` with precise outer walls and half the sum
  of the spacings otherwise. The inner threshold is `inner_spacing`. Add
  the source's scaled 0.001 mm allowance to each.
- Walk the original successive insets from the outer frontier. Then grow
  from all anchored lines, assigning each unnumbered contact one greater
  than its deepest contact in the previous frontier. Keep the original
  inset for lines not reached. Regroup in original flattened order, without
  capping the effective depth to the configured wall count or altering
  junctions, flags, or infill contours.

The existing top-area and widened-bead replacement remain in place. This
slice does not port positive-top-expansion clipping, change candidate sort
or region constraints, or change seams, travel heuristics, or G-code timing.

## Public behavior

Slice a synthetic coupon whose top terminates beside continuing walls. Its
regenerated inner paths include one that touches an anchored inner wall but
not the outer wall. Compare the complete emitted output with Native through
the public slicing API and CLI. The enabled input exercises deeper-path
ordering; the same input with `only_one_wall_top` disabled must still match
Native. Preserve complete ordered-byte equality, including progress lines.
The retained seeded Arachne project must also match complete Native output.
