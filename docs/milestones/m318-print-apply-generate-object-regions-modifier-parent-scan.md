# M318: PrintApply generate_print_object_regions modifier parent scan

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1028-1037`: modifier branch entry, `added = false`, `parent_model_part_id = -1`, reverse scan of existing volume regions, model-part/modifier parent eligibility, `find_modifier_volume_extents(...)`, and bbox intersection gating. Defer `PrintApply.cpp:1038-1050` modifier config/append behavior and later painted/fuzzy construction.

## Exit criteria

- Preserve modifier-only entry and non-modifier no-op.
- Preserve initial `added = false` and `parent_model_part_id = -1`.
- Preserve descending parent scan order.
- Preserve model-part/modifier parent eligibility.
- Preserve index-stable adapter for staged `find_modifier_volume_extents(...)`.
- Preserve intersection gating.
