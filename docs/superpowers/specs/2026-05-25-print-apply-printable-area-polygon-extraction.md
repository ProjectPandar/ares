# PrintApply printable-area polygon extraction Spec

## Goal
Port the printable-area and extruder-area point extraction prefix of OrcaSlicer's `is_printable_filament_changed(...)` into `ares-core` as private staged helpers for later geometry diff wiring.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:304-315`: read `printable_area`, read `extruder_printable_area`, preserve point order while constructing one printable polygon and multiple extruder polygons.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:316-340`: deferred `diff`, `intersection`, split-polygon assembly, and old/new intersection-id comparison.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:684-693`: `printable_area` and `extruder_printable_area` option defaults.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1481-1482`: option declarations as `ConfigOptionPoints` and `ConfigOptionPointsGroups`.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M265 until this M265 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private staged struct:
  - `PrintableAreaPolygons { printable: Vec<Point2>, extruders: Vec<Vec<Point2>> }`.
- Add a private helper:
  - `fn printable_area_polygons(new_full_config_values: &serde_json::Map<String, serde_json::Value>) -> Result<PrintableAreaPolygons, SliceError>`.
- Read `new_full_config_values["printable_area"]` as an array of finite JSON-number `[x, y]` point pairs and preserve order.
- Treat missing `printable_area`, non-array `printable_area`, malformed point pairs, non-number coordinates, or non-finite coordinates as `SliceError::InvalidInput("printable_area must be an array of [x,y] points")`.
- Read optional `new_full_config_values["extruder_printable_area"]` as an array of point arrays where every point is a finite JSON-number `[x, y]` pair; preserve group order and point order.
- Treat missing `extruder_printable_area` as an empty extruder polygon list, matching the upstream empty default from `PrintConfig.cpp:690-693`.
- Treat malformed `extruder_printable_area` groups, point pairs, non-number coordinates, or non-finite coordinates as `SliceError::InvalidInput("extruder_printable_area must be an array of point arrays")`.
- Use existing `Point2::new(x, y)` for staged floating-point polygons; do not introduce scaling to `coord_t` in this milestone.
- Do not implement scaling to `coord_t`, Clipper `diff`/`intersection`, split polygon assembly, intersection-id comparison, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual printable-filament change calculation beyond extraction.
- No geometry boolean operations.
- No public API or UI-facing API.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
