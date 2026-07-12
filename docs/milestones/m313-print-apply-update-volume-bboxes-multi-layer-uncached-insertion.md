# M313: PrintApply update_volume_bboxes multi-layer uncached insertion

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the uncached-volume insertion branch in the multi-layer `update_volume_bboxes(...)` loop at `OrcaSlicer/src/libslic3r/PrintApply.cpp:937-941`: for an eligible model volume whose id is not present in `cached_volume_ids`, compute per-layer range bboxes with `transformed_its_bboxes_in_z_ranges(...)`, then append `{ model_volume->id(), bbox.first }` only to layers whose corresponding bbox is populated. Required context comes from M308's sorted eligible model-volume/cache-id shell, M310's cleared/captured multi-layer outputs, M311's expanded layer ranges, M312's cached branch, and existing staged range-bbox data from `staged_transformed_its_bboxes_in_z_ranges(...)`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned bbox, cache, or slicing pipeline.

## Exit criteria

- Add private staged multi-layer uncached extent insertion for `PrintApply.cpp:937-941`.
- Preserve processing only solid-or-modifier model volumes.
- Preserve that cached model-volume ids are skipped by this slice.
- Preserve appending supplied per-layer bboxes only when the corresponding range bbox is populated.
- Preserve doing nothing for unpopulated per-layer bboxes.
- Preserve model-volume loop order and per-layer append order.
- Preserve existing per-layer output prefixes.
- Add tests for uncached insertion across populated layers, unpopulated layer skip, cached id deferral to M312, non-eligible filtering, existing output prefix/order preservation, and duplicate uncached visits.
- Defer full integration with real `ModelVolumePtrs`, real mesh/transform/matrix orchestration, final cache-id refresh already staged in M308, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
