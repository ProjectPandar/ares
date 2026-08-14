# Task 22O.99 implementation plan

## Source boundary

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`Arachne/BeadingStrategy/{BeadingStrategy,DistributedBeadingStrategy,
RedistributeBeadingStrategy,WideningBeadingStrategy,LimitedBeadingStrategy,
OuterWallInsetBeadingStrategy,BeadingStrategyFactory}.{hpp,cpp}` into
crate-private `ares-core::arachne::beading`. Include the strategy arithmetic,
metadata delegation, and factory composition. Defer half-edge/skeletal
trapezoidation, `WallToolPaths`, `FillConcentricInternal`, lifecycle, motion,
and G-code. No prior Ares beading scaffold exists; O99 adds no fallback.

## Steps

1. Add source-derived tests for the base/distributed strategy's bead layouts,
   thresholds, anchors, lengths, nonlinear points, and integer/f32 rounding.
2. Port the base value/interface and distributed strategy.
3. Add worked tests and ports for redistribution, widening, limiting, and
   signed optional outer inset.
4. Add the source-ordered factory and a KSR-style 0.42/0.42 mm worked stack.
5. Run focused Nextest, rustfmt, strict core Clippy, diff, macro, and LOC gates.
6. Update evidence; parent independently reviews, integrates, and pushes.

## Completed evidence

Ten source-worked tests pass, including full-expression conversion order and
the KSR-style factory stack. Rustfmt, strict all-target core Clippy, diff,
macro, and LOC gates pass; the largest implementation/test shards are 117/268
LOC. O99 remains inactive.
