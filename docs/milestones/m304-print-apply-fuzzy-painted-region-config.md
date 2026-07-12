# M304: PrintApply fuzzy painted region config

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the fuzzy-skin painted-region config derivation prefix in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:837-842`: iterate `fuzzy_skin_painted_regions`, resolve each parent print region via `region.parent_print_object_region(layer_range)`, copy the parent config, and force `cfg.fuzzy_skin.value = FuzzySkinType::All` only when the copied parent config is not `FuzzySkinType::Disabled_fuzzy`. Required context comes from `PrintObjectRegions::FuzzySkinPaintedRegion` in `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, `LayerRangeRegions::fuzzy_skin_painted_regions` in `Print.hpp:271-283`, parent resolution in `OrcaSlicer/src/libslic3r/Print.cpp:4932-4947`, and `FuzzySkinType` variants in `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-57`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned fuzzy-skin or slicing pipeline.

## Exit criteria

- Add private staged fuzzy-painted config derivation state for `PrintApply.cpp:837-842`.
- Preserve parent type resolution for both `VolumeRegion` and `PaintedRegion` parents.
- Preserve source order over fuzzy-skin painted regions.
- Preserve copying parent config before fuzzy override.
- Preserve that non-disabled parent fuzzy skin values become `All`.
- Preserve that `Disabled_fuzzy` remains disabled.
- Preserve fuzzy painted region id and parent reference metadata in staged derivations.
- Add tests for volume-region parent lookup, painted-region parent lookup, non-disabled-to-All normalization, disabled preservation, source order, and parent config immutation.
- Defer config comparison/update/reslice handling from `PrintApply.cpp:843-853`, ref-count increment from `PrintApply.cpp:856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
