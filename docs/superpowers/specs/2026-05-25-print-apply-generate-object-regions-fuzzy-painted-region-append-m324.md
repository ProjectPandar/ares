# Spec: M324 PrintApply generate_print_object_regions fuzzy painted-region append

## Goal

Port the fuzzy painted append loop for painted-region parents from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1089-1095` into `ares-core` private staged state.

## Upstream source mapping

```cpp
for (int parent_painted_regions_id = 0; parent_painted_regions_id < int(layer_range.painted_regions.size()); ++parent_painted_regions_id) {
    const PrintObjectRegions::PaintedRegion &parent_painted_region = layer_range.painted_regions[parent_painted_regions_id];

    PrintRegionConfig cfg = parent_painted_region.region->config();
    if (cfg.fuzzy_skin.value != FuzzySkinType::Disabled_fuzzy) cfg.fuzzy_skin.value = FuzzySkinType::All;
    layer_range.fuzzy_skin_painted_regions.push_back({FuzzySkinParentType::PaintedRegion, parent_painted_regions_id, get_create_region(std::move(cfg))});
}
```

The Rust staging must model:

- the enclosing `has_painted_fuzzy_skin` gate,
- a painted-region parent's staged config marker plus fuzzy-skin value,
- parent type `PaintedRegion`, parent painted-region index, and region id,
- region-set creation/reuse for the derived fuzzy config.

## Non-goals / deferred behavior

- Do not implement fuzzy painted sorting from `PrintApply.cpp:1097-1100`.
- Do not implement real Orca `PrintRegionConfig`, real config merge/diff, real model volume pointers, real `PrintRegion` pointers, slicing, G-code, profile loading, public API wiring, UI, OpenGL, filesystem, or terminal behavior.
- Do not create new crates, dependencies, or public APIs.
- Do not design an Ares-owned pipeline; this is only a source-cited `libslic3r` rewrite slice.

## Acceptance criteria

- When `has_painted_fuzzy_skin` is false, no fuzzy regions are appended and no staged regions are created.
- Painted-region parents produce fuzzy regions in source order.
- Non-disabled parent fuzzy skin values become `All`; disabled values remain disabled.
- Appends preserve parent painted-region index and parent type `PaintedRegion`.
- Equal derived fuzzy configs reuse one staged region id through `StagedGenerateRegionSet`.
- Empty painted-region parents are a no-op.
- All new symbols stay private to `ares-core` staged modules.
