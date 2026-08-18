# Plan: KSR FDM Test V4 task210 inclusive island bounds prefilter

1. Add a failing nested-island test with the entity point on the inner maximum-X boundary.
2. Make bounds containment inclusive on maxima while retaining smallest-area-first polygon confirmation.
3. Run extrusion-island/order tests and fixture gap-feature interleaving/count comparison.
4. Run line-count checks, formatting, and workspace Clippy.
5. Record the association-shell fix in `docs/roadmap.md`, commit, and push independently.
