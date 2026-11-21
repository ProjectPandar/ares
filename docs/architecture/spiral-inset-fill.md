# Spiral Inset for project surfaces and support interfaces

## Included

At OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`PrintConfig.cpp:336,474,2549–2575,2627–2633,7570–7589` registers
`spiralinset` for top and bottom surface fill and for support interfaces.
The project option enums in `options/process_options/object_source/enums.rs`
and their registry metadata must retain the canonical token through load,
effective configuration, and export. Preserve other choices and defaults.
The profile-backed STL entry uses this same project pipeline.

`FillBase.cpp:69,129–171` dispatches Spiral Inset and applies the common
`overlap - spacing / 2` domain offset. Implement the source algorithm in
`fill/spiral_inset` rather than aliasing it to Concentric or rectilinear fill:

- The classic overload (`FillSpiralInset.cpp:310–345`) derives integer ring
  spacing from density, with the existing solid-spacing adjustment when
  enabled. It accumulates contour and hole rings with the paired shrink/grow
  offset and orders them through the outside-in PolyTree walk. On the pinned
  source the original ExPolygon is moved into the offset stack before the
  long-hop containment check; the classic overload therefore admits only
  the unconditional short hops. Do not substitute a copy of the domain in
  the containment check and silently change that behavior.
- The solid Arachne overload (`:347–496`) offsets outward by half a bead,
  uses the same fixed WallToolPaths inputs as the standard Concentric fill,
  and sorts closed rings by containment depth, preserving sibling source
  order. The source domain remains available for the long-hop containment
  check. Retain open walls and chain them at the nearest spiral's center
  end with the upstream endpoint tie order. Below full density or when
  `dont_adjust` is set, dispatch to classic and convert to constant widths.
- The shared chain (`:29–308`) selects a convex square corner or the widest
  non-straight convex corner initially, then uses nearest vertices, splits
  at island changes or unreachable joins, and leaves the angle-dependent
  gap at each handoff. Clip the final ring by half the gap. Only classic
  adds a reachable centroid stub when the last ring still encloses a hole.
  Preserve point rounding, endpoint interpolation and two widths per
  segment, including tapering across unequal-width joins. Default and
  outward run from the center out, reversing path order and direction;
  inward leaves the original order unchanged.

`Fill.cpp:1010–1015,1425–1450` projects top/bottom order and selects the
Arachne overload independently of the perimeter generator. In the project
surface dispatcher (`project_slice/fill_entities`), preserve the existing
role/flow, bridge selection and residual gap-fill lifecycle. Keep Spiral
Inset collections unsortable (`FillSpiralInset.hpp::no_sort`) and honor the
explicit-order reversal gate from `FillBase.cpp:193–225`. Do not change how
other centered patterns interpret their order settings in this slice.

`SupportParameters.hpp:151–161`, `SupportCommon.cpp:1698–1722,1862,1944`,
and `TreeSupport.cpp:1688–1720` dispatch Spiral Inset for contact/interface
fill, not the base-fill pattern. Use the classic overload with the support
caller's spacing and density, `dont_adjust`, and default outward order.
Route the existing support and raft-contact callers through the same
pattern entry without changing their source-owned first-layer, raft or
interface applicability. Do not select it for base/scaffold-only fills.

## Dependencies and deferred boundaries

Reuse the existing miter offset and `offset2_ex`, outside-in union contour
walk, point and open-polyline containment, polygon centroid, classic
polyline tail clipping, Arachne wall solver and variable-width extrusion
materializer. New shape logic is limited to ring corner/clip decisions,
classic/variable ring chaining and the solid-wall nesting and ordering.
Spiral Inset clips variable widths together with points; the existing
`ThickPolyline::clip_end` does not implement that operation and must not be
used in its place. Any numeric mismatch in the reused infrastructure must
be localized before changing existing callers.

The independent raw STL/option-only path in `options/infill/patterns.rs`
and `infills` does not run the project Arachne fill. Adding parsing alone
there would accept a value without generating its toolpath, so that entry
is deferred. No new sparse-infill, internal-solid, ironing or surface UI
pattern choices are part of this slice. Other centered patterns, general
gap-fill ordering improvements, Arachne solver changes, and changing the
pinned classic moved-from behavior are separate boundaries.

## Public verification

Patch one setting at runtime in each committed input: support interface in
`support-interface-top-layers-base-connect/control.3mf`; top and bottom
surface separately in `orca_cli_ender3.3mf`. No derived Native fixture is
committed. Require Native to retain `spiralinset` and emit the intended
role; compare a Native replay, Ares public project API and Ares CLI via the
whole ordered G-code comparator with generator identity normalization only.
Before implementation, both modes must demonstrate the original fallback
footer and differing path stream; after implementation all three must
match and adjacent existing cases must show no pass-to-fail transitions.
