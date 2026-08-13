# Task 22O.44 architecture decision record

## Status

Accepted.

## Decision

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s shared
`Fill::connect_infill` as the next dependency-first rewrite slice. The exact
included and deferred ranges are normative in the matching O44 spec. The
destination is a crate-private `fill::connect` module returning final
connected/hooked polylines. It is not a prepared-project lifecycle stage and
it does not call the existing Ares `infills` compatibility scaffold.
`CoordinateScale` is explicit at the interface, replacing Orca's
process-global `SCALING_FACTOR` without changing its arithmetic.

```rust
pub(crate) fn connect_infill(
    infill_ordered: Vec<Polyline>,
    boundary: &ExPolygon,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError>;
```

`FillConnectionParams` retains the source field types: two `f32` anchors,
`i32` multiline, and `bool dont_sort`. Its fields are crate-private for the
next source-cited sibling fill module.

The module owns its working paths and hides a stable-index Rust equivalent of
Orca's mutable pointer-linked boundary/T-junction graph. The ExPolygon boundary
is borrowed and remains unchanged while the source-required contour-then-hole
working copy is split. Existing checked fixed-coordinate geometry failures
propagate before an output is returned.

Both active upstream sorts have comparators without a final tie-break. O44
amends ARD-0024's visibility decision only as far as necessary: the audited
MSVC STL 14.44 `fixed_msvc_sort_by` implementation becomes crate-private so
the sibling `fill` module can reuse its exact control flow and source
comparators. O44 adds no tie-break and does not call a host Rust sort.

## Rationale

The KSR path reaches CrossHatch anchor generation after O43, but the apparent
caller seam at `PrintObject.cpp:2725-2761` depends on `group_fills`, the complete
CrossHatch filler, and the large shared connector. The fixture's 20 mm anchor
maximum makes the connector active. A raw CrossHatch lattice is private local
state, and exposing it would create a one-consumer interface that omits active
connection and final rotation.

`Fill::connect_infill` is already a source-owned callable seam reused by many
fill patterns. Its small caller-facing interface hides substantial projection,
collision, boundary-splitting, ordering, merging, reversal, and hook logic.
Porting it first is therefore a deep dependency slice rather than a new Ares
pipeline stage. It also permits direct, exact testing against Orca's public
static helper before the broader fill dispatcher is available.

## Consequences

- Public slicing still consumes and disposes O43, then returns
  `ProjectSlicingIncomplete`; O44 alone changes no G-code byte.
- The exact CrossHatch filler can next consume O44 without depending on the
  legacy two-point scanline extension.
- A later deep integration transaction may own the source-analogous temporary
  `BTreeMap<lower_layer_index, Vec<Polyline>>`; O44 does not invent a public
  prepared checkpoint for it.
- That integration must project both anchor and anchor maximum from retained
  source options; the legacy `InfillOptions` accessor collapses them and is not
  a valid O44 adapter.
- The crate-private dependency has one reasoned
  `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` until the exact
  CrossHatch caller lands; no dummy production caller is added.
- Lightning preparation remains deferred as one generator-producing module;
  no temporary surface-overlay successor is introduced.
- Debug SVG, TBB scheduling, compile-disabled alternatives, support-fill
  connection, cancellation, logging, and host runtime behavior remain outside
  this platform-neutral core slice.
