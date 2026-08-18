# Plan: KSR FDM Test V4 task201 concentric loop finalization

1. Add failing focused tests for partial thick-polyline end clipping and concentric finalization order.
2. Port source point interpolation/truncation into `ThickPolyline::clip_end` without rewriting width payloads.
3. Reuse the retained KD endpoint-chain module for thick-polyline shortest traversal; rotate closed loops to the nearest source-origin point and discard invalid clipped paths.
4. Derive seam clipping from the typed region `seam_gap` option and nozzle diameter.
5. Run focused geometry/fill/shortest-path tests, fixture slicing smoke, line-count checks, formatting, and workspace Clippy.
6. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
