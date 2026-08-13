# Task 22O.46 architecture decision record

## Status

Accepted. The strict global fixed-MSVC replay rebuilt all 209 affected
`libslic3r`/CGAL objects per mode from pinned source, produced byte-identical
Debug and Release Layer results, and restored the pinned tree byte-exact. It
confirmed 103 O45 calls, 1,507 endpoint records with zero ties, 1,439 arc
records with 2,700 ties across 30 calls and 82 classes, and the normative
186-path / 5,941-point ordered digest
`917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.
The prior Linux and hybrid captures remain rejected diagnostics. The completed
Rust implementation passes the focused, dependency, workspace, Clippy, wasm32,
formatting, and structural gates, and independent source/specification and
standards rereviews approve unconditionally.

## Decision

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
public `Layer::generate_sparse_infill_polylines_for_anchoring` method from
`Layer.hpp:194-196` and `Fill/Fill.cpp:1377-1504` as the next dependency-first
rewrite slice.

The Rust boundary is one crate-private borrowed Layer view and one owned ordered
`Vec<Polyline>` result under
`project_slice::prepare_infill::bridge_over_infill::sparse_anchoring`. Private
implementation includes only the KSR-observable projection, grouping, and
priority portion of
`Fill.cpp:52-59,216-221,232-235,275-281,284,307-308,336-342,829-835`,
`Fill.cpp:855-858,861-862,864,866-867,881-884,891-898,925-926,934-936`, and
`Fill.cpp:943,979-989,1012-1067`. It directly calls O45.

Do not expose `SurfaceFillParams`, grouped jobs, projected primitive scalars, or
the lower-layer map. Do not add a prepared-project successor. Public slicing
remains terminal at O43.

## Rationale

The public Layer method is the smallest source-owned semantic result that hides
substantial policy: exact-corpus kind/pattern/bridge-angle projection, nominal
sparse Flow and angle/density/anchor cast order, comparator-equivalent grouping,
non-sparse priority trimming, and final CrossHatch generation. Its output is
independently callable and observable in pinned Orca.

Generic `group_fills` is also a real upstream concept with two callers, but a
complete port would additionally require the rotation-template language,
LockedZag sidecars, multi-region/no-overlap ownership, InternalVoid repair,
and narrow-solid splitting. Exporting a KSR-only partial group result would be
an invented seam. Keeping the reached subset private preserves the real Layer
interface while deferring generic grouping honestly.

The `PrintObject.cpp:2725-2761` map is transaction-local scheduling state with
one downstream semantic consumer at line 3203. Publishing it as an Ares stage
before that consumer exists would create a fake lifecycle checkpoint. A later
source-cited bridge transaction will own the map and consume it in the same
transaction.

## Consequences

- O46 reuses Ares's existing nominal sparse Flow resolver; actual/non-sparse
  role Flows, a second Flow model, and a legacy `InfillOptions` adapter are not
  introduced.
- The only shared geometry addition is the exact Polygon-subject/Polygon-clip
  safety-offset difference overload plus an existing safety-union reexport.
- Every retained KSR surface participates in bridge-angle/pattern grouping and
  priority geometry before exact `Internal` filtering. Independent per-surface
  filling is forbidden.
- Full-result literals use ARD-0024's fixed MSVC STL 14.44 sort control flow at
  both O44 sites; no Linux-host permutation, stable tie-break, or host sort is
  accepted.
- Inputs remain borrowed and unchanged; the result is owned. Reachable
  `ClipperError` values abort atomically, while source empty geometry remains
  `Ok(empty)`.
- O45 gains its next source-shaped caller and its temporary dead-code
  expectation is removed. A single reasoned expectation may mark O46 unwired.
- Full generic `group_fills`, its non-sparse postpasses, the lower-layer map,
  lifecycle activation, bridge commit, extrusion, motion, G-code, and CLI
  behavior remain deferred.
- No filesystem, UI, OpenGL, terminal, native-threading, platform fallback,
  public option/API, or workspace crate is introduced. The dependency remains
  suitable for WASM, Windows, macOS, and Linux.
