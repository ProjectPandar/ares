# M315: PrintApply generate_print_object_regions update_volume_bboxes call

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:995-996`: after the layer-range shell, `generate_print_object_regions(...)` computes `is_mm_painted` as `num_extruders > 1` and any model volume reporting `is_mm_painted()`, then calls `update_volume_bboxes(...)` using `out->trafo_bboxes` and an offset of `0.f` when multi-material painting is active, otherwise `std::max(0.f, xy_contour_compensation)`. Required context comes from M314's staged print-object-region shell and M308-M313's staged `update_volume_bboxes(...)` pieces. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned bbox update pipeline.

## Exit criteria

- Add private staged behavior for `PrintApply.cpp:995-996`.
- Preserve `is_mm_painted` as false when `num_extruders <= 1`, regardless of volume paint flags.
- Preserve `is_mm_painted` as true only when `num_extruders > 1` and at least one model volume is MM-painted.
- Preserve offset selection: `0.0` for MM-painted, otherwise clamp negative `xy_contour_compensation` to `0.0`.
- Preserve passing the shell's `trafo_bboxes` and cached volume ids through a staged update-volume-bboxes call record.
- Add tests for single-extruder painted volume, multi-extruder unpainted volumes, multi-extruder painted volume, negative compensation clamp, and preserving transform/cache/layer counts in the call record.
- Defer actual full `update_volume_bboxes(...)` orchestration over real `ModelVolumePtrs`, `get_create_region` from `PrintApply.cpp:998-1010`, volume-region construction from `PrintApply.cpp:1012-1054`, painting/fuzzy construction from `PrintApply.cpp:1056-1101`, real `PrintObjectRegions`, real configs/transforms, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
