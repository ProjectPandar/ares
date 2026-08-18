# Plan: KSR FDM Test V4 task218 clipped-arc sweep containment

1. Add a retained-arc clipping test with a candidate outside the original counter-clockwise sweep; confirm Ares incorrectly expands the arc.
2. Port `ArcSegment::is_point_inside` directed angular containment and clear retained arc data when containment fails.
3. Run focused arc tests and regenerate the complete KSR fixture output; record normalized first divergence plus line, arc, and wipe counts.
4. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
