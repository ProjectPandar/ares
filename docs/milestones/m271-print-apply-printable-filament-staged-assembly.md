# M271: PrintApply printable-filament staged assembly

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the full private `is_printable_filament_changed(...)` control flow in `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-340`, assembled from the already staged M264-M270 slices, with polygon operation context from `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:429-433`, `ClipperUtils.hpp:496-508`, `ClipperUtils.cpp:676-679`, and `ClipperUtils.cpp:696-703`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private `ares-core` staged helper that composes the M264 guard, M265 printable/extruder-area extraction, M266 scaling, M267 per-extruder diff collection, M268 all-extruder intersection append, M269 `find_intersections` helper, and M270 old/new id-set comparison into the upstream `is_printable_filament_changed(...)` control flow.
- Preserve upstream behavior from `PrintApply.cpp:297-340`: return `false` for equal old/new polygons; return `false` for manual filament map mode; otherwise build scaled printable/extruder polygons from the new full config, collect split polygons by diffing printable against each extruder polygon, append the first all-extruder intersection result when present, and return the old/new intersection-id comparison.
- Keep the actual Clipper difference/intersection implementation deferred; this milestone accepts injected private callbacks so later source-cited milestones can port or select the boolean backend.
- Do not add public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, crates, dependencies, or independent Ares pipeline behavior.
