# M320: PrintApply generate_print_object_regions modifier unchanged fallback

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1043-1050`: while scanning intersecting modifier parents, if the derived modifier config is unchanged and `parent_model_part_id == -1` and the parent volume is a model part, remember that parent region index; after the scan, if no changed-config append was added and a model-part parent was found, append an unchanged modifier `VolumeRegion` with the current modifier volume, that parent id, the parent region's existing region pointer, and the current modifier bbox. Required context comes from M318's staged parent scan candidate order, M319's changed-config `added` result, `Print.hpp:229-240` `VolumeRegion`, and the `ModelVolume::is_model_part()` / `is_modifier()` predicates cited by earlier milestones. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region-construction pipeline.

## Exit criteria

- Add private staged behavior for the unchanged fallback branch of `PrintApply.cpp:1043-1050`.
- Preserve model-part parent selection only when the candidate's derived config equals the parent config.
- Preserve first eligible model-part parent only: once `parent_model_part_id` is set, later unchanged model-part candidates do not replace it.
- Preserve fallback append only when M319 produced no changed-config append (`added == false`) and a model-part parent was selected.
- Preserve no fallback append when any changed-config append already occurred.
- Preserve appended current modifier volume id, selected parent region index, reused parent region index, and current modifier bbox identity.
- Preserve modifier-only entry and no-op for non-modifier current inputs.
- Keep the implementation in private staged `ares-core` modules and split away from existing modules if needed to keep Rust files below 400 LOC.
- Add tests for selecting the first unchanged model-part parent, skipping modifier parents for fallback selection, no fallback after changed append, no fallback without a selected model-part parent, non-modifier no-op, and parent/region/bbox identity.
- Defer painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `region_config_from_model_volume(...)`, real `PrintRegionConfig`, real `PrintRegion`, real `ModelVolumePtrs`, real bbox pointers, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
