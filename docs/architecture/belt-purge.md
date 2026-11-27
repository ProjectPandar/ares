# Belt purge prism: option handling and backend boundary

Reference: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Paths below are relative to `src/libslic3r/` unless otherwise noted.

## Scope and admission

`BeltPurge.cpp:48–61`, `Print::has_belt_purge_tower`, requires a belt printer,
`enable_belt_purge_tower`, non-spiral mode, **more than one configured filament
slot** (`filament_diameter.size()`, not the number of used tools), and a
`PrintObject` with `belt_purge_tower_object`. `Print.cpp:3598–3628`,
`Print::process`, invokes the planner only for non-`ByObject` sequencing. The
planner also tests for a tool change after constructing the layer grid and tool
ordering; activation and actual purging are distinct.

Ares's `crates/ares-core/src/project_slice/belt.rs::admit` requires exactly one
printable object with one instance and one model-part volume, by-layer
sequencing, exactly one configured logical filament, and no used slot other
than slot zero. `project/effective_config/cardinality.rs:39–40` derives that
logical count from `filament_diameter`. Therefore **the active planner is
unreachable without changing the material restriction**, even for a marked
single object. A model and a separate marked prism would also violate the
object restriction. Admitting unused extra filament slots would still change
that restriction and is not part of this slice.

With the existing restrictions intact, the executable scope is the inactive
purge cases and the marker's independent effect on belt brim. Remove only the
`belt_purge` rejection in `belt.rs:64–79`; retain the support, object/instance,
volume, material, sequencing, and other existing checks. Do not add an
unreachable planner or claim active purge support. An active backend port
requires a separate change to those prerequisites and its own positive Native
comparison. The existing failure in `belt.rs` is an admission check, not a
purge strategy to preserve.

## Included source behavior and Rust destinations

| Upstream owner | Behavior | Ares destination (relative to `crates/ares-core/src/`) |
| --- | --- | --- |
| `PrintConfig.hpp:1377,1983–1986`; `PrintConfig.cpp:7810–7833,8499–8507` | Printer enable and object marker default to false. `belt_purge_tower_width` defaults to 35 mm (minimum 1 mm), but does not generate or resize a model in the backend. | Reuse printer and width fields in `options/printer_options/remaining.rs` and `options/process_options/print_source.rs`. Resolve the marker through `options/object_fields.rs` and the corresponding object source/override decoding. No new option or public slicing endpoint. |
| `Format/bbs_3mf.cpp:2266–2281`; `PrintApply.cpp:1667–1675`; `PrintObject.cpp:4289–4300`, `PrintObject::object_config_from_model_object` | Load object metadata from the project and overlay it on print-object defaults. An explicit object value can clear an inherited true marker. | Use `project/model_settings/object_metadata.rs` and `project/effective_config/candidates.rs::resolve_project_objects`. Replace the raw project-settings probe with the resolved object marker where it is consumed. Do not infer the marker from the name, width, enable switch, or `flush_into_objects`. |
| `BeltPurge.cpp:48–61`, `Print::has_belt_purge_tower`; `Print.cpp:2278–2310,3612–3626`, `Print::{validate,process}` | On an admitted belt profile the enable switch does not start purge planning, with or without a marked mesh. No missing-prism or by-object purge warning applies in this domain. A stored marked mesh remains a normal sliced object; the backend neither creates nor removes it in the inactive path. | `project_slice/belt.rs::admit` and the existing slice/export flow; no new object creation, grid alignment, layer truncation, or fill removal in these cases. |
| `Print.cpp:857–872`, `Print::{has_tilted_belt,has_belt_brim}`; `PrintObject.cpp:1369–1393`, `PrintObject::has_belt_brim`; `BeltBrim.hpp:49–50` | A marked object never receives a belt brim, even when the purge switch is off or planning is inactive. For unmarked objects preserve the existing X/Y inclusive absolute tilt window of 1° to 85° and the existing brim predicates. This is a brim window, not a purge activation test. | `project_slice/belt/brim/mod.rs::generate` uses the resolved object marker before constructing bands. The existing `gcode_emit/layers/` schedule and footprint paths receive no bands for that object; do not special-case layer moves or footprint output. |
| `Print.cpp:5189–5207`, `Print::has_wipe_tower`; `GCode/ToolOrdering.cpp:419–440`, `ToolOrdering::insert_wipe_tower_extruder` | No classic wipe tower or extra classic-tower tool on a belt, even when `enable_prime_tower` is true. | Preserve the exclusions in `project/effective_config/usage.rs::has_wipe_tower` and the existing belt tool ordering/export, independent of the purge switch. Cartesian activation stays unchanged. |

`BeltSliceStrategy.cpp:12–80`, `BeltSliceStrategy::apply_preslice_transforms`,
rotates and lifts the supplied mesh; `PrintObjectSlice.cpp:996–1056`,
`PrintObject::slice`, adds the normal per-object print-Z offset and keeps the belt
floor in sync. `GCode.cpp:2338–2388,2474–2486`,
`GCode::collect_layers_to_print`, merges any brim prologue bands with object and
support layers and removes non-extruding belt entries. Preserve these paths for
marked and unmarked meshes. `BeltGCode.cpp:13–48`,
`BeltGCode::{init_belt_writer,write_belt_header,on_set_origin}`, installs the belt
writer, writes the belt frame header, and transforms the object origin. It has
no purge-specific generator; preserve the existing `project_slice/belt/`
transforms and `project_slice/gcode_emit/` object emission rather than emitting
moves in machine coordinates from a new path.

## Deferred active backend and its source boundary

`BeltPurge.cpp` operates on already sliced objects. Prism mesh generation, width,
capacity sizing, and plate placement are in `src/slic3r/GUI/BeltPurgeTower.cpp`
(`ensure_belt_purge_tower`, lines 35–45, 242–264, 409–442), not the headless
backend. The GUI creates a separate object with a marker and `flush_into_objects`
set independently. Do not port automatic generation or placement to Ares core.
A future active fixture must supply the mesh and object options in the project.

| Upstream function | Active behavior to preserve when admitted | Ares destination |
| --- | --- | --- |
| `BeltPurge.cpp:85–138`, `Print::_align_belt_purge_layers`; `:324–345`, `PrintObject::belt_shift_layer_grid`; `Print.cpp:2505–2558`, `Print::validate` | On equal layer grids, take the first nonempty marked prism and use its regular layer height. Take the first tallest non-prism as reference by last `print_z`. Compute residue from first `print_z` as `z - floor(z/h) * h`; wrap each delta to `(-h/2, h/2]` and ignore sub-`EPSILON` shifts. Shift layer and brim prologue Z and belt floor/global offsets together, not slice planes or XY. Support layer shifting is deferred with belt support. | `project_slice/belt/` and the resolved physical layers/floor consumed by belt brim and tool ordering; align before tool ordering reads Z. |
| `BeltPurge.cpp:146–196`, `Print::_plan_belt_purge`; `GCode/ToolOrdering.cpp:765–778`, `ToolOrdering::initialize_layers` | Clear classic-tower results, restore pristine layers/fills where needed, align, then build shared tool ordering without priming. Merge near-coincident Z entries with Native epsilon. Reject empty tool ordering; skip the planning pass when the ordered tool sequence has no changes (do not key off the first band's wipe-tower flag). For a change, use the old/new flush matrix entry times the multiplier only under `purge_in_prime_tower && single_extruder_multi_material`, otherwise `prime_volume`. | `project_slice/gcode_emit/tool_ordering/` using a belt-specific plan from sliced objects, not classic wipe tower output. |
| `BeltPurge.cpp:198–256,353–368`, `Print::_plan_belt_purge` and `PrintObject::belt_truncate_layers_above` | Bound the last-change scan by the tallest non-prism print layer (including support in upstream). Scan ordered tool visits, not just layer boundaries. Trim the first nonempty marked prism strictly above `last_change_z + EPSILON` before marking; if no change is found within the bound, do not truncate it to zero. | `project_slice/belt/` and the layers exposed to `gcode_emit/layers/schedule.rs`. |
| `BeltPurge.cpp:258–315,376–398`, `Print::_plan_belt_purge` and `PrintObject::belt_drop_unclaimed_fills`; `GCode/ToolOrdering.cpp:3394–3406,3429–3572,3574–3631`, `WipingExtrusions::{is_overriddable,mark_wiping_extrusions,ensure_perimeters_infills_order}` | Match absorbers to the layer within `EPSILON`. In tool order, allocate real purge volume using the existing eligibility and entity ordering. Keep only prism fills claimed at that point for copy zero; retain walls. Then apply the ordering overrides. Distinguish real claims from overrides added just for ordering. Report total unabsorbed volume only above 1 mm³, with the largest leftover and its Z, using the upstream rounding. | Extend `project_slice/gcode_emit/tool_ordering/wiping/` and its layer input; keep the same post-plan geometry visible to scheduling and export. |
| `BeltPurge.cpp:400–431`, `PrintObject::{belt_restore_dropped_fills,belt_restore_truncated_layers}`; `Print.hpp:631–638`, `PrintObject::belt_undo_purge_plan`; `Print.cpp:3288–3299`, `Print::process` | Replanning must start from the original layer and fill order, including restored layer adjacency. | Reuse immutable slice input and produce a per-request plan; do not port pointer stashes or upstream incremental invalidation into a one-shot slice. |
| `Print.hpp:945–950,1191,1559–1561`, `WipeTowerData` and `Print::tool_ordering`; `GCode.cpp:3651–3679,4744–4798,7010–7032,7190–7200,7568–7602,10990–11059,11344–11358`, `GCode::{_do_export,process_layer,set_extruder}` | The planner writes the same tool ordering consumed by export. For each tool pass, select/change the tool with the normal filament-change path; emit any belt brim for that tool, then overridden object extrusions before normal extrusion, with the normal purge completion comment. No classic tower moves, priming, or final purge. | `project_slice/gcode_emit/layers/` and the existing tool-change and belt-writer paths. |

Belt support, raft, multi-object or multi-instance belt output, multi-material
belt output, spiral/by-object belt output, and all active planning above remain
outside the admitted slice. This also defers the by-object/missing-prism
warnings and multiple-prism error from `Print.cpp:2278–2310`, `Print::validate`.
Do not introduce synthetic tool changes to make the planner reachable, or copy
undefined upstream behavior.

## Verification contract

Exercise the CLI 3MF-to-G-code boundary in
`crates/ares-cli/tests/orca_parity/belt_print/` and its existing parent module.
Use synthetic geometry, Native OrcaSlicer in both Official and Seeded modes,
and the existing whole-file ordered-byte comparator with only its standard
producer identification handling. Do not filter motion, settings, or purge
comments out of the comparison or store Native output in the repository.

- Replace `rejects_belt_purge_without_gcode` with successful whole-file parity
  for enable on, marker absent, one filament. Cover enable off/on crossed with
  marker absent/present using one printable, pre-positioned synthetic mesh with
  nonzero requested belt brim, at a representative 45° X tilt. A true marker
  suppresses the apron independently of enable; a false marker does not.
- Exercise object metadata independently of project defaults: include an
  inherited true marker and an explicit per-object false override. Vary
  `belt_purge_tower_width` with the same supplied mesh, and include classic
  prime enabled on a belt. Compare each case with Native so configuration
  reporting is also checked; width is not expected to change paths.
- Compare marked and unmarked cases below, at, and just above the brim window
  boundaries (0.5°, 1°, 85°, 86°), retaining an existing no-brim 90° control
  and a supported Y or negative-tilt case. No synthetic purge band should appear.
- Keep support rejection and assert no G-code for disallowed instance/object
  and material cases even when purge is enabled. These are Ares admission
  checks, not assertions that Native rejects its broader supported domain.
- Run workspace nonparity checks and compare Ares output hashes from separate
  runs at different available core counts with identical inputs and settings.
  An inactive-case match does not establish active purge parity. That would
  require fixtures with a model plus a marked prism and real swaps, covering
  alignment, bounded truncation, claimed fills, tool-pass order, and leftover
  warnings in both Native modes after the admission prerequisites change.
