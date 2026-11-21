# MultiAsSingle per-layer tool changes

## Source boundary and destination

Port the MultiAsSingle branch of `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:584-600` and `:781-838`, together with `OrcaSlicer/src/libslic3r/CustomGCode.cpp:103-112`. The existing Rust owner is `ares-core`'s `project_slice::gcode_emit::tool_ordering`: `collect_layers` prepares `LayerTools`, `collect::object` records the tools used by each object layer, and `custom_gcodes` assigns layer events from `Project::curr_plate_custom_gcodes`.

## Included behavior

When the active machine has more than one filament, every object uses one distinct extruder, and the selected plate's custom-G-code mode is `MultiAsSingle`, collect `ToolChange` events as ordered `(print_z, one-based filament id)` switches. An event applies to its layer and later layers, with an out-of-range filament id mapped to the first filament as in `CustomGCode::custom_tool_changes`. The active switch replaces the object's configured printing extruders at and above that Z while preserving the existing layer-role and support collection rules. The event remains available to the existing custom-G-code assignment so the writer emits its `Tn` transition.

`ToolOrdering.cpp:837` retains the switch on `LayerTools` for export. The object tool buckets use it for walls and fills (`ToolOrdering.cpp:118-139, 166-175`); support and raft buckets keep their own selectors. `Print::object_extruders()` (`Print.cpp:689-703`) excludes support, so a support filament can share a layer without disabling the object override.

The mode decision uses the existing `Project::curr_plate_custom_gcodes` model accessor and the same object-extruder cardinality used by `ToolOrdering::assign_custom_gcodes`. A single-filament machine and a print whose objects use more than one extruder do not apply the override.

## Deferred behavior

Support-filament overrides, wipe-tower-specific ordering, sequential/by-object ordering, color-change conversion, and custom-G-code parsing are outside this slice. Existing custom-G-code event assignment remains unchanged except that a MultiAsSingle `ToolChange` is no longer discarded before writer emission.

## Verification

Add a public CLI parity case using identical Native and Ares arguments, a synthetic two-filament single-nozzle machine, one object assigned to filament one, and a `MultiAsSingle` `ToolChange` event to filament two. Compare exit status, streams, result report, and normalized complete G-code, including the emitted `T1`. Verify a switch back to filament one, an out-of-range switch, and a switch on layers shared with independently assigned support. Keep a single-filament control where no switch override applies. Run the focused cases in official and seeded parity environments, then the relevant paired tool-ordering and custom-G-code cases, formatting, all-target Clippy, and the WASM check.
