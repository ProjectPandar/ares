# ToolOrdering: standalone thin-gap-fill filament

## Upstream boundary

This is a Rust rewrite slice of OrcaSlicer 2.5.0 source
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/Fill/Fill.cpp:1528–1537` wraps each thin-fill entity in
  its own `ExtrusionEntityCollection` and appends it to the region fills.
- `src/libslic3r/GCode/ToolOrdering.cpp:144–164`, `region_filament`,
  selects a collection's 1-based filament. Without infill, only a
  collection consisting entirely of inner-wall loops takes the inner-wall
  key; all other non-infill collections take the outer-wall key.
- `src/libslic3r/GCode/ToolOrdering.cpp:865–889`,
  `ToolOrdering::collect_extruders`, collects those actual region fills.

Rust destinations are
`project_slice/extrusion_islands/tool_regions.rs::{RegionToolUsage,collect}`
and `project_slice/gcode_emit/tool_ordering/collect.rs::object`.
`RegionToolUsage` remains a temporary presence-summary shell around upstream
collection decisions, not a separate slicing design. This slice replaces its
incorrect sparse classification of independent thin GapFill collections.

## Contract

An independent thin-fill Path or Loop whose role is GapFill is not sparse
infill. Its upstream wrapper selects `outer_wall_filament_id`, not
`sparse_infill_filament_id`. Record that fill-category presence separately
and collect its outer-wall filament with the other fill categories.

Do not represent it as ordinary perimeter presence: that path currently adds
both wall keys when multiple walls are configured. A standalone gap path must
not invent an inner-wall tool. Do not simply discard it either: its outer-wall
filament still belongs to a layer containing only those fills.

GapFill is overridable under `flush_into_objects`, not under
`flush_into_infill`; soluble-filament handling and the existing region-wide
non-overridable-fill gate must remain in force. Tool IDs stay 1-based until
the existing normalization boundary. Sorting, carry-forward ordering, tower
partitioning, and emission remain unchanged.

Other thin-fill roles retain their existing classification. Surface-fill
collection classification and ordinary perimeter classification are deferred;
this slice does not claim a complete port of every `region_filament` case.

## Non-negotiable boundaries

- No fill-generation, path-role, geometry, configuration inheritance,
  retraction, temperature, estimator, tower, or printer-specific changes.
- No dropped executable commands, time exclusions, rounded role maps,
  alternate oracle, relaxed tolerances, or comparator changes.
- Keep the original project untouched. Use a synthetic dual-filament project
  for the public CLI regression, with walls/solid fills on filament 1 and
  genuine sparse infill on filament 2.
- The regression must distinguish premature collection of filament 2 from
  its required later use; compare complete retained Native/API/CLI outputs
  through the existing strict seam in both Native modes.
- Core changes remain platform-neutral and do not add I/O or native APIs.

The observed first tool-membership defect does not by itself account for the
complete remaining time-cost difference.
