# M265: PrintApply printable-area polygon extraction

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the printable-area and extruder-area polygon construction prefix inside `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:304-315`, with option-definition context from `PrintConfig.cpp:684-693` and declaration context from `PrintConfig.hpp:1481-1482`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add internal `ares-core` helpers that read staged JSON values for `printable_area` and `extruder_printable_area` as finite `[x, y]` point pairs and preserve Orca's point and group ordering as `Point2` polygons.
- Preserve upstream required `printable_area` behavior for this staged helper: missing or non-array printable area is invalid because Orca dereferences the option before constructing the polygon.
- Preserve upstream optional-empty `extruder_printable_area` default behavior by returning no extruder polygons when the key is absent.
- Reject malformed `printable_area` with `SliceError::InvalidInput("printable_area must be an array of [x,y] points")`.
- Reject malformed `extruder_printable_area` with `SliceError::InvalidInput("extruder_printable_area must be an array of point arrays")`.
- Do not implement scaling to `coord_t`, Clipper `diff`/`intersection`, split polygon assembly, intersection-id comparison, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
