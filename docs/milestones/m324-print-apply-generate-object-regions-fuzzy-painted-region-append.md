# M324: PrintApply generate_print_object_regions fuzzy painted-region append

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1089-1095`: while `has_painted_fuzzy_skin` is true, iterate each layer range's already generated `painted_regions`; for every painted-region parent, copy the parent painted region config, change any non-`Disabled_fuzzy` fuzzy-skin value to `All`, and append a `FuzzySkinPaintedRegion` with parent type `PaintedRegion`, the parent painted-region index, and `get_create_region(std::move(cfg))`.

This milestone depends on M316 staged region-set context from `PrintApply.cpp:998-1010`, M321-M322 generated painted-region context, M323 fuzzy volume-region context, `PrintObjectRegions::PaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:243-251`, `PrintObjectRegions::FuzzySkinPaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, and `FuzzySkinType` variants from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-57`.

## Exit criteria

- Preserve the `has_painted_fuzzy_skin` false no-op gate.
- Preserve source-order iteration over existing painted-region parents within one layer range.
- Preserve config derivation: copy painted parent config and change non-disabled fuzzy skin to `All`, while keeping disabled fuzzy skin disabled.
- Preserve appended fields: parent type `PaintedRegion`, parent painted-region index, and region id from the staged region set.
- Preserve region id reuse through the M316 staged `StagedGenerateRegionSet::get_create_region(...)` helper.
- Defer fuzzy painted sorting from `PrintApply.cpp:1097-1100`, real `PrintRegionConfig`, real configs/regions/pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
