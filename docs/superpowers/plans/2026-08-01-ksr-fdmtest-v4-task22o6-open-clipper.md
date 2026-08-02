# Task 22O.6 Exact Open-Path Clipper Plan

Source pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

1. Add `geometry::Polyline`, exact `Polygon::split_at_first_point`, and separate direct geometry tests.
2. Rename `ClosedClipper` to `Clipper` without an alias, add `has_open_paths`, open input errors/APIs, and make flat execution return an error when open inputs exist. Preserve closed behavior and do not touch `ordering.rs`.
3. Port `deps_src/clipper/clipper.cpp:756-949` open `AddPath`/`AddPathInternal` behavior exactly: validation order, two-point acceptance, duplicate rules, open collinear retention, skipped terminal edge, flat minima, non-flat minima/bounds/LML, and zero wind deltas.
4. Audit and port every output-affecting open/`IsOpen` branch from the fixed Clipper v6 execution paths around `1137-1152`, `1992-2017`, `2218+`, `2800-2860`, joins/fixups/strict-simple/orientation, and PolyTree construction/extraction around `4119-4179`, plus matching `clipper.hpp`. Keep active full-range cross products as `f64` and preserve casts/order.
5. Change PolyTree contours to a typed open-polyline/closed-polygon representation. Open records are roots; closed parent/hole topology and `into_expolygons` remain exact and do not reinterpret open output.
6. Port `ClipperUtils.cpp:835-934` `_clipper_pl_open`, `_clipper_pl_recombine`, `_clipper_pl_closed`, and only polygon `intersection_pl`/`diff_pl`, with NonZero fills, exact closure duplication, extraction order, nested loop order, four branch priority, erase, and retry.
7. Add separate semantic fixtures for all specified open input, scanline, horizontal, output, PolyTree, large-coordinate, wrapper, and recombination cases; retain all inherited closed suites. Keep every source/test Rust file below 400 lines.
8. Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` with the exact O6 boundary and O7 deferral. Do not alter O5 lifecycle except mechanical `Clipper` imports.
9. Validate focused open and inherited closed suites, core/workspace checks, Clippy, rustfmt, WASM where available, file LOC, forbidden patterns, untouched `ordering.rs`, and a static comparison against every fixed upstream open branch.

This plan stops before `PerimeterGenerator.cpp:153-228` materialization. It adds no extrusion types, path ordering, lifecycle advancement, placeholder traversal output, writer, G-code, dependency, FFI, filesystem/process access, or runtime Orca oracle.
