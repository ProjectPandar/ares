# Spec: M321 PrintApply generate_print_object_regions painted region append

## Goal

Port the painted-region append loop from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1056-1067` into `ares-core` private staged state.

## Upstream source mapping

```cpp
// Finally add painting regions.
for (PrintObjectRegions::LayerRangeRegions &layer_range : layer_ranges_regions) {
    for (unsigned int painted_extruder_id : painting_extruders)
        for (int parent_region_id = 0; parent_region_id < int(layer_range.volume_regions.size()); ++ parent_region_id)
            if (const PrintObjectRegions::VolumeRegion &parent_region = layer_range.volume_regions[parent_region_id];
                parent_region.model_volume->is_model_part() || parent_region.model_volume->is_modifier()) {
                PrintRegionConfig cfg = parent_region.region->config();
                cfg.wall_filament.value    = painted_extruder_id;
                cfg.solid_infill_filament.value = painted_extruder_id;
                cfg.sparse_infill_filament.value       = painted_extruder_id;
                layer_range.painted_regions.push_back({ painted_extruder_id, parent_region_id, get_create_region(std::move(cfg))});
            }
```

The Rust staging must model:

- each layer's existing staged volume regions,
- parent volume type eligibility,
- parent config key and derived painted config key,
- painted extruder ids,
- appended painted region fields and region-set creation/reuse.

## Non-goals / deferred behavior

- Do not implement painted-region sorting from `PrintApply.cpp:1068-1072`.
- Do not implement fuzzy painted construction from `PrintApply.cpp:1075-1101`.
- Do not implement real Orca `PrintRegionConfig`, real config merge/diff, real model volume pointers, real `PrintRegion` pointers, slicing, G-code, profile loading, public API wiring, UI, OpenGL, filesystem, or terminal behavior.
- Do not create new crates, dependencies, or public APIs.
- Do not design an Ares-owned pipeline; this is only a source-cited `libslic3r` rewrite slice.

## Acceptance criteria

- Eligible model-part and modifier parents produce painted regions for each painted extruder.
- Ineligible parent volume types are skipped.
- Appends preserve source nested iteration order.
- Derived painted config sets wall, solid infill, and sparse infill filament keys to the extruder id while retaining the parent config identity marker.
- Equal derived painted configs reuse one staged region id through `StagedGenerateRegionSet`.
- Empty painting extruders and empty parent volume regions are no-ops.
- All new symbols stay private to `ares-core` staged modules.
