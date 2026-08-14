# Task 22O.94 implementation plan

1. Add RED module/lifecycle and KSR island inventory tests.
2. Define owning layer island and infill-entity types.
3. Port bbox-area ordering and first-point assignment.
4. Advance public lifecycle transactionally and retain fallback island.
5. Run focused/dependency/strict gates, update evidence, commit/push.

## Completed evidence

KSR freezes 3,350 total / 2,881 nonempty / zero nonempty-fallback islands,
1,658 generated fills, 2,285 thin fills, and 2,881 perimeter collections. The
1,835 perimeter-only and 1,046 mixed-island split is deterministic. Three
focused tests and strict core Clippy, rustfmt, diff, and LOC gates pass.

No multi-region/tool behavior, traversal chaining, motion, fallback pipeline, or
G-code.
