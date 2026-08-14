# Task 22O.99 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the Arachne beading strategy stack from OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `Arachne/BeadingStrategy/BeadingStrategy.hpp/.cpp`;
- `DistributedBeadingStrategy.hpp/.cpp`;
- `RedistributeBeadingStrategy.hpp/.cpp`;
- `WideningBeadingStrategy.hpp/.cpp`;
- `LimitedBeadingStrategy.hpp/.cpp`;
- `OuterWallInsetBeadingStrategy.hpp/.cpp`;
- `BeadingStrategyFactory.hpp/.cpp`.

The crate-private `ares-core::arachne::beading` boundary owns `Beading`, an
object-safe strategy interface, the six concrete/base meta-strategies, and the
source-ordered factory composition. Integer conversions, f32 distribution
weights, odd/even transition thresholds, outer-wall redistribution, the Orca
thin-wall two-bead guard, zero-width limit markers, optional signed outer inset,
and active-coordinate-scale ten-micron constants follow the pinned sources.

The factory takes already-scaled typed values, plus the active
`CoordinateScale`, and returns the same stack order as Orca. It does not read
configuration or package data itself.

## Boundary

O99 is inactive outside its focused tests. Skeletal trapezoidation, half-edge
graphs, `WallToolPaths`, extrusion-line production, variable-width entity
conversion, `FillConcentricInternal`, project lifecycle, motion, and G-code are
deferred to later source-cited slices. No alternate Ares beading algorithm,
fixture branch, or legacy fallback is introduced.

Ten source-worked tests pass. Rustfmt, strict all-target core Clippy, diff,
macro, and LOC gates pass; the largest implementation/test shards are 117/268
LOC.
