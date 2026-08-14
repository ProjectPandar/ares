# Task 22O.99 — Arachne beading strategies

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`Arachne/BeadingStrategy/BeadingStrategy.*`,
`DistributedBeadingStrategy.*`, `RedistributeBeadingStrategy.*`,
`WideningBeadingStrategy.*`, `LimitedBeadingStrategy.*`,
`OuterWallInsetBeadingStrategy.*`, and `BeadingStrategyFactory.*` into
crate-private `ares-core::arachne::beading` modules.

Requirements:

- preserve 0/1/2/many-bead layouts and per-bead integer/f32 rounding;
- preserve odd/even counts, transition thicknesses, anchors, lengths, and
  nonlinear-thickness reporting;
- retain outer-wall redistribution and minimum variable-width decisions;
- retain Orca's thin-wall branch only when at most one bead is requested;
- retain limited-strategy zero-width markers and symmetric locations;
- apply a nonzero signed outer inset before the limiting strategy;
- compose the exact pinned factory order from typed scaled inputs;
- resolve ten-micron source constants with active `CoordinateScale`;
- keep focused tests separate and every ordinary Rust file below 400 LOC;
- do not activate project slicing.

Deferred: half-edge/skeletal topology, `WallToolPaths`, extrusion-line output,
variable-width entities, `FillConcentricInternal`, lifecycle, motion, and
G-code.
