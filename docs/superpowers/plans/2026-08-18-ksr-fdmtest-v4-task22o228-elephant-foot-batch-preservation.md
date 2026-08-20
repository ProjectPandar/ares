# Plan: KSR FDM Test V4 task228 elephant-foot batch preservation

1. Replace the obsolete union-pinning batch test with a failing regression requiring tiny and fallback contours to remain byte-for-byte identical and in input order.
2. Remove the post-compensation union and return the collected per-ExPolygon results directly.
3. Update focused fallback expectations that encoded union-only contour rotation while retaining behavior tests for compensation, errors, holes, and ordering.
4. Run focused elephant-foot tests and regenerate the complete KSR fixture; record the next normalized divergence plus line, arc, and wipe counts.
5. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
