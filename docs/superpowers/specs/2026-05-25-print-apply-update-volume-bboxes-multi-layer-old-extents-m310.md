# Spec: M310 PrintApply update_volume_bboxes multi-layer old extents

## Goal

Port OrcaSlicer's multi-layer `update_volume_bboxes(...)` old-extents setup into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:908-917`: in the multi-layer branch, initialize `volumes_old`; when cached ids are empty, clear each `layer_range.volumes`; otherwise reserve/capture each layer range's old volumes with `std::move(layer_range.volumes)`.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`: M308 owns sorted eligible volume ordering and final cached-id refresh.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:895-907`: M309 stages the single-layer old/new extent behavior.
- Existing staged `StagedVolumeExtents` models volume id plus bbox.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a private staged layer range volume-cache record owning a `Vec<StagedVolumeExtents>`.
- Add a helper that mutates staged layer ranges and returns captured old extents for the multi-layer branch.
- If `cached_volume_ids` is empty, clear every layer's current volumes and return an empty captured-old-extents list.
- If `cached_volume_ids` is non-empty, capture every layer's current volumes into a returned `Vec<Vec<StagedVolumeExtents>>` in layer order and leave each layer's current volumes empty for later output population.
- Preserve empty input and empty per-layer volumes without special-case fallback behavior.
- Do not perform layer-height range expansion, cached multi-layer reuse, uncached bbox generation, final cache-id refresh, real meshes, transforms, real `LayerRangeRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Empty cached ids clear all layer volumes and return no captured old extents.
- Non-empty cached ids capture old volumes for every layer and clear layer outputs.
- Empty layer-range input returns empty captured old extents.
- Non-empty cached ids preserve empty per-layer volume lists in the captured output.
- Captured old extents preserve layer order and per-layer volume order.

## Migration note

This milestone stages only `PrintApply.cpp:908-917`. Later milestones must continue with range expansion at `PrintApply.cpp:919-927`, cached multi-layer extent reuse at `PrintApply.cpp:928-936`, and uncached bbox generation/insertion at `PrintApply.cpp:937-941` as separate source-cited rewrite slices.
