# Spec: M320 PrintApply generate_print_object_regions modifier unchanged fallback

## Goal

Port OrcaSlicer's `generate_print_object_regions(...)` unchanged modifier fallback branch into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:1043-1050`: during modifier parent scanning, if a candidate's derived config is equal to its parent config and no model-part fallback parent has been recorded yet, record the candidate id when the parent volume is a model part; after scanning, if no changed-config append was added and a fallback model-part parent exists, append `{ &volume, parent_model_part_id, layer_range.volume_regions[parent_model_part_id].region, bbox }`.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:1028-1037` is staged by M318 and supplies intersecting parent candidates in descending scan order.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:1038-1042` is staged by M319 and supplies the changed-config `added` decision.
- `OrcaSlicer/src/libslic3r/Print.hpp:229-240` defines `VolumeRegion` fields: model volume, parent, region, bbox, and previous-same-region pointer.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a focused private staged module for unchanged modifier fallback.
- Add a staged current modifier input carrying current volume id, volume type, and current modifier bbox.
- Add staged candidate input carrying parent region index, parent volume type, parent region index, parent config, and derived modifier config.
- Add staged fallback output carrying model volume id, parent region index, reused region id, and bbox.
- Add a helper that scans candidate inputs in M318/M319 candidate order.
- Return a result exposing selected `parent_model_part_id` and an optional appended fallback region.
- Select only unchanged model-part candidates where derived config equals parent config.
- Select only the first unchanged model-part candidate; do not replace it with later candidates.
- Append only when current type is `StagedModelVolumeType::ParameterModifier`, M319 `added` is false, and a fallback model-part parent was selected.
- Do not create a new region id; reuse the selected parent candidate's existing region id.
- Preserve current modifier bbox identity in the appended fallback record.
- Keep all M320 symbols private to staged `print_apply` modules.
- Do not perform real `region_config_from_model_volume(...)`, real config merging, painted/fuzzy construction, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Selects the first unchanged model-part candidate in candidate order and appends fallback when `added` is false.
- Skips unchanged modifier parents for fallback selection.
- Does not append fallback when a changed-config append already set `added` true.
- Does not append fallback when no unchanged model-part parent exists.
- Non-modifier current input returns no selected parent and no append.
- Appended fallback preserves current modifier volume id, selected parent region index, reused parent region index, and bbox identity.

## Migration note

This milestone stages only the unchanged fallback branch of `PrintApply.cpp:1043-1050`. Later milestones must continue with painted/fuzzy construction at `PrintApply.cpp:1056-1101` as separate source-cited rewrite slices.
