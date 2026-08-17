# Plan: Task 22O.155 near-vertex seam snapping

1. Add a failing loop test whose selected seam lies within the upstream 1.5-micron radius of a vertex.
2. Add failing KSR assertions for the single exact wipe move and absence of its artificial preliminary segment.
3. Search loop vertices in path order before projection; reuse the first vertex inside the scale-adjusted radius.
4. Preserve projected split behavior when no source vertex qualifies.
5. Run focused seam and KSR contracts, generate the CLI slice, and identify the next exact divergence.
6. Run rustfmt and Clippy, record the roadmap milestone, then commit and push independently.
