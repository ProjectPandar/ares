# Spec: M319 PrintApply generate_print_object_regions modifier changed-config append

## Goal

Port OrcaSlicer's `generate_print_object_regions(...)` modifier changed-config append branch into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:1038-1042`: for each intersecting parent from the modifier parent scan, derive a config from the parent region config and current modifier; if it differs from the parent config, set `added = true` and append `{ &volume, parent_region_id, get_create_region(std::move(config)), bbox }`.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:1028-1037` is staged by M318 and supplies intersecting parent candidates in descending scan order.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:998-1010` is staged by M316 and supplies region-set reuse/create behavior.
- `OrcaSlicer/src/libslic3r/Print.hpp:229-240` defines `VolumeRegion` fields: model volume, parent, region, bbox, and previous-same-region pointer.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:542-546` defines eligible model-volume types.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a focused private staged module for modifier changed-config append.
- Add a staged current modifier input carrying current volume id, volume type, and current modifier bbox.
- Add staged parent candidate input carrying parent region id, parent config, derived modifier config, and current modifier bbox context where needed.
- Add staged appended volume-region output carrying model volume id, parent region id, region id, and bbox.
- Add a helper that scans M318-style candidate inputs in source scan order.
- Return a result exposing `added` and appended regions.
- Append only for `StagedModelVolumeType::ParameterModifier` current inputs.
- Append only when derived config differs from the parent config.
- Use `StagedGenerateRegionSet::get_create_region(...)` so equal derived configs reuse a region id and distinct configs create distinct region ids.
- Preserve candidate order in appended output.
- Keep unchanged-config fallback parent-model-part handling deferred; do not append an unchanged modifier in M319.
- Keep all M319 symbols private to staged `print_apply` modules.
- Do not perform real `region_config_from_model_volume(...)`, real config merging, fallback parent selection, painted/fuzzy construction, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- A modifier current input with a changed derived config appends one region and sets `added = true`.
- An unchanged derived config appends nothing and keeps `added = false`.
- Multiple changed candidates append in candidate order.
- Equal derived configs reuse one region id across appended candidates.
- Distinct derived configs create distinct region ids.
- Non-modifier current input returns no appends and `added = false`.
- Appended records preserve current modifier volume id, parent region id, region id, and bbox identity.

## Migration note

This milestone stages only the changed-config append branch of `PrintApply.cpp:1038-1042`. Later milestones must continue with fallback parent-model-part selection and unchanged modifier append at `PrintApply.cpp:1043-1050`, then painted/fuzzy construction at `PrintApply.cpp:1056-1101` as separate source-cited rewrite slices.
