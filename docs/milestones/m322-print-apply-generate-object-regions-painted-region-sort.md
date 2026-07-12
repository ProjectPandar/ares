# M322: PrintApply generate_print_object_regions painted region sort

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1068-1072`: after painted regions are appended, sort each layer range's painted regions by the parent volume region's `print_object_region_id()`, then by `extruder_id`, to help MM segmentation.

This milestone depends on M321 staged painted-region append context from `PrintApply.cpp:1056-1067` and `PrintObjectRegions::PaintedRegion` field context from `OrcaSlicer/src/libslic3r/Print.hpp:243-251`.

## Exit criteria

- Preserve sorting primary key: `layer_range.volume_regions[painted.parent].region->print_object_region_id()`.
- Preserve sorting secondary key: `painted.extruder_id`.
- Preserve sorting within one layer range only; do not reorder layer ranges.
- Preserve behavior for empty and single-entry painted-region lists.
- Defer fuzzy painted construction from `PrintApply.cpp:1075-1101`, real `PrintRegion` pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
