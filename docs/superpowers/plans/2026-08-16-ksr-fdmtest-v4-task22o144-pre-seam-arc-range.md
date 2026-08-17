# Plan: Task 22O.144 pre-seam arc-range simplification

1. Add a focused KSR assertion for the first outer-wall fitted range boundary; verify it fails when the split path is independently simplified during emission.
2. Move option-gated path simplification before seam placement, port the source integer-coordinate circle and arc-slice acceptance checks, and emit fitted ranges without a second Douglas-Peucker pass.
3. Keep the new stage and test-only consumers in separate modules below 400 LOC; run the neighboring simplification, seam, clipping, and arc tests plus strict Clippy and rustfmt, then commit and push.
