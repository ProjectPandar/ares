# Plan: KSR FDM Test V4 task230 processor acceleration envelopes

1. Add a failing focused processor test requiring an `M204 T20000` update to retain the loaded 9000 mm/s² travel ceiling.
2. Pass the three typed machine acceleration envelopes through the project G-code processor seam.
3. Clamp legacy and modern M204 state updates exactly where OrcaSlicer's time machine clamps them.
4. Regenerate the complete KSR fixture and record header time, M73 placement, output counts, and the next normalized divergence.
5. Run focused processor and complete-slice tests, formatting, and focused Clippy; commit and push independently.
