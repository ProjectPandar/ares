# Task 22O.11 implementation plan

1. Pin Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` and primary boundary `PerimeterGenerator.cpp:1573-1581,1583-1585`, stopping before line 1586.
2. Reuse the internal Clipper v6 `opening_ex`, `offset2_ex`, and ordinary `difference_ex` seams and the source-shaped Polygon/ExPolygon Douglas–Peucker methods.
3. Add an aligned O11 successor that stages every fallible geometry result while borrowing O10/O5, then moves O10 appended collections and retains its boxed O5 pointer.
4. Wire preparation and the public incomplete sink through O11 without adding ThickPolyline or medial-axis placeholders.
5. Add independent direct, lifecycle, and in-memory KSR anchors for arithmetic, geometry, ordering, error behavior, reachability, and allocation/checksum preservation.
6. Record the source boundary in architecture and roadmap and run focused Nextest, workspace check/Clippy, WASM check, and rustfmt gates.

The exact implementation contract and included/deferred behavior are in the paired Task 22O.11 spec. The next source boundary begins at `PerimeterGenerator.cpp:1586` with `ExPolygon::medial_axis` and its actual ThickPolyline prerequisites.
