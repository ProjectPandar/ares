# Plan: Task 22O.142 inner-path aligned seam projection

1. Add a failing KSR `slice_project` assertion for the exact first inner-wall start travel while retaining the exact neighboring outer-wall seam assertions.
2. Replace loop nesting-role classification in `seam_placement::place_loop` with the first extrusion path's `Perimeter` role, matching OrcaSlicer `ExtrusionLoop::role()` and `SeamPlacer::place_seam`.
3. Run focused seam-placement and motion tests, strict Clippy, rustfmt, then commit and push this source-cited slice.
