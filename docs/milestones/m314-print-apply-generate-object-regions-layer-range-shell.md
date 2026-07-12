# M314: PrintApply generate_print_object_regions layer-range shell

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the object reuse/new allocation and layer-range shell at `OrcaSlicer/src/libslic3r/PrintApply.cpp:953-993`: `generate_print_object_regions(...)` wraps an old `PrintObjectRegions` or creates a new one, clears `all_regions`, detects reusable old layer ranges, asserts layer-range count and exact `layer_height_range` matches, refreshes each reused range config pointer, clears `volume_regions`, `painted_regions`, and `fuzzy_skin_painted_regions`, and otherwise stores `trafo_bboxes`, reserves layer ranges, and creates fresh layer-range entries from `model_layer_ranges`. Required structure context comes from `OrcaSlicer/src/libslic3r/Print.hpp:271-296` for `LayerRangeRegions`, `all_regions`, `layer_ranges`, `trafo_bboxes`, and `cached_volume_ids`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print-object-region pipeline.

## Exit criteria

- Add private staged `generate_print_object_regions` layer-range shell behavior for `PrintApply.cpp:953-993`.
- Preserve clearing `all_regions` for both reused and fresh outputs.
- Preserve reuse detection only when an old object exists and its layer-range list is non-empty.
- Preserve assertion behavior for reused layer-range count mismatch and `layer_height_range` mismatch.
- Preserve reused range config refresh and clearing of `volume_regions`, `painted_regions`, and `fuzzy_skin_painted_regions`.
- Preserve reused existing `volumes`, cached volume ids, and old `trafo_bboxes` for later update-volume-bboxes reuse.
- Preserve fresh path assigning the input `trafo_bboxes` and creating layer ranges from `model_layer_ranges`.
- Add tests for fresh creation, old empty ranges taking the fresh path, reuse clearing mutable region lists, reused config refresh, preserving reused volumes/cache/old transform, all-regions clearing, count mismatch panic, and range mismatch panic.
- Defer `is_mm_painted` / `update_volume_bboxes(...)` invocation from `PrintApply.cpp:995-996`, `get_create_region` from `PrintApply.cpp:998-1010`, volume-region construction from `PrintApply.cpp:1012-1054`, painting/fuzzy painting construction from `PrintApply.cpp:1056-1101`, real `PrintObjectRegions`, real configs and transforms, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
