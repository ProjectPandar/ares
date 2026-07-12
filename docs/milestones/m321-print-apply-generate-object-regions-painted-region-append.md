# M321: PrintApply generate_print_object_regions painted region append

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1056-1067`: after volume-region generation, iterate each layer range, each `painting_extruders` entry, and each volume region; for model-part or modifier parents, copy the parent region config, set wall/solid/sparse filament values to the painted extruder id, and append a `PaintedRegion` with the extruder id, parent volume-region index, and `get_create_region(std::move(cfg))`.

This milestone depends on M316 staged region-set context from `PrintApply.cpp:998-1010`, generated volume-region staged context from M317-M320, and `PrintObjectRegions::PaintedRegion` field context from `OrcaSlicer/src/libslic3r/Print.hpp:243-251`.

## Exit criteria

- Preserve nested iteration order: layer ranges, then painting extruders, then volume regions.
- Preserve parent eligibility: only model-part and modifier volume regions produce painted regions.
- Preserve parent config copy before overriding filament roles.
- Preserve setting wall, solid infill, and sparse infill filament values to the painted extruder id.
- Preserve appended painted-region fields: extruder id, parent volume-region index, and region id from the staged region set.
- Preserve region id reuse through the M316 staged `StagedGenerateRegionSet::get_create_region(...)` helper.
- Defer painted-region sorting from `PrintApply.cpp:1068-1072`, fuzzy painted construction from `PrintApply.cpp:1075-1101`, real `PrintRegionConfig`, real pointer identity, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
