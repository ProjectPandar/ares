# Task 22O.10: Perimeter collection append

## Upstream boundary

This source-cited Rust rewrite is fixed to OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`. It owns `PerimeterGenerator.cpp:1451-1569`: conditional overhang reorientation, outer-first/layer-zero outer-brim reversal, sandwich wall reordering, and nonempty append into `this->loops`. The Rust destination is crate-private `project_slice::perimeters::classic::perimeter_append`.

## Reachable behavior

O1 transactionally rejects `overhang_reverse=true`, every `wall_sequence` except `InnerOuter`, and layer-zero `brim_type=OuterOnly` with positive `brim_width`. Therefore lines 1451-1565 are provably inactive for every state reaching O10. O10 retains typed operands and the exact false reason; it does not implement or represent an active fallback.

Lines 1567-1569 remain executable. Each nonempty O9 flat `ExtrusionEntityCollection` is moved as one nested perimeter collection; an empty collection is omitted. Source index, entity/path order, roles, `inset_idx`, numeric fields, point buffers, and the exact boxed O5 predecessor are preserved. This move is observably equivalent to the source clone because the local source collection is dead after append.

## Deferred behavior

Relaxing any O1 ordering/reorientation rejection requires a separate source-cited implementation of the corresponding upstream algorithm before an active O10 state may be accepted. Gap filling begins at `PerimeterGenerator.cpp:1573` and remains deferred with active thin walls, seams, infill, motion, G-code, writer/post-processing, complete Task 22O, and final byte parity. Public slicing executes O10 and remains `ProjectSlicingIncomplete`.
