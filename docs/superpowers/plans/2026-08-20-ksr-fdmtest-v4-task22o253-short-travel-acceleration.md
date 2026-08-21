# Plan: Task 22O.253 short outer-wall travel acceleration

1. Add a focused generated-output assertion for the exact Z0.4 acceleration/travel order; run it red.
2. Pass destination role and generated travel length into travel acceleration selection, matching OrcaSlicer's first-layer, short outer-wall, short overhang, and default branches.
3. Run the focused motion test, rustfmt, and clippy; commit and push the slice.
4. Regenerate KSR G-code and record the next normalized executable-body divergence.
