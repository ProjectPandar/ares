# Plan: Task 22O.150 pre-fit aligned seam corner

1. Add a focused failing assertion for the second KSR spiral lift and outer-wall seam travel.
2. Preserve pre-fit perimeter points through path simplification without copying, and use them only for seam-candidate extraction and collection mapping.
3. Keep the nearest acceptable sibling feature, retain its best-scoring corner within one flow width, and discard transient candidate buffers after seam placement.
4. Run focused seam-placement, retained-arc, and KSR travel contracts; generate the CLI slice and identify the next exact divergence.
5. Run rustfmt and Clippy, record the milestone, then commit and push independently.
