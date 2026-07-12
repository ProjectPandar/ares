# Spec: M308 PrintApply update_volume_bboxes volume order/cache ids

## Goal

Port OrcaSlicer's `update_volume_bboxes(...)` model-volume sorting/filtering and cached-volume-id refresh into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893`: `update_volume_bboxes(...)` begins by sorting `model_volumes` by id and then processes model volumes only when `model_volume_solid_or_modifier(...)` is true.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:946-950`: after bbox processing, clear `cached_volume_ids`, reserve model-volume capacity, and append ids for sorted solid-or-modifier volumes.

Required context:
- Existing staged model-volume type handling maps Orca solid/modifier eligibility: model part, negative volume, and parameter modifier are accepted; support blocker, support enforcer, and invalid are rejected.
- Existing `volume_cache_state.rs` owns staged cache-related helpers for `PrintApply` volume cache behavior.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Put M308 tests in a separate focused test module so existing Rust files stay below the 400 LOC split threshold.
- Add a private staged helper that accepts existing cached ids and model volumes, sorts model volumes by id, filters to solid-or-modifier volumes, and replaces cached ids with the filtered sorted ids.
- Return or expose the sorted solid-or-modifier model volumes/ids needed by later bbox milestones without computing bboxes in this milestone.
- Preserve duplicate ids if duplicate staged inputs are provided; do not invent deduplication.
- Preserve clearing stale cached ids when the new sorted solid-or-modifier list is empty.
- Do not perform single-layer bbox reuse, multi-layer bbox reuse, bbox computation, real meshes, transforms, real `ModelVolumePtrs`, real `LayerRangeRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Unsorted model volumes produce sorted refreshed cached ids.
- Non-solid-or-modifier volumes are filtered out of refreshed cached ids.
- Empty model volume input clears stale cached ids.
- Already sorted eligible volumes preserve order.
- Duplicate ids are preserved after sorting.
- Stale cached ids are replaced by the current eligible sorted ids.

## Migration note

This milestone stages only the ordering/filtering/cache-id shell of `update_volume_bboxes(...)`. Later milestones must continue with single-layer cached bbox reuse from `PrintApply.cpp:895-907` and multi-layer bbox behavior from `PrintApply.cpp:908-941` as separate source-cited rewrite slices.
