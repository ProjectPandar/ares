# Spec: M322 PrintApply generate_print_object_regions painted region sort

## Goal

Port the painted-region sort comparator from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1068-1072` into `ares-core` private staged state.

## Upstream source mapping

```cpp
// Sort the regions by parent region::print_object_region_id() and extruder_id to help the slicing algorithm when applying MM segmentation.
std::sort(layer_range.painted_regions.begin(), layer_range.painted_regions.end(), [&layer_range](auto &l, auto &r) {
    int lid = layer_range.volume_regions[l.parent].region->print_object_region_id();
    int rid = layer_range.volume_regions[r.parent].region->print_object_region_id();
    return lid < rid || (lid == rid && l.extruder_id < r.extruder_id); });
```

The Rust staging must model:

- a painted region's `parent` volume-region index,
- a painted region's `extruder_id`,
- each parent volume region's staged print object region id,
- sorted painted-region output.

## Non-goals / deferred behavior

- Do not implement fuzzy painted construction from `PrintApply.cpp:1075-1101`.
- Do not implement real Orca `PrintRegion` pointers or real `print_object_region_id()` lookup.
- Do not create new crates, dependencies, or public APIs.
- Do not design an Ares-owned pipeline; this is only a source-cited `libslic3r` rewrite slice.

## Acceptance criteria

- Sort painted regions by parent print object region id ascending.
- Break ties by extruder id ascending.
- Preserve all painted-region fields while reordering.
- Empty and single-entry lists are no-ops.
- Sorting is scoped to the supplied layer range data only.
- All new symbols stay private to `ares-core` staged modules.
