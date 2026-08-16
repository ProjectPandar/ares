# Plan: Task 22O.127 forward stored-path wipe

1. Add a focused `wipe_moves` test with a clipped current endpoint and a closed stored path; assert Orca's current-to-second-point forward traversal and run it red.
2. Replace reverse-window traversal with forward iteration over the stored path after its first point, preserving distance clipping and zero-length handling.
3. Run focused travel tests, regenerate KSR output and inspect the first outer-loop wipe, then run rustfmt, strict `ares-core` Clippy, and LOC checks; commit and push this isolated source-cited slice.
