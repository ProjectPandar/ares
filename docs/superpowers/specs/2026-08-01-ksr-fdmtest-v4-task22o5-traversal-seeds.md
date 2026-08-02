# Task 22O.5 traversal-seed specification

## Upstream boundary

This milestone rewrites OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `src/libslic3r/PerimeterGenerator.cpp:100-151` and `PerimeterGeneratorLoop::is_internal_contour` at `2537-2547`, into `project_slice::perimeters::classic::traversal`. The prefix ends before executable `apply_fuzzy_skin` at line 154. The read-only predicate `detect_overhang_wall && layer_id > raft_layers` at line 158 is retained only as a pending disposition; neither successor path executes. `PerimeterGenerator.hpp:81` stores `layer_height` as `double`, so O5 retains `f64`; narrowing to `float` is deferred to the later `ExtrusionPath::height` / `extrusion_paths_append` boundary in `ExtrusionEntity.hpp:164,170,551+`.

## Included behavior

O5 builds a transactional sidecar over O4 roots only. It preserves root/child order, polygon, depth, contour, smaller-width, and children. Depth zero selects external-perimeter role; other depths select perimeter role. A contour with no immediate contour child is internal, contours otherwise default, and holes are hole loops. External smaller, external normal, and internal loops select respectively the smaller-external, external, and ordinary predecessor lower series and flows. Internal routing ignores the smaller flag. Seeds retain selected width as `f32` and `mm3_per_mm` as `f64` without recomputation or per-seed series clones.

Each record retains overhang flow, exact `f64` layer height, detect/layer/raft predicate provenance, pending clipping-versus-ordinary disposition, and inactive provenance for the source expression `overhang_reverse && layer_id % 2 == 1`. O1 rejects active overhang reversal and fuzzy skin. Construction and terminal consumption are iterative. O5 nests O4 and preserves optional record/surface alignment and O4 diagnostics without traversing diagnostics as seeds.

## Deferred behavior

O5 does not execute fuzzy mutation, steep-overhang detection, bounding-box support clipping, `intersection_pl`/`diff_pl`, path chaining, extrusion path/loop/entity construction, orientation, recursive entity emission or reordering, thin walls, active overhang reversal, wall ordering, gaps/fill, seams, infill, motion, writer, or G-code. O4 is an immutable compatibility predecessor. Public slicing executes O5 and remains `ProjectSlicingIncomplete`.

Production code must remain platform-neutral and use no filesystem, FFI/Orca runtime, reference G-code or exact-byte oracle, fixture identity branch, include macro, unsafe code, lint allowance, fallback, or new dependency.
