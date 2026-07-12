# M300: PrintApply painted region extruder config

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the color-painted region config derivation prefix in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:813-820`: iterate painted regions, look up the parent volume region, copy `parent_region.region->config()`, and assign `region.extruder_id` to `wall_filament`, `solid_infill_filament`, and `sparse_infill_filament`. Required context comes from `PrintObjectRegions::PaintedRegion` in `OrcaSlicer/src/libslic3r/Print.hpp:243-252`, `LayerRangeRegions::painted_regions` in `Print.hpp:271-283`, `VolumeRegion` in `Print.hpp:229-240`, and the filament option fields in `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1121,1154,1161`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned painted-region or slicing pipeline.

## Exit criteria

- Add a private staged helper that derives painted-region configs from parent-indexed volume-region configs.
- Preserve assignment of the painted region `extruder_id` to `wall_filament`.
- Preserve assignment of the painted region `extruder_id` to `solid_infill_filament`.
- Preserve assignment of the painted region `extruder_id` to `sparse_infill_filament`.
- Preserve unrelated parent config fields while applying the three filament overrides.
- Add tests for each filament override, unrelated-field preservation, and distinct painted-region extruder ids producing distinct derived configs.
- Defer comparison/update/reslice handling from `PrintApply.cpp:821-831`, ref-count increment from `PrintApply.cpp:834`, fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
