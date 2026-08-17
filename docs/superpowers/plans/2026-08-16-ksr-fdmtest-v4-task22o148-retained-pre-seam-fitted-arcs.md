# Plan: Task 22O.148 retained pre-seam fitted arcs

1. Add a failing focused assertion for the first reference G2 after the newly matched travel.
2. Carry arc fitting ranges on materialized `Polyline3` values and populate them during the existing pre-seam simplification stage.
3. Port metadata reversal, seam split, inserted-point, and clip-end updates; consume retained ranges in the emitter.
4. Run the focused KSR contract plus arc, seam-placement, prior range-boundary, and lifted-travel tests.
5. Generate the CLI slice, identify the next exact divergence, run rustfmt and Clippy, then commit and push independently.
