# M284: PrintApply keep reusable cached volume ids

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `print_objects_regions_invalidate_keep_some_volumes(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:664-695`, with model-volume sorting context from `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`, `ObjectID` ordering context from `OrcaSlicer/src/libslic3r/ObjectID.hpp:20-37`, `PrintObjectRegions::all_regions` / `cached_volume_ids` storage context from `OrcaSlicer/src/libslic3r/Print.hpp:291-296`, `ModelVolumeType` context from `OrcaSlicer/src/libslic3r/Model.hpp:340-348`, and M279 predicate context from `PrintApply.cpp:542-546`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned invalidation pipeline.

## Exit criteria

- Add a private staged helper equivalent to the cache-retention portion of `print_objects_regions_invalidate_keep_some_volumes(...)`.
- Preserve clearing staged `all_regions` before volume matching.
- Preserve sorting old and new volume inputs by id before matching.
- Preserve filtering new volumes through `model_volume_solid_or_modifier(...)` semantics from M279.
- Preserve monotonic `i_old` matching: advance old volumes until `old.id >= new.id`, then only consider equal ids.
- Preserve transform reuse test through an injected/staged transform-equivalence flag rather than real Eigen matrix comparison.
- Preserve cached-volume retention order by scanning `cached_volume_ids` forward until the matched id, asserting the id is present, writing kept ids compactly at the front, and truncating after the last kept id.
- Preserve skipping unmatched, non-solid/modifier, and transform-changed volumes.
- Add tests for all-regions clearing, sorted old/new matching, non-solid filtering, transform-changed skip, missing cached id panic, and compacting/truncating kept cached ids in upstream order.
- Defer real `PrintObjectRegions`, `PrintRegion`, `ModelVolume` pointers, Eigen `isApprox`, bbox recomputation, layer-range region rebuilding, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
