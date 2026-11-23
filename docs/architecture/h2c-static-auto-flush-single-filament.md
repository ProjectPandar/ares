# Static multi-nozzle Auto For Flush with one filament

## Upstream boundary

OrcaSlicer v2.5.0 `libslic3r/GCode/ToolOrdering.cpp:1506-1526`
(`build_nozzle_groups`) expands the configured counts when detailed nozzle
statistics are absent. `get_recommended_filament_maps` (`:1752-1769,1807-1856`)
builds a nozzle-centric context and dispatches Auto For Flush to
`FilamentGroup::calc_filament_group`; it does not use the Manual branch.
`libslic3r/FilamentGroup.cpp:1090-1105,1314-1318,955-976` routes the flush
request to enumeration for small cases. `:272-415` ranks the candidate maps
by printability and capacity before score, including the master preference;
with one filament there is no inter-filament flush. Physical indices in the
result refer to the flattened nozzle list, not to extruder indices.

`ToolOrdering.cpp:1682-1745,1842-1856` applies the master-extruder preference
where a swap is allowed, then compares the preferred and original maps
against printer inventory. With no loaded-filament information, the first
candidate wins a tie. `:1859-1876,3084-3091` materializes the result and
updates the filament, volume and nozzle maps through
`Print::update_filament_maps_to_config` (`libslic3r/Print.cpp:4661`).

## Rust destination and scope

Port the single-filament static case in the existing
`crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/`
boundary. `context.rs` presently rejects Auto For Flush when an extruder has
multiple nozzles. The two-head automatic enumerator and master swap in
`mod.rs::apply` are a compatibility shell for one-nozzle-per-extruder
contexts, not a physical-nozzle assignment algorithm. Reuse the physical
nozzle list already built by `context/inventory.rs` and pass selected nozzle
IDs through `apply`'s existing map publication and downstream nozzle
consumers. Do not route automatic grouping through the Manual solver or
make a decision based on the printer name or nozzle diameter.

Include two extruders, one configured and used model filament, homogeneous
nozzle volume per extruder, count-based nozzle inventory, and no loaded
filaments or dynamic remapping. Keep the upstream ranking and permissible
master preference for that case; do not unconditionally select the master
or substitute the logical extruder index for the physical nozzle ID.
Manual and existing one-nozzle-per-extruder behavior stay unchanged.
Defer multiple-filament optimization, hybrid and reported nozzle inventory,
loaded-filament matching, dynamic maps, and unsupported print sequences.

## CLI verification

Use a synthetic saved project with the stock H2C two-extruder configuration,
counts `1,6`, and one PLA object. Slice plate 1 with identical argv in Ares
and Native under official and seeded references. Require both to emit
nonempty G-code, and compare the emitted `filament_map`,
`filament_volume_map`, and `filament_nozzle_map`. For the stock configuration
with extruder 2 preferred they are `2`, `0`, `1` respectively. With only the
master changed to extruder 1 they are `1`, `0`, `0`; compare both cases to
Native so a fixed stock assignment cannot satisfy the test. Retain the
existing Manual and single-nozzle automatic parity coverage.
