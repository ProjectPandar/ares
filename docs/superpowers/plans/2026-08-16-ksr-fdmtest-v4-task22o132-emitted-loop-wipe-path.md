# Plan: Task 220.132 emitted-loop wipe path

1. Replace the stale raw-subpath wipe assertion with a red emitted-loop contract and add an exact first-KSR-wipe assertion.
2. Retain post-fitting emitted segment endpoints for each path, aggregate every subpath across an extrusion loop, and let the existing wipe traversal continue from the clipped loop end around its beginning.
3. Regenerate KSR output, verify the first wipe block and focused travel tests, then run rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the isolated slice.
