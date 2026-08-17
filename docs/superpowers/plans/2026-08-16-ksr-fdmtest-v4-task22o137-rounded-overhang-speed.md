# Plan: Task 220.137 rounded dynamic-overhang base speed

1. Add a red KSR assertion for the rounded initial and precise restored feedrates on the first dynamically processed inner wall.
2. Round the fully supported speed branch at the same seam as interpolated and fully unsupported overhang speeds.
3. Run the focused assertion, motion tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
