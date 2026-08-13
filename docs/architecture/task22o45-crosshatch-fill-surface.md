# Task 22O.45 architecture decision record

## Status

Accepted and implemented.

## Decision

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
public inherited CrossHatch `Fill::fill_surface` transaction as the next
dependency-first rewrite slice. The exact included and deferred ranges are
normative in the matching O45 spec. The destination is a crate-private
`fill::cross_hatch` module returning final offset, clipped, connected, and
rotation-restored polylines. It consumes O44 internally, does not create a
prepared-project lifecycle stage, and does not call the existing Ares
`infills` compatibility scaffold.

The module accepts one borrowed raw ExPolygon, already-resolved source-typed
fill parameters, and explicit `CoordinateScale`. It owns every offset component
and generated path. Checked geometry failures return before any result is
published, and the borrowed surface remains unchanged.

O45 intentionally selects only the KSR-reached `multiline == 1` no-op and
active boundary-connection branch. Supporting other multiline values would
require Orca's separate Clipper2 open-offset implementation; supporting an
anchor maximum below 0.05 mm would require the free
`ShortestPath.cpp:1968-1996::chain_polylines` function.
Neither receives an Ares fallback.

## Rationale

The protected `_fill_surface_single` override is not the right external seam:
it consumes an already-offset component, appends into a shared mutable vector,
and exposes unused direction/thickness arguments. The public inherited method
adds active f32 half-spacing offset behavior and ordered component ownership,
then returns the meaningful final path vector.

The broader anchor-map caller is also not the right O45 seam. Source-faithful
activation at `PrintObject.cpp:2725-2761` additionally requires
`Fill.cpp::group_fills`, nominal sparse-flow resolution, angle/template
projection, adaptive/Lightning choices, and transaction-local map ownership.
Porting those by independently filling retained `Internal` surfaces would be
an Ares-designed shortcut. CrossHatch is a reusable prerequisite with a deep
owned-result interface and can be tested exactly through Orca's public method.

## Consequences

- O44 gains its intended source-shaped CrossHatch caller; its temporary
  dead-code expectation is removed.
- O45 itself remains crate-private and unwired with one narrow reasoned
  dead-code expectation. Public slicing still disposes O43 and returns
  `ProjectSlicingIncomplete`, so O45 changes no G-code byte.
- A later source-cited transaction may project nominal sparse flow and
  CrossHatch angle, call this seam through complete `group_fills`, and own the
  lower-layer `BTreeMap<usize, Vec<Polyline>>` until downstream bridge-anchor
  consumption.
- The old `infills.rs`, `InfillPath`, rotation helper, and collapsed legacy
  anchor accessor remain temporary compatibility scaffolds, not source parity.
- The only shared geometry addition is open-polyline Intersection over the
  already-existing checked Clipper worker. Pattern-specific transforms and
  arithmetic stay private to `fill::cross_hatch`.
- No filesystem, UI, OpenGL, terminal, native-threading, platform sort,
  fallback, public option, or new workspace crate is introduced. The module
  remains compatible with WASM, Windows, macOS, and Linux.
- Exact public-seam coverage freezes source f32 multiplication and f32 `exp`
  in the repeat-ratio calculation; widening either operation changes the
  ordered CrossHatch path and is rejected.
