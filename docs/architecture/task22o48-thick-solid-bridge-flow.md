# Task 22O.48 architecture decision record

## Status

Accepted and implemented. Seven focused field-bit/error/ownership tests pass,
and O47's 18-layer KSR flat geometry remains byte-identical when its target
height is resolved through O48.

## Decision

Port the `frSolidInfill, thick_bridge=true` specialization of pinned
OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`LayerRegion::bridging_flow()` from
`OrcaSlicer/src/libslic3r/LayerRegion.cpp:31-61`, with
`PrintRegion.cpp:7-22`, `Config.hpp:624-628,1284-1286`,
`Flow.hpp:14,49-115`, and `Flow.cpp:213-229` as reached dependencies.

The Rust destination is a crate-private resolver in
`project_slice::perimeters::flow` returning the existing
`project_slice::perimeters::types::Flow`:

```rust
pub(in crate::project_slice) fn resolve_thick_solid_infill_bridge_flow(
    region: &RegionOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<Flow, SliceError>;
```

It resolves `frSolidInfill` through `internal_solid_filament_id`, applies
`bridge_line_width` against that nozzle, applies positive `bridge_flow` through
the source f64-square-root/f32 multiplication order, and creates a circular
bridge Flow. It reuses a private role-neutral thick-bridge helper already
needed by overhang Flow resolution.

## Rationale

`PrintObject::bridge_over_infill()` consumes this exact Flow at
`PrintObject.cpp:2795,3154-3157,3214`. O47 currently accepts the caller-scaled
target height; O48 removes the test-only manual reconstruction and supplies the
real typed embedded-option dependency for future clustering, deep-area, and
anchored-polygon composition.

The complete `LayerRegion::bridging_flow` method also contains non-thick
rounded extrusion behavior and other roles. Those are not reached by the cited
bridge-over-infill calls and remain owned by existing role resolvers or future
source slices.

## Consequences

- No second Flow type or extrusion scaffold is introduced.
- Selector zero, underflow, and out-of-range preserve Orca's element-zero
  fallback already used by Ares Flow resolution.
- Absolute and percent bridge widths preserve source f64 evaluation before the
  f32 thread-diameter cast.
- Positive flow ratio is square-rooted in f64, cast to f32, then multiplied
  into the f32 diameter. Zero/nonpositive source behavior is retained behind
  existing project option validation.
- Bridge width and height equal the thread diameter; spacing is diameter plus
  `0.05` mm; `mm3_per_mm` is the circular f32 area promoted to f64.
- No lifecycle stage, map, geometry, public API, filesystem behavior, fallback,
  or G-code output is introduced.
