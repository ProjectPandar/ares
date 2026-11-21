# Type1 ramming nozzle-change markers

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `libslic3r/GCode/WipeTower.cpp:3623,3651–3655,3681,3802` (`WipeTower::ramming`) formats and brackets the outgoing ramming stream with `; NOZZLE_CHANGE_START OF%d NF%d ON%d NN%d` and the corresponding `; NOZZLE_CHANGE_END OF%d NF%d ON%d NN%d`. Both lines carry the same tuple. `OF` and `NF` are the zero-based logical filament IDs passed to `ramming`; `ON` and `NN` come from `get_nozzle_id(filament, m_cur_layer_id)`. `WipeTower.cpp:5561` delegates to `LayeredNozzleGroupResult`; `MultiNozzleUtils.cpp:454–463` returns the nozzle group's global ID (or `-1` when unmapped), not the extruder index or a tool command number.
- `libslic3r/GCode.cpp:1231–1257` transforms and inserts that stream; it does not generate the markers. `libslic3r/GCode/GCodeProcessor.cpp:189–190` defines the standalone tags.

## Rust destination and scope

Replace the two temporary comments in `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/nozzle_change.rs`. Retain the existing cross-physical-head invocation in `type1/tool_change.rs`. Use its old/new filament slots and resolve nozzle IDs using the static grouping result in `filament_nozzle_map` through `motion::tool_selection::nozzle`. Despite its name, `Plan::nozzle_map` is populated from the one-based *extruder assignment* `filament_map`; it is not the source of the marker's `ON` or `NN`. Do not introduce layered grouping or change when the stream is invoked.

Base: `8b2829635`. No dependency on the nozzle-change acceleration or ramming row-spacing changes: first and last markers bracket existing ramming output regardless of those commands. No ramming movement, acceleration, tower routing, or template serialization changes are included.

## Processor follow-up

`GCodeProcessor.cpp:1662–1674,1746–1762` parses the tuple, marks the outgoing `ExtruderUsageBlcok` at START, closes its nozzle/filament interval at END, and opens the next block with the incoming nozzle and filament. `:2027–2043` supplies those alongside filament blocks to the pre-cooling/pre-heating injector. This is not a planner command or an input to the initial move-time estimate. The second pass uses timing already recorded; segmentation can affect placement of pre-cooling/pre-heating and resulting output line IDs.

Ares `processor/pre_heating.rs::UsageBlocks::line` scans tags and filament selections (`T`, `;VT`, `M1020`), but has neither this tag consumer nor the extruder usage-block model. `processor/pre_heating/injector.rs::Injector::new` currently builds free intervals from filament blocks. Porting the extruder usage-block lifecycle and reconciling the injector's two block sources is deferred; this slice does not manufacture approximate boundaries or change estimate accounting. Existing timing or preheat differences are not attributed to the writer spelling.

## Public behavior

Slice a synthetic two-slot, arc-enabled H2D project with both producers. Across heads, each must report exactly one START/END pair with native field order and values, bracketing outgoing extrusion before selection of the incoming filament. The same physical head and disabled tower are negative controls; the same-head case still performs the logical change with the tower enabled. Compare complete marker lines without normalizing them. The witness checks placement, not equality of neighboring motions owned by other ports.
