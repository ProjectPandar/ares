# Spec: M325 PrintApply generate_print_object_regions fuzzy painted-region sort

## Goal

Port the fuzzy painted-region sort comparator from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1097-1100` into `ares-core` private staged state.

## Upstream source mapping

```cpp
// Sort the regions by parent region::print_object_region_id() to help the slicing algorithm when applying fuzzy skin segmentation.
std::sort(layer_range.fuzzy_skin_painted_regions.begin(), layer_range.fuzzy_skin_painted_regions.end(), [&layer_range](auto &l, auto &r) {
    return l.parent_print_object_region_id(layer_range) < r.parent_print_object_region_id(layer_range);
});
```

Parent id resolution comes from `OrcaSlicer/src/libslic3r/Print.cpp:4932-4947`:

```cpp
if (this->parent_type == FuzzySkinParentType::PaintedRegion) {
    return layer_range.painted_regions[this->parent].region;
}

assert(this->parent_type == FuzzySkinParentType::VolumeRegion);
return layer_range.volume_regions[this->parent].region;
```

The Rust staging must model:

- fuzzy parent type `VolumeRegion` or `PaintedRegion`,
- each fuzzy region's parent index,
- each volume parent and painted parent staged `print_object_region_id`,
- sorted fuzzy-region output preserving all fuzzy-region fields.

## Non-goals / deferred behavior

- Do not implement real Orca `PrintRegion` pointers or real `parent_print_object_region(...)`.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.

## Acceptance criteria

- Sort fuzzy painted regions by resolved parent print-object region id ascending.
- Resolve `VolumeRegion` parents through the volume-region parent list.
- Resolve `PaintedRegion` parents through the painted-region parent list.
- Preserve all fuzzy painted-region fields while reordering.
- Empty and single-entry lists are no-ops.
- Sorting is scoped to the supplied layer range data only.
- All new symbols stay private to `ares-core` staged modules.
