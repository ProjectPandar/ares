# Plan: Task 22O.254 precise seam and variable-loop wipe state

1. Extend the focused later-layer motion assertion with the three exact OrcaSlicer wipe moves; run it red.
2. Derive seam candidate Z from f64 planned-layer accumulation and cast once at the source `Layer::slice_z` seam.
3. Retain variable-speed wipe geometry and its processed source-precision endpoint separately from formatted writer XY state.
4. Run the focused motion test, rustfmt, and clippy; commit and push the slice.
5. Regenerate KSR G-code and record the next normalized executable-body divergence.
