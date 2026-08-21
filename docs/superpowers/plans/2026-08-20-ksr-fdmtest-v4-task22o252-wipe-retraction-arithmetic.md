# Plan: Task 22O.252 proportional wipe retraction arithmetic

1. Extend the focused KSR first-layer motion assertion with the three exact OrcaSlicer wipe lines; run it red.
2. Match `Wipe::wipe` multiplication and division grouping, source-coordinate path lengths, and retained fitted-circle radius; remove the non-source per-segment remainder cap.
3. Run the focused motion and travel tests, rustfmt, and clippy; commit and push the slice.
4. Regenerate KSR G-code and record the next normalized executable-body divergence.
