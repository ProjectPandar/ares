# Task 22O.47 architecture decision record

## Status

Accepted and implemented. The private operation, eight focused discriminators,
and the 18-layer real-KSR regression pass. The KSR result contains 115 flat
Polygons and 5,641 points in 91,464 serialized bytes with ordered SHA-256
`f28db7dd3fc63155752ba5c33d4cd6338b2e311d83eb973c473d7f65268aa92a`.

## Decision

Port the `gather_areas_w_depth` dependency inside pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject::bridge_over_infill()` at
`OrcaSlicer/src/libslic3r/PrintObject.cpp:2819-2846`.

The Rust destination is a crate-private borrowed operation under
`project_slice::prepare_infill::bridge_over_infill::deep_sparse_area`. It
accepts aligned retained layer views, one candidate layer index, the caller's
already-scaled target bridge-flow height, and the existing coordinate scale. It
returns owned flat deep sparse-infill `Polygon` paths or the first
`ClipperError`, matching the source `diff(ExPolygons, ExPolygons) -> Polygons`
overload.

This is an unwired dependency operation, not a prepared-project stage. Public
project slicing continues to consume and dispose O43 and return
`ProjectSlicingIncomplete`.

## Rationale

The source lambda is a complete geometric operation with an independently
observable result. It is required before the real line-3203 sparse-anchor
consumer can construct its `expansion_area`.

The adjacent alternatives are not valid seams yet:

- `PrintObject.cpp:2725-2761`'s lower-layer polyline map is transaction-local
  scheduling state. Task 22O.46 already decided that it must be created and
  consumed in one future bridge transaction rather than published as a stage.
- `PrintObject.cpp:2763-2817`'s clusters isolate sequential mutations and also
  require the exact thick solid-infill bridge Flow and lower-bridge removal.
  Exporting clusters before those consumers would expose scheduling state.
- Combining the map directly with line 3203 now would require a caller-supplied
  fabricated `expansion_area`; the source computes that area through this
  deep-area operation and later current-layer processing.

## Consequences

- Inputs remain borrowed and unchanged; only ephemeral geometry is cloned.
- Traversal starts at the immediately lower layer. That layer is included even
  when it lies below the depth threshold; deeper traversal stops at the first
  lower `print_z`.
- `Internal` with region density below 100 percent and every `InternalVoid`
  surface are sparse. All other reached surfaces are non-sparse.
- Sparse and non-sparse geometry are independently unioned and closed by
  `SCALED_EPSILON`, then flattened contour-before-holes and subtracted through
  the source-equivalent one-pass path difference.
- The caller's target height already contains the first `0.9f` factor from
  `PrintObject.cpp:3155-3157`; this operation preserves the lambda's second
  `0.9f` multiplication at line 2823.
- No new Flow model, option, public API, filesystem access, native threading,
  fallback, map, lifecycle successor, or G-code behavior is introduced.
- Multi-region layer aggregation remains deferred beyond the current validated
  single-region KSR graph; a future transaction must carry density per region,
  not per whole layer.
- Thick bridge Flow, clustering, lower-bridge subtraction, current-layer
  expansion preparation, lower-layer map ownership/consumption, direction,
  anchored polygon construction, surface commit, extrusion, motion, and G-code
  remain deferred source-cited work.
