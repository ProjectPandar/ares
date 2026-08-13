# Task 22O.42 — Process external surfaces

## Status

Implemented, verification-complete, and independently approved after review
repairs. The normalized KSR golden remains RED at the CLI `--options`
boundary.

## Goal and upstream boundary

Port OrcaSlicer 2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/LayerRegion.cpp:486-623::LayerRegion::process_external_surfaces`,
and activate it immediately after horizontal-shell discovery as ordered by
`src/libslic3r/PrintObject.cpp:610-641`.

The Rust destination is a crate-private
`project_slice::prepare_infill::external_surfaces::process` record transform
plus an `external_surfaces::prepare` lifecycle adapter consuming
`PreparedPostHorizontalShellPropagation`. The behavior seam is one owned layer
record; the public lifecycle seam remains `slice_project` with a 3MF byte
input.

## Direct dependencies

- `LayerRegion.cpp:146-164::fill_surfaces_extract_expolygons` for move-based
  source extraction and layer thickness;
- `LayerRegion.cpp:395-484` for the O35 and O41 expansion helpers;
- `Flow.hpp:57-67::Flow::scaled_width/scaled_spacing` for integer-scaled flow
  values;
- `Algorithm/RegionExpansion.hpp:69-83::RegionExpansionParameters::build`;
- `Surface.hpp:9-47` and `Surface.cpp` collection operations for default
  metadata, removal, append, clear, and final order;
- `ClipperUtils.hpp/cpp` for union and difference geometry;
- `libslic3r.h` scaling constants and `Geometry::deg2rad`.

## Required behavior

For every valid prepared layer record, Ares must:

1. derive shell width, minimum expansion, top/bottom/bridge expansion, the
   0.1 mm step, and the solid-spacing closing radius with the source's f32/f64
   operation order;
2. move and independently union `InternalSolid`, `Internal`, and `Top` sources
   into expansion zones in that order while retaining a source thickness;
3. select absolute custom-angle expansion only when `bridge_angle > 0` and
   `relative_bridge_angle == false`; otherwise run O41 automatic detection;
4. add the model rotation only to absolute custom angles, and add a relative
   custom angle only to nonnegative automatically detected angles;
5. restore bridge-trimmed Top geometry, then expand Bottom and Top in that
   order using the updated solid-zone parameters;
6. when global `spiral_mode` is false and sparse density is positive, move
   sparse regions with area `<= scale(scale(minimum_sparse_infill_area))` into
   the solid zone and union them;
7. clear and rebuild output strictly as `InternalSolid`, `Internal`, bridges,
   bottoms, then tops, using default metadata plus the retained thickness; and
8. propagate geometry errors without fallback. The consuming lifecycle drops
   the owned failed stage; it does not defensively copy the full project graph.

The stage uses the already composed `RegionOptions`, scaled classic-prelude
flow widths/spacings, per-record model rotation, global print spiral option,
and `CoordinateScale`. No Option is read from CLI data or invented locally.
When none of the three zone sources exists, the upstream local thickness is
unobservable because no thickness-bearing zone is emitted; the Rust boundary
uses the existing `RegionSurface` default `-1.0` sentinel and still performs
the source's final clear/rebuild.

## KSR values covered by the active path

The committed project resolves `wall_loops=2`, `bridge_angle=0`,
`relative_bridge_angle=false`, `align_infill_direction_to_model=false`, sparse
density 15%, minimum sparse area 15 mm², `spiral_mode=false`, Normal coordinate
scale, and identity XY rotation. Per-layer perimeter/external/solid flows are
already present in `PerimeterInputRecord` and `ClassicPreludeRecord`.

## Included and deferred behavior

Included is the complete active `#if 1` body, direct record behavior, lifecycle
alignment, owned success/error cleanup, and the KSR 3MF path through the new
stage. Unused `lower_layer` arguments, debug SVG/logging, cancellation, the
dead `#else` implementation, adjacent fill generation, toolpaths, motion,
G-code, processors, and CLI project activation remain deferred. Existing Ares
pipeline code is a temporary compatibility shell around these upstream
boundaries, not an independently designed slicing pipeline.

## Acceptance

Tests must cover zero/nonzero walls, exact derived parameters at both scales,
automatic/absolute/relative bridge angles, zone reconstruction and final
ordering/metadata, sparse-area promotion including equality and spiral/density
gates, geometry errors, lifecycle alignment, and the real KSR project. Tests
live in ordinary separate modules and every Rust file remains below 400 LOC.
Focused and workspace Nextest, rustfmt, warning-denying workspace Clippy, LOC,
include-macro, and normalized golden progress checks must run before a fresh
six-dimensional independent review.

## Verification evidence

The lifecycle stub RED left an injected `InternalVoid` in the real KSR graph.
After activation and review repairs, 19 focused O42 tests pass, all 72
external-surface tests pass, and the O24-O26/O40-O42 regression band passes
119/119. Workspace Nextest passes 6,126/6,126 with 27 slow and two skipped.
Warning-denying all-target/all-feature workspace Clippy, rustfmt, diff,
include-macro, and sub-400-LOC audits pass; `ares-core` and `ares-wasm` both
check for `wasm32-unknown-unknown`.

Review repairs made public O42 invocation/disposal observable, marked and
verified all 460 present records, drove sparse-area behavior from a real 3MF
option mutation, retained the sparse-zone allocation, reserved the complete
five-group output once, matched `ClipperError` exhaustively, and completed the
LargeBed nonzero-wall arithmetic matrix. Automatic O41 outputs always receive
a nonnegative angle before O40; therefore the source's inline negative-angle
guard is structurally unreachable for this trusted Rust seam and has no
manufactured invalid-input test.

The final independent standards, specification, and upstream-parity re-review
returned unconditional approval with no remaining findings.

The ignored normalized KSR golden was run fresh and remains the expected RED:
the CLI rejects the project-only invocation because `--options` is still
required. O42 changes that project graph before the explicit next-stage
`ProjectSlicingIncomplete` terminal; it does not yet activate the project-only
CLI contract or emit G-code.
