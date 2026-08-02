# Task 22O.3: bounded Classic raw-onion iteration

## Fixed rewrite boundary

This milestone rewrites OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/PerimeterGenerator.cpp:1304-1387`, into
`ares-core::project_slice::perimeters::classic::onion`. The source boundary is
a loop-back continuation: Task 22O.2 has already completed depth zero and the
line 1380 top split; this stage applies the line 1381 termination gate and then
re-enters lines 1304-1371 for depths at least one. It stops before line 1388,
`// nest loops: holes first`.

Task 22O.2 exclusively owns the `i == 0` external offset and dynamic-top split.
Its production and tests are an immutable predecessor. Its post-split
`remaining` seeds `last`, while its unsplit normal and smaller-width first
offsets seed raw depth zero. This stage does not reimplement depth zero.

## Included behavior

* Lines 1304-1329 choose external-to-internal spacing at depth one and perimeter
  spacing thereafter, then execute `offset2_ex`.
* Lines 1330-1340 append ordered gap masks before termination by applying
  `diff_ex(offset(last, -0.5 * distance), offset(offsets, 0.5 * distance + 10))`.
* Lines 1341-1352 reduce the effective loop count and clear `last` on collapse,
  or discard a calculated shell when `i > loop_number`.
* Lines 1353-1369 retain raw contour/hole `ExPolygon` geometry by signed depth;
  depth zero also retains Task 22O.2 smaller-width geometry.
* Line 1371 replaces `last` after a retained shell.
* Lines 1381-1387 stop on disabled gap fill or zero after the typed sparse
  density is converted to the source `int` local, otherwise executing one final
  gap-only iteration.

The fixed-coordinate `-1`, `+1`, and `10` terms are not millimetres. C++ f64
promotion is preserved until each explicit f32 Clipper delta cast. Existing
`ClipperUtils`-compatible `offset2_ex`, `offset`, `diff_ex`, and returned append
ordering are used directly. Typed effective 3MF `Percent` density is validated,
then truncated toward zero into the fixed source's `int sparse_infill_density`
local, transactionally for the complete stage before predecessor ownership is
moved.

## Typed result

Each successor object nests its Task 22O.2 object. Aligned optional records and
source-ordered surfaces expose source index, initial and effective loop counts,
ordered raw shell depths, final `last`, and ordered gaps. Task 22O.2 top fills,
fill clips, outcomes, source surfaces, and configuration remain reachable only
through that nested predecessor.

## Deferred boundary

Hierarchy/nesting beginning at `PerimeterGenerator.cpp:1388`, loop traversal,
perimeter extrusion entities, overhang splitting/reversal, gap medial-axis
construction, fill remainder, thin walls, G-code, metadata, post-processing,
and exact KSR G-code parity are deferred. Task 22O.3 is neither complete Task
22O nor an independently designed Ares perimeter pipeline.

## Verification

Direct transparent geometry tests cover distance selection, exact cast points,
fixed-coordinate constants, gap-before-termination ordering, collapse,
gap-only iteration, depth-zero normal/smaller seeds, holes/disjoint polygons,
and deterministic ordering. Typed KSR mutations cover sparse density, gap-fill
enablement, and wall count; the public lifecycle executes this stage and still
returns `ProjectSlicingIncomplete`.
