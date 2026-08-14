# Task 22O.98 — Arachne extrusion-line primitives

Port pinned `Arachne/utils/ExtrusionJunction.hpp` and
`Arachne/utils/ExtrusionLine.hpp/.cpp:21-275` as crate-private Rust geometry.

Requirements:

- retain scaled point, width, perimeter, inset, odd-line, and closed metadata;
- preserve source mutation/reversal and per-segment integer length arithmetic;
- emit the exact doubled endpoint-width layout used by `ThickPolyline`;
- preserve Arachne clockwise-contour and signed-area conventions;
- port integer weighted-width area deviation and source simplification order;
- resolve the upstream five-micron constant through active `CoordinateScale`;
- use focused tests in a separate module and ordinary files below 400 LOC;
- do not activate this seam from project slicing or legacy toolpaths.

Deferred: C++ `ExtrusionPaths` adapters, free closed-line `to_polygon` and
`to_points` helpers, half-edge and skeletal topology, beading, `WallToolPaths`,
`FillConcentricInternal`, variable-width extrusion entities, runtime lifecycle,
motion, and G-code.
