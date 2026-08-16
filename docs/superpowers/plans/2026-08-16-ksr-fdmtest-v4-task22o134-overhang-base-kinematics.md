# Plan: Task 220.134 overhang-role base kinematics

1. Add a red KSR output assertion for the acceleration and feedrate immediately surrounding the first `Overhang wall` feature.
2. Map `Overhang wall` to the already option-resolved bridge acceleration and speed in the motion feature dispatcher.
3. Regenerate KSR output and verify the changed block against Orca, then run focused motion tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
