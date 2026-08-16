# Plan: Task 220.133 contiguous extrusion feedrate state

1. Add a red KSR output assertion that the source-equivalent first inner-wall path emits one `G1 F3000` despite Ares materializing it as adjacent fragments.
2. Preserve the preceding extrusion feedrate across endpoint-contiguous fragments; emit a new feedrate after travel/retraction interruption or when resolved kinematics change.
3. Regenerate KSR output, confirm the first structural divergence advances and redundant feedrate count drops, then run focused tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
