# Spec: M315 PrintApply generate_print_object_regions update_volume_bboxes call

## Goal

Port OrcaSlicer's `generate_print_object_regions(...)` MM-painted offset selection and `update_volume_bboxes(...)` call boundary into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:995-996`: compute `is_mm_painted = num_extruders > 1 && any model_volume->is_mm_painted()`, then call `update_volume_bboxes(layer_ranges_regions, out->cached_volume_ids, model_volumes, out->trafo_bboxes, is_mm_painted ? 0.f : std::max(0.f, xy_contour_compensation))`.

Required context:
- M314 stages `generate_print_object_regions(...)` output shell with layer ranges, cached volume ids, and `trafo_bboxes`.
- M308-M313 stage the internal `update_volume_bboxes(...)` pieces; M315 records the call boundary and offset semantics without integrating real model meshes/transforms.
- `OrcaSlicer/src/libslic3r/Model.hpp:1014` defines `ModelVolume::is_mm_painted()` as non-empty MMU segmentation facets; M315 represents that as a staged boolean.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a lightweight staged model-volume paint record with id and `is_mm_painted` boolean for this call-boundary slice.
- Add a helper that receives a staged print-object-region shell, staged model volumes, `num_extruders`, and `xy_contour_compensation`, then returns a call record describing the `update_volume_bboxes(...)` invocation.
- Compute `is_mm_painted` exactly as upstream: `num_extruders > 1 && any(volume.is_mm_painted)`.
- Select `offset` exactly as upstream: `0.0` if MM-painted, otherwise `xy_contour_compensation.max(0.0)`.
- Preserve the shell's `trafo_bboxes`, cached volume ids, and layer-range count in the call record.
- Do not perform real bbox updating, model-volume sorting, mesh/transform orchestration, region creation, painted/fuzzy construction, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Single extruder with a painted volume is not MM-painted and clamps negative compensation to zero.
- Multiple extruders with no painted volumes use clamped positive compensation.
- Multiple extruders with any painted volume set offset to zero.
- Empty model-volume input is not MM-painted.
- The call record preserves `trafo_bboxes`, cached volume ids, and layer-range count from the staged shell.

## Migration note

This milestone stages only `PrintApply.cpp:995-996`. Later milestones must continue with `get_create_region` at `PrintApply.cpp:998-1010` and volume-region construction at `PrintApply.cpp:1012-1054` as separate source-cited rewrite slices.
