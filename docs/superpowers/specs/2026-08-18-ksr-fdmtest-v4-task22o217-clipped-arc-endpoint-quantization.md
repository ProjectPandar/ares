# Spec: KSR FDM Test V4 task217 clipped-arc endpoint quantization

## Observable contract

When loop clipping shortens a retained fitted G2/G3 segment, the projected endpoint is quantized on the active slicer coordinate grid before the arc length and emitted endpoint are updated. Projection normalizes the center-to-candidate vector, scales it by the fitted radius, converts each vector component to integer coordinates by truncation toward zero, and then adds the fitted integer-grid center. Normal and large-bed coordinate scales use their respective grid factors.

This keeps clipped arc endpoints, arc lengths, extrusion, and subsequent wipe paths derived from generated geometry and the project `enable_arc_fitting` option. A focused non-grid projection distinguishes the source integer-vector result from an unquantized floating endpoint.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Circle.hpp:24-27`, `Circle.cpp:230-236`, and `Polyline.cpp:52-90`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/arc/retained.rs::clip_end`; `gcode_emit/motion/path.rs` supplies the active `CoordinateScale`.

Included: retained fitted-arc endpoint projection and coordinate-grid conversion after end clipping. Deferred: remaining arc candidate/range differences, monotonic geometry, retraction/wipe count, timing/M73, and other normalized G-code divergences. No fixture-name, reference-G-code, or coordinate-specific production branch is introduced.
