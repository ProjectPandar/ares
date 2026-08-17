# Plan: Task 22O.158 adjusted solid-fill flow

1. Add a failing KSR contract for the exact first bottom-surface line-width marker.
2. Return the actual adjusted scanline spacing from monotonic fill generation.
3. Port non-bridge and bridge `Flow::with_spacing` width and volume updates.
4. Apply the adjusted flow when converting monotonic polylines into extrusion paths.
5. Update focused monotonic tests for the deeper result interface.
6. Run focused fill and KSR motion contracts, rustfmt, and Clippy.
7. Record the roadmap milestone, commit, and push independently.
