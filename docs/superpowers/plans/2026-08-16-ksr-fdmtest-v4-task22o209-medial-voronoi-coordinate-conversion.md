# Plan: KSR FDM Test V4 task209 medial Voronoi coordinate conversion

1. Add a failing focused positive/negative fractional vertex conversion test.
2. Replace nearest rounding with source truncation toward zero at the shared medial validation conversion seam.
3. Run medial-axis, variable-width gap, and fixture slicing comparisons; measure gap-fill blocks/moves.
4. Run line-count checks, formatting, and workspace Clippy.
5. Record the source-cited result in `docs/roadmap.md`, commit, and push independently.
