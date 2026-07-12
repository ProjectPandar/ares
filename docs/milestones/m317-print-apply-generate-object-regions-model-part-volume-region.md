# M317: PrintApply generate_print_object_regions model-part volume region

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1012-1024`: after `get_create_region(...)`, `generate_print_object_regions(...)` iterates `model_volumes` in model-volume order, keeps only `model_volume_solid_or_modifier(...)`, scans each `PrintObjectRegions::LayerRangeRegions`, uses `find_volume_extents(layer_range, volume)` as the per-layer presence gate, and for `volume.is_model_part()` appends a `VolumeRegion` with the model volume, parent `-1`, a region from `get_create_region(region_config_from_model_volume(...))`, and the found bbox pointer. Required context comes from M314-M316's staged print-object-region shell, call boundary, and region-set helper, plus `Print.hpp:229-240` `VolumeRegion` structure and `PrintApply.cpp:542-546` model-volume eligibility. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region-construction pipeline.

## Exit criteria

- Add private staged behavior for the model-part branch of `PrintApply.cpp:1012-1024`.
- Preserve model-volume iteration order.
- Preserve filtering to model-part, negative-volume, and parameter-modifier volume types before the model-part branch.
- Preserve per-layer `find_volume_extents(...)` gating: append only to layers that contain the model volume extent.
- Preserve `volume.is_model_part()` branch behavior: append a volume region with parent `-1`, a non-null region id from the staged region-set helper, and the found bbox/extent identity.
- Preserve region reuse across layers/volumes when the staged derived region config is equal.
- Keep the implementation in private staged `ares-core` modules and split away from `generate_regions_state.rs` if needed to keep Rust files below 400 LOC.
- Add tests for model-volume order, per-layer extent gating, skipping unsupported volume types, negative/modifier deferral, region reuse for equal configs, distinct regions for config differences, and parent/bbox fields.
- Defer negative-volume branch from `PrintApply.cpp:1025-1027`, modifier branch from `PrintApply.cpp:1028-1054`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `region_config_from_model_volume(...)`, real `PrintRegionConfig`, real `PrintRegion`, real `ModelVolumePtrs`, real bbox pointers, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
