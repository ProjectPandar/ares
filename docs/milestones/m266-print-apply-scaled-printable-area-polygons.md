# M266: PrintApply scaled printable-area polygons

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `Point(scale_(pt.x()), scale_(pt.y()))` conversion loops inside `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:306-315`, with scaling context from `libslic3r.h:40-43`, `libslic3r.h:60-64`, `libslic3r.h:92-94`, `libslic3r.cpp:3`, and `Point.hpp:190-205`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private `ares-core` staged geometry helpers that convert the M265 `Point2` printable/extruder polygons into Orca-style scaled integer coordinate polygons while preserving printable point order, extruder group order, and per-group point order.
- Preserve Orca's default scaling basis for this slice: `SCALING_FACTOR_INTERNAL = 0.000001`, `SCALING_FACTOR = SCALING_FACTOR_INTERNAL`, `scale_(val) = val / SCALING_FACTOR`, and `Point(double, double)` rounding to integer `coord_t`.
- Keep the helper private and staged for later `diff` / `intersection` wiring.
- Do not implement large-printer scaling-factor selection, Clipper `diff`, Clipper `intersection`, split polygon assembly, intersection-id comparison, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
