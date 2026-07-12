# PrintApply scaled printable-area polygons Spec

## Goal
Port OrcaSlicer's `scale_` plus `Point(...)` conversion loops for printable-area and extruder-area polygons into `ares-core` as private staged helpers for later geometry diff wiring.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:306-315`: loop through `printable_area` and each `extruder_area`, append `Point(scale_(pt.x()), scale_(pt.y()))`, and preserve polygon/group order.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:316-340`: deferred `diff`, `intersection`, split-polygon assembly, and old/new intersection-id comparison.
- `OrcaSlicer/src/libslic3r/libslic3r.h:40-43`: `coord_t` integer coordinate type.
- `OrcaSlicer/src/libslic3r/libslic3r.h:60-64`: `SCALING_FACTOR_INTERNAL = 0.000001` and large-printer constant context.
- `OrcaSlicer/src/libslic3r/libslic3r.h:92-94`: `scale_(val) = val / SCALING_FACTOR` and `unscale_` context.
- `OrcaSlicer/src/libslic3r/libslic3r.cpp:3`: default `SCALING_FACTOR = SCALING_FACTOR_INTERNAL`.
- `OrcaSlicer/src/libslic3r/Point.hpp:190-205`: `Point` stores `coord_t` and rounds double constructor inputs.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M266 until this M266 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add private staged integer geometry types, for example:
  - `ScaledPoint { x: i64, y: i64 }`;
  - `ScaledPrintableAreaPolygons { printable: Vec<ScaledPoint>, extruders: Vec<Vec<ScaledPoint>> }`.
- Add a private helper, for example:
  - `fn scale_printable_area_polygons(polygons: &PrintableAreaPolygons) -> ScaledPrintableAreaPolygons`.
- Use Orca's default scaling for this staged slice: divide each millimeter coordinate by `0.000001` and round to the nearest integer, matching `Point(double, double)` after `scale_(...)` under the default `SCALING_FACTOR`.
- Preserve printable point order, extruder group order, and per-group point order exactly.
- Support negative and fractional coordinates according to the same scaling/rounding rule.
- Do not introduce external geometry dependencies or public API surface.
- Do not implement large-printer scaling-factor selection, Clipper `diff`, Clipper `intersection`, split polygon assembly, intersection-id comparison, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No geometry boolean operations.
- No dynamic scaling-factor selection for large printers.
- No public API or UI-facing API.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
