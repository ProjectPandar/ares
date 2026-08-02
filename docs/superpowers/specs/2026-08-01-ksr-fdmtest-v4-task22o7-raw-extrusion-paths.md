# Task 22O.7: Raw Extrusion Path Materialization

Date: 2026-08-01

## Boundary

This source-cited Rust rewrite ports the reached classic-perimeter seam from OrcaSlicer v2.4.2 `PerimeterGenerator.cpp:153-207,218-224`, reached definitions in `ExtrusionEntity.hpp:153-188,551-580`, and `Polyline.hpp:291-302` into crate-private `project_slice::perimeters::classic::materialize`.

It consumes O2's exact bbox prefilter, O5's ordered traversal seeds and pending branch, and O6's `Polyline`, polygon closure, `intersection_pl`, and `diff_pl`. O1 proves fuzzy skin inactive and rejects active `overhang_reverse`; unreachable fuzzy, steep-overhang, and reverse branches are not modeled.

## Required behavior

The fallible successor `PreparedPostClassicRawPaths` owns boxed `PreparedPostClassicTraversal` and aligned object/optional-record/surface/tree sidecars. Local minimal types are `ExtrusionRole::{ExternalPerimeter,Perimeter,OverhangPerimeter}`, fixed `Point3`/`Polyline3` with `z=0`, and `ExtrusionPath { polyline, role, mm3_per_mm, width, height }`. Each aligned node retains the source `Vec<ExtrusionPath>` directly. The public legacy extrusion scaffold is unchanged.

Dispatch uses only O5 `PendingPathBranch`. Ordinary seeds emit one exact `polygon.split_at_first_point()` path with seed role/flow/width and `layer_height as f32`. Overhang seeds derive `SCALED_EPSILON` by scaling `EPSILON=1e-4`, bbox-filter the borrowed route-selected final lower series, then append valid `intersection_pl` fragments before valid `diff_pl` fragments. Supported fragments use seed role/flow/width and layer height; remainder fragments use `OverhangPerimeter` and `overhang_flow` mm3/width/height. O6 order and orientation are preserved. A genuinely empty clipped result remains an empty source path vector; O8 owns the subsequent line-208 `continue`.

All fallible sidecars are built while borrowing O5. Errors map precisely to `SliceError`, then O5 is iteratively consumed. Trees are built postorder without recursion; terminal O7 sinking is iterative. Public slicing executes O7 and remains `ProjectSlicingIncomplete`.

## Verification and exclusions

Direct tests cover exact closure/casts, mixed append order and numeric provenance, route-final-series behavior, normal/large-bed epsilon, bbox edge crossing, error transactionality, determinism, and constrained-stack sinking. In-memory KSR coverage proves real ordinary/overhang reachability, roles, flow/height provenance, XYZ/path checksum determinism, route provenance, O5 preservation, and lifecycle execution.

This milestone stops before lines 208-210 start-point selection/chaining and line 227 `ExtrusionLoop` construction. O8 owns the empty-path `continue`, exact reached `ShortestPath` ordering/reversal, and loop-role wrapping. O9+ owns recursive entity traversal/collection, thin walls, entity chaining, orientation, G-code, and runtime output. No reference runtime oracle, fixture identity branch, source-pin hash, dependency, unsafe, filesystem/process/FFI, or platform-specific behavior is added.
