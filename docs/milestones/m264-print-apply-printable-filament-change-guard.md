# M264: PrintApply printable-filament change guard

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the entry guard of `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-303`, with deferred geometry-comparison context from `PrintApply.cpp:304-340` and `filament_map_mode` option-definition context from `PrintConfig.cpp:577-582`, `PrintConfig.cpp:2414-2428`, and `PrintConfig.hpp:424-428` / `PrintConfig.hpp:1335`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add an internal `ares-core` helper that stages the `is_printable_filament_changed(...)` entry guard over JSON option maps and point polygons.
- Preserve upstream guard behavior: equal old/new polygons return `false`; differing polygons with manual `filament_map_mode` return `false`; differing polygons with absent or non-manual `filament_map_mode` enter the deferred geometry branch.
- Represent the deferred geometry branch with a private staged `true` result only for this milestone; document that actual printable-area/extruder-area intersection semantics remain unimplemented.
- Add focused tests for equal polygons, differing polygons with missing mode, differing polygons with manual mode, and differing polygons with a non-manual mode.
- Do not implement printable-area/extruder-area polygon construction, Clipper diff/intersection behavior, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
