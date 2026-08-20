# Plan: Task 22O237 aligned-seam source priority

1. Add a focused comparator test with candidates on different layers; prove cross-layer source scoring is unavailable through the current layer-local helper.
2. Implement the source comparator across layer plans and stable-sort chosen seams before seam-string traversal.
3. Run the focused test, KSR slice regression suite, formatting, Clippy, and workspace nextest; commit and push independently.
