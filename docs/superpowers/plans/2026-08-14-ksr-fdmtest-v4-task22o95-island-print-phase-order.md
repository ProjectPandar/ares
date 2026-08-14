# Task 22O.95 implementation plan

1. Add RED phase-order and KSR inventory tests.
2. Define owning ordered-island entity output.
3. Port first-layer override and resolved `is_infill_first` dispatch.
4. Advance lifecycle and verify repeatability/disposal.
5. Run strict gates, update evidence, commit/push.

## Completed evidence

Focused coverage proves first-layer wall-first and both later-layer option
branches. KSR freezes 3,350 islands, 2,881 nonempty/perimeter-first islands,
and 2,881/1,658/2,285 perimeter/fill/thin inventories. Four tests and strict
core Clippy, rustfmt, diff, and LOC gates pass.

No infill chaining/reversal, multi-region/tool behavior, motion, fallback
pipeline, or G-code.
