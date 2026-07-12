# M323: PrintApply generate_print_object_regions fuzzy volume-region append

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1075-1086`: when `has_painted_fuzzy_skin` is true, iterate each layer range and each volume region; for model-part or modifier parents, copy the parent region config, change fuzzy skin to `All` unless it is `Disabled_fuzzy`, and append a `FuzzySkinPaintedRegion` with parent type `VolumeRegion`, parent volume-region index, and `get_create_region(std::move(cfg))`.

This milestone depends on M316 staged region-set context from `PrintApply.cpp:998-1010`, generated volume-region staged context from M317-M320, and `PrintObjectRegions::FuzzySkinPaintedRegion` field context from `OrcaSlicer/src/libslic3r/Print.hpp:255-264`.

## Exit criteria

- Preserve `has_painted_fuzzy_skin` gate.
- Preserve volume-region parent iteration order.
- Preserve parent eligibility: only model-part and modifier volume regions produce fuzzy painted regions.
- Preserve parent config copy and fuzzy-skin derivation: non-disabled fuzzy skin becomes `All`, disabled fuzzy skin remains disabled.
- Preserve appended fuzzy-region fields: parent type `VolumeRegion`, parent volume-region index, and region id from the staged region set.
- Preserve region id reuse through the M316 staged `StagedGenerateRegionSet::get_create_region(...)` helper.
- Defer painted-region parent fuzzy append from `PrintApply.cpp:1089-1095`, fuzzy painted sorting from `PrintApply.cpp:1097-1100`, real `PrintRegionConfig`, real pointer identity, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
