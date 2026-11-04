# ZAA prerequisite: mechanical fill Polyline3 migration

## Boundary

This is the separate mechanical prerequisite of repair PLAN
90381524dd2bfdf862998e678b7b2dbc7bfcc411. It starts after the zag and
Triangles changes were merged in main 104d8497097dc7239ad63f7a69f3b418b699b3e8,
as a standalone prerequisite on main
57fea94989ff362129d0e53da0c35c298822f689. The branch contains no live-ZAA
RED, Tree3f/contour kernel commit or contour stage caller. Those remain on
the separate feature stack until submitted with production GREEN.

Upstream commit 8500fcdccaa10b5099ac20d252af3a7c560046f1:

- `ExtrusionEntity.hpp:153-166`: ExtrusionPath owns Polyline3, flow, width,
  height and a default-false z_contoured flag.
- `Polyline.hpp:291-301,341-344`: XY-to-XYZ conversion defaults Z to zero;
  fitting data belongs to the polyline.
- `ExtrusionEntity.cpp:52-58`: XY consumers collect XY coordinates from XYZ.

Rust boundary: FillExtrusionPath in project_slice/fill_entities/types.rs
uses the existing materialize::Polyline3 carrier. FillExtrusionPath remains
an Ares compatibility shell around upstream ExtrusionPath; this prerequisite
does not expand fill algorithms or replace their individual source boundaries.

## Included and deferred

All ordinary fill constructors embed Z=0 and z_contoured=false. The existing
fitting Vec moves into Polyline3.fitting. Flow, role, width, height, ordering,
reversal, fitting indices, arc direction and emission coordinates stay the same.
The prior fill and perimeter reverse routines share their unchanged operations
through Polyline3::reverse. Existing assertions retain their operators and
expected geometry; only constructor carriers change.

DP retains its comparison order, distance expression, tolerance and selected
points. A borrowed projection lets the same algorithm operate directly on
Point3 without an intermediate XY Vec. Raft coverage accepts an iterator;
its existing polygon allocation collects XY directly. Conflict collection and
skirt bounds project coordinates lazily. G-code still emits the same XY stream.

The owned XY constructor reserves the source Vec capacity and consumes points
once into the wider XYZ elements. This requires one allocation per polyline,
not per point. It creates no cloned point Vec, simultaneous XY/XYZ sidecar,
per-point heap object or newly allocated projection buffer. The fitting Vec
moves without copying. Existing simplification result capacities are unchanged.

Deferred: fill contour application, nonzero fill Z, motion XYZ and extrusion
scaling, ZAA guard deletion and retirement of its old private capability test.
The public enabled-ZAA RED is absent from this standalone branch. It remains
on the separate feature stack and is excluded from the67 mechanical directed
cases; the earlier67-case run did not report a result for that test.
The existing capability guard and its old private assertion remain unchanged.

## Verification contract

Run existing fill-grid/zag, Triangles multiline, multiline union, local
internal-solid Grid and support-ironing pattern regressions in both Native
modes with the strict whole-output assertions intact. Each mode uses fresh
Native cache/artifact/target directories.
Also run existing fill/materialize/chaining/simplification core tests,
strict clippy, fmt and wasm checks. Runtime evidence is never committed to
the repository.
