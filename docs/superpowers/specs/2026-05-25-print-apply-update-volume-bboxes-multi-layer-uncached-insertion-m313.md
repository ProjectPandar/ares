# Spec: M313 PrintApply update_volume_bboxes multi-layer uncached insertion

## Goal

Port OrcaSlicer's multi-layer `update_volume_bboxes(...)` uncached per-layer bbox insertion branch into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:937-941`: for an eligible uncached `model_volume`, call `transformed_its_bboxes_in_z_ranges(...)`, then for every `layer_range`, append a `VolumeExtents` with the current volume id and `bbox.first` only when the corresponding `bbox.second` populated flag is true.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`: M308 owns sorted eligible volume ordering and final cached-id refresh.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:908-917`: M310 stages multi-layer old-volume setup and output clearing/capture.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:919-927`: M311 stages expanded ranges used to compute bboxes.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:928-936`: M312 stages cached old-extent reuse.
- Existing staged `StagedRangeBoundingBox3f` models the `{ BoundingBox, bool }` result of `transformed_its_bboxes_in_z_ranges(...)`.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a helper that mutates staged multi-layer cache layers by appending supplied per-layer range bboxes for uncached eligible model volumes.
- Process only model volumes whose type passes staged `model_volume_solid_or_modifier(...)`.
- Process only model volumes whose id is absent from `cached_volume_ids`; cached reuse remains owned by M312.
- For each uncached eligible model volume, use the corresponding per-layer bbox list and append only populated bboxes to matching layer indexes.
- Preserve existing layer output prefixes, model-volume order, per-layer order, and duplicate uncached model-volume visits.
- Do not implement public APIs, profile loading, UI, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Uncached eligible ids append populated range bboxes across matching layers.
- Unpopulated per-layer bboxes are skipped without fallback insertion.
- Cached ids append nothing in this slice.
- Non-solid-or-modifier volume types are filtered out even when bboxes are supplied.
- Existing layer outputs are preserved as prefixes and new extents append in model-volume order.
- Duplicate uncached ids are processed as independent visits.

## Migration note

This milestone stages only `PrintApply.cpp:937-941`. Later milestones may integrate the staged multi-layer pieces into a larger private helper, but must still remain source-cited and avoid public pipeline/API expansion until a milestone names the relevant upstream boundary.
