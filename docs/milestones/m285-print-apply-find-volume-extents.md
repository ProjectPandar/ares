# M285: PrintApply find volume extents lookup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `find_volume_extents(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:697-703`, with `PrintObjectRegions::VolumeExtents` context from `OrcaSlicer/src/libslic3r/Print.hpp:224-228`, `PrintObjectRegions::LayerRangeRegions::volumes` sorted-by-id context from `OrcaSlicer/src/libslic3r/Print.hpp:271-278`, `ObjectID` ordering context from `OrcaSlicer/src/libslic3r/ObjectID.hpp:20-37`, and lower-bound lookup style context from `OrcaSlicer/src/libslic3r/libslic3r.h:230-247`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to `find_volume_extents(...)` over sorted staged volume extent records.
- Preserve lower-bound-by-volume-id lookup semantics: find the first extent whose `volume_id` is not less than the queried id.
- Preserve returning the bbox only when the found `volume_id` equals the queried volume id.
- Preserve returning no bbox when the lower bound is at the end or points to a different id.
- Add tests for exact match, query below first id, query between ids returning none, query above last id returning none, and duplicate-id behavior matching lower-bound first equal record.
- Defer real `PrintObjectRegions::LayerRangeRegions`, `ModelVolume`, `ObjectID` wrapper types, bbox clipping/recalculation, modifier extents, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
