# Arachne transition angle: preserve upstream float evaluation

## Upstream boundary

- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.hpp`,
  `WallToolPathsParams::wall_transition_angle`: stored as `float`.
- `Arachne/WallToolPaths.cpp:490`, `WallToolPaths::generate`: calls
  `Geometry::deg2rad(m_params.wall_transition_angle)` before widening to double.
- `Geometry.hpp:301-305`, `deg2rad<T>`: `T(PI) * angle / T(180.0)`.
  Here template deduction makes **T=float**, including both arithmetic operations.
- `Arachne/BeadingStrategy/BeadingStrategyFactory.cpp:19-31`: consumes a float angle.

## Decision

At the three perimeter call sites constructing `RawWallToolPathConfig`, narrow
Rust's degree parameter to f32, multiply by f32 PI, divide by f32 180, then widen
the result to the existing f64 radians field. These sites are the main wall pass
in `project_slice/perimeters/arachne/walls.rs` and both regeneration branches in
`top_surface.rs`. Do not use double-first conversion, final-only narrowing,
`to_radians`, an epsilon, or a tuned constant instead of the upstream operations.

The existing f64 parameter/storage types remain temporary compatibility shells
around this upstream float calculation. This slice changes only its three
construction expressions, not public API types, sorting, outlines, beading,
Clipper, Voronoi, fitting, emission, or other angle conversions. It introduces no
independent pipeline, platform dependency, fallback, or fixture-specific policy.

## Evidence and qualification

The unchanged whole-output test committed in `cdfde747c` is RED on parent `71f2119b0`.
A fresh paired runtime control (baseline107/control108) changes only the actual
L125 call2 factory angle's four bytes from `c2b8323e` to `c3b8323e`. Other raw and
prepared buffers, the complete actual factory configuration outside that window,
and control-file hashes are unchanged before resume. All captured factory and
trapezoidation fields, generated/stitched/filtered toolpaths and the returned
185-point contour become Native-exact. No optimized caller value is reconstructed.

The paired-control verification record captures this bounded causal result;
it is not a complete G-code qualification.
Keep the frozen fixture, comparator and assertions unchanged. After linting and a
separate commit, strict Native-double-replay/API/CLI directed parity must pass
before unfiltered official FULL, seeded FULL and REQUIRE=1 workspace non-parity.
Every broad-gate flip must be explained, using same-environment RED-tip controls
for new failures. Global parity is deferred to those gates, not inferred here.
