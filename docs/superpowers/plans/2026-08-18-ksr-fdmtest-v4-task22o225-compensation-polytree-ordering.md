# Plan: KSR FDM Test V4 task22o225 compensation PolyTree ordering

1. Add a focused regression for deterministic outer-sibling and same-parent-hole ordering.
2. Normalize owned compensation inputs and union outputs at the `PrintObjectSlice.cpp:1274-1292` seam: order outer ExPolygons by descending contour area and holes by source PolyTree scan order.
3. Re-run the focused first-layer project test and regenerate KSR G-code; record the next normalized motion divergence and structural counts.
4. Run formatting, focused Clippy, and file-size checks; update the roadmap and commit/push this source-cited slice independently.
