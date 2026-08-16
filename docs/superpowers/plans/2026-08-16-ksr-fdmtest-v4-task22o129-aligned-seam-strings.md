# Plan: Task 220.129 aligned seam strings

1. Add a red KSR assertion for the exact first inner travel and outer seam-start move; if upstream visibility/context precision prevents exact coordinates in this slice, retain an explicit bounded coordinate assertion and defer the named residual.
2. Retain per-layer candidates and initial choices across the object, then port nearby-layer seam-string discovery and score tolerance.
3. Port weighted cubic B-spline fitting into a dedicated module and apply fitted positions through OrcaSlicer internal/outer loop projection.
4. Regenerate KSR output, verify the first outer seam travel and extrusion exactly and the preceding inner travel within 0.03 mm per axis, run focused tests, rustfmt, strict `ares-core` clippy, and LOC checks; record exact inner coordinate parity as deferred, then commit and push the isolated slice.
