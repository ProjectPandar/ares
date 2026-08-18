# Plan: KSR FDM Test V4 task221 rectilinear rotation half-up rounding

1. Add a failing numeric regression covering positive and negative half ties plus OrcaSlicer's predecessor-of-0.5 exception.
2. Add one shared `fast_round_up` primitive and replace default rounding at forward contour rotation and reverse emitted-polyline rotation.
3. Run the focused rectilinear suite and regenerate the complete KSR fixture output; record normalized first divergence plus line, arc, and wipe counts.
4. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
