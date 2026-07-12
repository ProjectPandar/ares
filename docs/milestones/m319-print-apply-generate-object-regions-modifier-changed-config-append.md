# M319: PrintApply generate_print_object_regions modifier changed-config append

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1038-1042`: inside the modifier parent scan, when `region_config_from_model_volume(parent_region.region->config(), nullptr, volume, num_extruders)` differs from the parent region config, `generate_print_object_regions(...)` sets `added = true` and appends a modifier `VolumeRegion` with the current modifier volume, the scanned parent region id, a region from `get_create_region(std::move(config))`, and the current modifier bbox. Required context comes from M316's staged region-set helper, M318's staged intersecting parent candidates, `Print.hpp:229-240` `VolumeRegion`, and `PrintApply.cpp:542-546` model-volume eligibility. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region-construction pipeline.

## Exit criteria

- Add private staged behavior for the changed-config append branch of `PrintApply.cpp:1038-1042`.
- Preserve modifier-only entry and no-op for non-modifier current inputs.
- Preserve candidate order from the M318 parent scan output.
- Preserve changed-config gating: append only when the staged derived modifier config differs from the parent region config.
- Preserve `added = true` if at least one changed-config append is produced; keep it false otherwise.
- Preserve appended parent region id, region id creation/reuse through the staged region-set helper, current modifier volume id, and current modifier bbox identity.
- Keep unchanged-config fallback parent-model-part handling from `PrintApply.cpp:1043-1050` deferred.
- Keep the implementation in private staged `ares-core` modules and split away from existing modules if needed to keep Rust files below 400 LOC.
- Add tests for changed-config append, unchanged no-op, multiple candidates in scan order, region reuse for equal derived configs, distinct regions for distinct configs, non-modifier no-op, and bbox/parent preservation.
- Defer fallback parent-model-part selection and unchanged modifier append from `PrintApply.cpp:1043-1050`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `region_config_from_model_volume(...)`, real `PrintRegionConfig`, real `PrintRegion`, real `ModelVolumePtrs`, real bbox pointers, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
