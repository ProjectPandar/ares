# Plan: KSR FDM Test V4 task225 G28 processor motion

1. Add a failing processor test proving axis-specific G28 contributes distance and resets only the requested logical axis.
2. Rewrite G28 into the equivalent zero-target G1 command while preserving current processor positioning and feedrate state.
3. Run focused processor tests, Clippy, rustfmt, and the KSR slice smoke test; record the timing delta.
4. Commit and push this source-cited processor slice independently.
