# Plan: Task 141 arc-fitting extrusion-path simplification

1. Add a failing `slice_project` assertion for the KSR V4 inner-wall vertex sequence where Ares emits redundant points.
2. Port the `ArcFitter` fitted-range bookkeeping and per-range Douglas-Peucker pass into a dedicated `motion::arc::simplify` module.
3. Simplify option-driven extrusion points before dynamic overhang-speed estimation so processed paths cannot bypass source simplification.
4. Run the focused parity test, formatting, strict Clippy, and the relevant core suite; commit and push the vertical slice.
