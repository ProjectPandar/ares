# Plan: Task 22O240 internal-bridge spacing preservation

1. Extend focused KSR output assertions to reject adjusted internal-bridge width/height tags and require the resulting 519-tag processor count.
2. Forward the surface bridge classification into monotonic fill's `dont_adjust` parameter, preserving the existing non-bridge solid-fill behavior.
3. Run focused output tests, regenerate and compare KSR structure, then run formatting, Clippy, and workspace nextest; commit and push independently.
