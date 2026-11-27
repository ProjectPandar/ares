# Belt multi-filament admission without active purge

Reference: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Native paths below are relative to `src/libslic3r/` unless noted. This extends
[belt purge option handling](belt-purge.md) without adding a purge planner.

## Admission and validation

Preserve the existing one printable object, one instance, one model-part
volume, by-layer belt boundary and its other feature checks (including no
support, raft, skirt, draft shield, spiral vase, or contoured toolpaths). Keep
the one-slot behavior unchanged. For multiple configured slots, admit ordinary
(non-mixed) filaments on one physical nozzle under the existing configuration
and selector validation. Valid resolved logical slots need not include slot
zero: one used slot, multiple used slots, and unused configured slots are all
possible. This does not expand belt admission to multiple physical nozzles,
virtual mixed filaments, or additional geometry authorship such as modifiers
or painted material assignments. Use the existing unsupported-feature path for
cases beyond the admitted material surface.

`BeltPurge.cpp:48–61`, `Print::has_belt_purge_tower`, tests the configured
`filament_diameter` cardinality, not the used-tool count. With the retained
belt/by-layer/non-spiral conditions, **more than one configured slot plus
purge enabled plus a resolved true marker on the printable object must be
rejected**, even for a single used tool. `Print.cpp:3612–3626`, `Print::process`,
enters `_plan_belt_purge` before knowing whether there are tool changes;
`BeltPurge.cpp:146–183` performs setup and only then takes its no-change
return. A successful Native slice with no visible purge moves is not evidence
that this path was inactive. With purge disabled, a marked mesh is still
ordinary geometry and does not receive belt brim (`PrintObject.cpp:1369–1393`).
With no marked object, purge enabled does not prevent normal tool changes.

Use the applied logical count and object marker in
`crates/ares-core/src/project_slice/belt.rs::admit`, shared by print validation
and slicing. Keep the existing selector and compatibility checks, including
valid resolved tool indices; do not use typed configured usage as the layer
schedule (`project/effective_config/{cardinality,selector_validation,usage}.rs`).
`Print.cpp:689–727,772–804` distinguishes configured object extruders from the
broader set used for validation, which includes current-plate tool changes.
Neither set proves that a tool actually extrudes.

`Print.cpp:2278–2310`, `Print::validate`, warns about unpurged changes for
by-object sequencing or, in the admitted sequence, when purge is enabled,
there is no prism, and `object_extruders().size() > 1`. The latter is a
warning, not a rejection, and is not triggered by a `MultiAsSingle` switch
alone. The CLI logs these validation warnings or passes them to its status
callback (`src/OrcaSlicer.cpp:7009–7078`, `CLI::run`); successful fixture
G-code and `result.json` do not carry the warning. Preserve admission; CLI
warning-log and callback parity are outside this G-code contract. Multiple
prisms and by-object sequencing remain rejected by the existing shape and
sequence checks. The common tower checks at `Print.cpp:2441–2480,2505–2558`
run only with a classic tower or an active belt purge prism; do not impose
those diameter/grid checks on towerless prints. The equal-grid checks also
require multiple objects. Neither those checks nor the classic-only relative-E
and ooze-prevention constraints enable an otherwise rejected active case.

## Tool ordering and export

Use the shared by-layer orderer and normal filament-change path. Do not
synthesize a wipe tower, purge-only tool, or extra purge pass. Classic tower
exclusion and the configuration echo remain unchanged (`Print.cpp:5189–5207`,
`GCode/ToolOrdering.cpp:419–440`, `GCode.cpp:8272–8298`; see the purge spec for
classic tower position fitting).

| Native owner | Observable rule and Ares destination |
| --- | --- |
| `GCode/ToolOrdering.cpp:557–614,765–778,833–894`, `ToolOrdering::{ToolOrdering,initialize_layers,collect_extruders}` | Order actual wall and fill roles on the belt-shifted physical layer grid; merge Z entries with Native epsilon. Reuse `project_slice/gcode_emit/tool_ordering/` collection and `layers/tool_buckets.rs` role routing. Configuring a role or an extra slot without any matching extrusion does not alone schedule a tool visit. |
| `GCode/ToolOrdering.cpp:584–600,833–891,1454–1516`; `CustomGCode.cpp:103–112` | The current plate's `MultiAsSingle` `ToolChange` events change the object filament only when multiple slots are configured and there is exactly one configured object extruder. For an in-range event, apply the persistent override at the first physical object layer where `event_z < print_z + EPSILON`. Route walls and fills through the same override; the stored event is not a raw G-code insertion. Reuse `tool_ordering/custom_gcodes.rs` and the per-layer override in the object buckets. |
| `PrintObject.cpp:1395–1408`, `PrintObject::belt_brim_filament`; `GCode/ToolOrdering.cpp:570–578,1001–1043`, `ToolOrdering::collect_extruders` | Derive brim ownership from the lowest positive resolved outer-wall selector (one-based, default 1), independent of an object-layer tool override. Register this tool on every nonempty apron and coincident brim band before recommendation, reorder, and extruder statistics. An apron-only layer has a tool but is not an object layer. Share one `project_slice/belt/brim/` plan between tool ordering and `gcode_emit/layers/schedule.rs`; replace the late slot-zero patch rather than adding a second orderer. |
| `GCode/ToolOrdering.cpp:480–501,1245–1272,2858–2908`; `GCode.cpp:3651–3679` | Use the normal no-tower reorder with the complete band and object tool lists. Initial/last tools, used-filament maps and export state must reflect that schedule, including a print which starts directly on a nonzero slot. Preserve emitted filament metadata and statistics through existing export paths. |
| `GCode.cpp:6155–6248,6253–6267,7568–7618,8134–8162,10990–11059`, `GCode::{process_belt_brim_layer,emit_belt_brim_bands,process_layer,set_extruder}` | An apron-only layer selects its registered tool. On an ordinary layer emit apron bands in the matching tool pass, and coincident bands once at the matching instance visit; otherwise the end-of-layer sweep selects the brim tool before trying the unvisited band's emission. Keep the resulting active tool for the next layer. In `gcode_emit/layers/` use the matching tool plus the existing `tool_transition` path (including filament-change macros); do not emit a band in the first arbitrary nonempty pass or under whichever tool is active at fallback. A coincident band can cause a return switch on the same Z even when the object tool was already changed by a plate event. |

Do not change belt transforms or brim geometry. Active purge alignment,
truncation, fill allocation, prism mesh generation and placement remain
separate work. Preserve the existing Cartesian and one-slot belt paths.

## Native fixture witnesses and verification

Use `tests/parity/mesh-edge-pairing/cube-control.3mf` with the existing belt
coordinate/remap settings, 45° X tilt, 0.2 mm layers, one 0.4 mm nozzle in
single-extruder multi-material mode, and 5 mm outer brim where requested.
Fill out the per-filament vectors, including
colors and maps, and use a correctly sized flush matrix even with no tower
(`GCode.cpp:8272–8298`, `GCode::append_full_config`). For event fixtures put
`ToolChange` entries in the current plate's `Metadata/custom_gcode_per_layer.xml`
with `MultiAsSingle` mode (`Format/bbs_3mf.cpp:3420–3480`). The test change
macro can be a comment naming the previous and next tool; do not let the macro
itself select a tool. Here slot 1 emits `T0`, slot 2 emits `T1`.

The fixtures below were sliced successfully by both Official and Seeded Native
at the reference pin. The tool witnesses check positive extrusion moves under
the selected tool and distinguish an initial selection from a later change.

| Fixture | Native witness |
| --- | --- |
| Two slots, unmarked, both tower switches enabled, slot 1 only; also three configured slots with both switches off | Only `T0`, with brim and outer walls on `T0`; no tower or manufactured switch. |
| Two slots, all roles inherited from object slot 2, no purge | Only `T1`, with the apron, coincident brim and object paths on `T1`; do not confuse initial selection with a swap. |
| Two slots, marked mesh with purge disabled and brim requested | Normal `T0` walls and no brim. |
| Two slots, unmarked, no brim, plate switches at 80 mm to slot 2 and 82 mm back to slot 1 | `T0 → T1 → T0`; walls and solid fill use `T1` starting at `;Z:80.1675` and return to `T0` at `;Z:82.1675`. These are physical belt print-Z values, not model-space heights. |
| Same plate switches, with outer brim and both tower switches enabled | At `;Z:80.1675`, Native changes to `T1`, extrudes object walls/fill, changes back to `T0` on that same layer, and extrudes brim. Also exercise an early event at 1 mm (back to slot 1 at 80 mm): below the first object there is `T0` apron; at the first object layer (`;Z:77.9675`) `T1` prints the wall and `T0` prints the coincident brim. |
| Two slots, unmarked, internal solid infill set to slot 2; walls and brim inherited from slot 1; purge off and on | `T0` prints outer walls and brim, `T1` prints internal solid infill starting at `;Z:79.1675`, with real switches and no purge in either case. A sparse-infill-only override on this tilted cube is not a substitute: it may not extrude. |
| Two slots, marked printable mesh with purge enabled, with and without the plate switches | Native enters the planner in both cases, even though the no-switch case only selects `T0`. Ares must reject both without writing G-code; these are bounds tests, not successful export comparisons. |

After spec approval, add the CLI witnesses under
`crates/ares-cli/tests/orca_parity/belt_print/` before the feature. Reuse the
belt whole-file byte comparator (standard generator handling only), with
Official and Seeded each checked against its own Native output. Assert real
post-selection extrusion and absence of classic tower and purge output on the
inactive paths; the complete comparison owns the ordering, motion, metadata
and configuration. Preserve the existing feature rejection tests and update
the old multi-slot rejection to explicitly exercise active purge. Run focused
paired parity with no selected skips, workspace nonparity and both native and
Wasm Clippy gates. Check entire Ares G-code hashes across 1, 2, 7 and 16 Rayon
workers for representative unused-slot, nonzero-slot, plate-switch/brim and
role-split inputs.
