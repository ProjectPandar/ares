# PrintApply printable-filament staged assembly Spec

## Goal
Port the full control-flow assembly of OrcaSlicer's private `is_printable_filament_changed(...)` into `ares-core` by composing the already staged PrintApply helpers, while still deferring concrete Clipper boolean operations.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-340`: private `is_printable_filament_changed(...)` guard, printable/extruder polygon construction, per-extruder diff split collection, all-extruder intersection append, `find_intersections`, old/new intersection-id comparison, and final boolean return.

Staged predecessor context:
- M264: `PrintApply.cpp:297-303` guard semantics.
- M265: `PrintApply.cpp:304-315` printable/extruder-area extraction.
- M266: `PrintApply.cpp:306-315` scaled `Point(...)` construction.
- M267: `PrintApply.cpp:317-320` per-extruder `diff(printable_poly, poly)` first-result collection.
- M268: `PrintApply.cpp:323-324` all-extruder `intersection({printable_poly}, extruder_polys)` first-result append.
- M269: `PrintApply.cpp:326-333` `find_intersections` set construction.
- M270: `PrintApply.cpp:335-337` old/new id-set comparison.

Boolean operation context only:
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:429-433` and `ClipperUtils.cpp:676-679`: `diff(...)` delegates to `_clipper(... ctDifference ...)`.
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:496-508` and `ClipperUtils.cpp:696-703`: `intersection(...)` delegates to `_clipper(... ctIntersection ...)`.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M271 until this M271 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private staged assembly helper that accepts `new_full_config_values`, one `(old_poly, new_poly)` pair parameter, and injected per-extruder diff, all-extruder intersection, and single-polygon intersection callbacks matching M267-M270 helper shapes.
- Return `Ok(false)` without invoking geometry callbacks when old and new polygons are equal.
- Return `Ok(false)` without invoking geometry callbacks when old and new polygons differ but `filament_map_mode` is manual.
- When the geometry branch is active, parse printable/extruder-area polygons from `new_full_config_values`, scale them using the staged M265/M266 helpers, and preserve all upstream ordering constraints.
- Build `split_polys` by calling the staged per-extruder diff first-result collector, then append the staged all-extruder intersection first result.
- Use the staged M270 `printable_filament_intersection_ids_changed` helper to compare scaled old/new polygons against `split_polys` and return that boolean.
- Propagate existing `SliceError::InvalidInput(...)` results from malformed config values.
- Do not introduce a geometry dependency or implement the actual Clipper difference/intersection operations in this milestone.
- Do not implement public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual polygon boolean operation.
- No public `is_printable_filament_changed` API wiring.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
