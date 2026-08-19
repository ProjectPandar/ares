# Plan: KSR FDM Test V4 task224 rectilinear offset quantization

1. Replace the obsolete fractional-offset pinning assertion with a failing focused test requiring the integral scaled coordinates produced by Orca's `float`-to-`coord_t` constructor conversion.
2. Quantize each checked rectilinear offset after its source-order `f32` conversion, before either outer or inner polygon offset executes.
3. Probe Orca's `ExPolygonWithOffset` and monotonic-fill path with the KSR surface, then compare the regenerated Ares output against both O223 and the reference.
4. Record that the corrected offset seam exposes an earlier adjacent surface-geometry difference rather than falsely claiming the reference arc is fixed; update the roadmap with the next divergence and structural counts.
5. Run formatting, focused nextest, focused Clippy, and file-size checks; commit and push this source-cited correction independently.
