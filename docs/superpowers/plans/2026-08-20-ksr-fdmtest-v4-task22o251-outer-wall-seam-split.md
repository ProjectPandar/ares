# Plan: Task 22O.251 outer-wall aligned seam split

1. Extend the focused KSR first-layer extrusion test with the OrcaSlicer outer-wall travel and short first extrusion sequence; run it red.
2. Trace the aligned seam position, scaled point, closest segment, snap decision, and fitted split in Ares and OrcaSlicer.
3. Correct the smallest source-derived arithmetic or split-order mismatch without fixture-specific branches.
4. Run the focused seam and spline tests, rustfmt, and clippy; commit and push the slice.
5. Regenerate KSR G-code and record the next normalized executable-body divergence.
