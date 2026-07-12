# Spec: M323 PrintApply generate_print_object_regions fuzzy volume-region append

## Goal

Port the fuzzy painted volume-region parent append loop from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1075-1086` into `ares-core` private staged state.

## Upstream source mapping

```cpp
if (has_painted_fuzzy_skin) {
    using FuzzySkinParentType = PrintObjectRegions::FuzzySkinPaintedRegion::ParentType;

    for (PrintObjectRegions::LayerRangeRegions &layer_range : layer_ranges_regions) {
        // FuzzySkinPaintedRegion can override different parts of the Layer than PaintedRegions,
        // so FuzzySkinPaintedRegion has to point to both VolumeRegion and PaintedRegion.
        for (int parent_volume_region_id = 0; parent_volume_region_id < int(layer_range.volume_regions.size()); ++parent_volume_region_id) {
            if (const PrintObjectRegions::VolumeRegion &parent_volume_region = layer_range.volume_regions[parent_volume_region_id]; parent_volume_region.model_volume->is_model_part() || parent_volume_region.model_volume->is_modifier()) {
                PrintRegionConfig cfg = parent_volume_region.region->config();
                if (cfg.fuzzy_skin.value != FuzzySkinType::Disabled_fuzzy) cfg.fuzzy_skin.value = FuzzySkinType::All;
                layer_range.fuzzy_skin_painted_regions.push_back({FuzzySkinParentType::VolumeRegion, parent_volume_region_id, get_create_region(std::move(cfg))});
            }
        }
```

The Rust staging must model:

- the `has_painted_fuzzy_skin` gate,
- each parent volume region's type and fuzzy config,
- parent type `VolumeRegion`, parent index, and region id,
- region-set creation/reuse for the derived fuzzy config.

## Non-goals / deferred behavior

- Do not implement painted-region parent fuzzy append from `PrintApply.cpp:1089-1095`.
- Do not implement fuzzy painted sorting from `PrintApply.cpp:1097-1100`.
- Do not implement real Orca `PrintRegionConfig`, real config merge/diff, real model volume pointers, real `PrintRegion` pointers, slicing, G-code, profile loading, public API wiring, UI, OpenGL, filesystem, or terminal behavior.
- Do not create new crates, dependencies, or public APIs.
- Do not design an Ares-owned pipeline; this is only a source-cited `libslic3r` rewrite slice.

## Acceptance criteria

- When `has_painted_fuzzy_skin` is false, no fuzzy regions are appended.
- Eligible model-part and modifier volume parents produce fuzzy regions.
- Ineligible parent volume types are skipped.
- Non-disabled parent fuzzy skin values become `All`; disabled values remain disabled.
- Appends preserve parent volume-region order.
- Equal derived fuzzy configs reuse one staged region id through `StagedGenerateRegionSet`.
- Empty parent volume regions are a no-op.
- All new symbols stay private to `ares-core` staged modules.
