# Plan: KSR FDM Test V4 task220 rectilinear offset float scaling

1. Add a numeric regression using fractional scaled offsets and confirm Ares cannot preserve the source `f32` values because it converts through integer coordinates.
2. Introduce a checked scaled-float conversion at the monotonic surface seam and use it for both source offset formulas.
3. Update the rectangle behavior expectation for the corrected endpoint; run focused surface tests.
4. Regenerate the complete KSR fixture output and record normalized first divergence plus line, arc, and wipe counts.
5. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
