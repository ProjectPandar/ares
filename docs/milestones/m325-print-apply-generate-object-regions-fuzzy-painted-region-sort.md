# M325: PrintApply generate_print_object_regions fuzzy painted-region sort

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1097-1100`: after fuzzy painted regions from volume-region and painted-region parents are appended, sort each layer range's `fuzzy_skin_painted_regions` by `FuzzySkinPaintedRegion::parent_print_object_region_id(layer_range)` to help fuzzy skin segmentation.

This milestone depends on M323-M324 generated fuzzy painted-region context, `PrintObjectRegions::FuzzySkinPaintedRegion` field context from `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, and parent resolution from `OrcaSlicer/src/libslic3r/Print.cpp:4932-4947`, where `PaintedRegion` parents resolve through `layer_range.painted_regions[parent].region` and `VolumeRegion` parents resolve through `layer_range.volume_regions[parent].region`.

## Exit criteria

- Preserve sorting each layer range's fuzzy painted regions only; do not reorder layer ranges.
- Preserve parent print-object region id resolution for both `VolumeRegion` and `PaintedRegion` fuzzy parent types.
- Preserve sorting primary key: resolved parent region `print_object_region_id()`.
- Preserve all fuzzy painted-region fields while reordering.
- Preserve behavior for empty and single-entry fuzzy painted-region lists.
- Defer real `PrintRegion` pointers, real `parent_print_object_region(...)`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
