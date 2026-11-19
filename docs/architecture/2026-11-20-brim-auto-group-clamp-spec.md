# Auto group and combined outer brim widths

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `libslic3r/Brim.cpp:493–505,549–607`, object area generation. The truncated `Flow::scaled_spacing()` coordinate supplies both double millimeter spacing and float scaled spacing. The real-number and scaled width steps are not interchangeable.

- Auto: compute the existing group heuristic from the original group slices. At line 550, floor the double millimeter width to an even number of spacings, scale in double, and narrow directly to float. Do the same conversion for the 5 mm additional width (line 499). After optional EFC intersection, test each resulting ExPolygon's signed area divided by its contour length in scaled units. Add the float additional width only when the group is taller than 10 mm, its float width is below both 5 scaled mm and one float spacing, and the ratio is below the float scaled 1.1 mm threshold. Limit that float result to the float cast of the largest integer contour bbox side (line 574, not the 1.5× suggested by the preceding comment). Apply the outer-width floor once: two float divisions, then double floor and products, and narrow to the float offset (line 576).
- Explicit combined: the contour takes the same first float width and second floor as OuterOnly (lines 497, 505, 576 and 592). The hole inset is independent of this second floor (lines 596–607); this change preserves its existing width input.

## Rust boundary and exclusions

Replace the millimeter-space approximation in `project_slice/gcode_emit/brim/auto.rs` with the scaled float pipeline. Share the outer-width conversion and floor at the object brim call sites in `project_slice/gcode_emit/brim.rs` and pass the separate contour/hole offsets to `brims/combined.rs`. Preserve the group heuristic, assignments, EFC outline selection, gap, hole widths, ears, support/raft/tree, masks and brim path assembly.

## Public contract

Slice synthetic 3MF projects through Native and the public Rust and CLI entry points. Check actual Native extrusion to distinguish an automatic addition from a zero-width control, a bbox-limited outer extent from the uncapped group width, and an explicit combined contour from its in-hole extrusion. Compare whole ordered G-code, including configuration, with only generator identity/time normalization. Run official and seeded Native modes and compare identical brim test inventories before and after the code change.
