# Plate custom G-code emission (slice C)

This is the deferred
**Emission** item of `project-plate-custom-gcodes.md` (slice A). Slice A
loads `Model::plates_custom_gcodes`, and slice B shows them in the legend.
No slice yet puts a tick into the G-code, so an Ares slice of a project with
a pause or custom tick lacks Native's lines. Slice B's times are those of
that shorter G-code.

## Upstream boundary

Upstream is OrcaSlicer v2.4.2, `8500fcdcc`. Citations give exact lines.

| Upstream | What | Ares destination |
|---|---|---|
| `GCode/ToolOrdering.hpp:168` `LayerTools::custom_gcode` | Per-layer pointer to at most one `CustomGCode::Item`, `nullptr` by default | new field `custom_gcode: Option<CustomGCodeItem>` on `LayerTools` (`project_slice/gcode_emit/tool_ordering.rs:26-35`) |
| `GCode/ToolOrdering.cpp:1448-1516` `ToolOrdering::assign_custom_gcodes` (comment `:1448-1452`, body `:1454-1516`) | Takes `get_curr_plate_custom_gcodes()` and returns when it is empty (`:1459-1461`). Print mode: one entry in `filament_diameter` gives `SingleExtruder`; one object extruder gives `MultiAsSingle`; else `MultiExtruder` (`:1464-1467`). `ignore_tool_and_color_changes` when exactly one of print mode and model mode is `MultiExtruder` (`:1473`). Walks the layers from the top down (`:1478-1479`) and marks each layer's extruders as printing at and above (`:1482-1483`). The skip loop (`:1485-1487`) skips events above the midpoint to the layer above, but only when there is a layer above: at the top layer `print_z_above` is `lowest()`, so an event above the top layer is **not** skipped and lands on the top layer. The same loop skips **every** `ToolChange`, so the `tool_change` / `tool_changes_as_color_changes` branch (`:1475`, `:1501`, `:1509`) is unreachable and a ToolChange item never reaches a layer. An event applies to the layer when `print_z > 0.5 * (print_z_below + lt.print_z)`, with `print_z_below = 0` under the first layer (`:1495-1498`); an event at or below the first layer's midpoint is never assigned. Pause, Custom and Template always apply. A color change applies in `SingleExtruder` mode, or when its 1-based `extruder <= num_filaments` and that extruder prints at or above this layer (`:1503-1509`). An applied event is stored (`:1510-1511`). The event is consumed either way (`:1513`). For a list sorted by `print_z` (slice A keeps file order), the last of several events inside one layer span wins | new `project_slice/gcode_emit/tool_ordering/custom_gcodes.rs` (`ToolOrdering::assign_custom_gcodes`), reading `traversal.project.curr_plate_custom_gcodes()` (slice A) |
| `GCode.cpp:2764-2767` (`GCode::_do_export`, non-sequential branch) | `tool_ordering = print.tool_ordering(); tool_ordering.assign_custom_gcodes(print);`, i.e. after the wipe tower's tool ordering is final. The sequential branch (`:2758-2760`, "We don't allow switching of extruders per layer by Model::custom_gcode_per_print_z in sequential mode") never assigns | the by-layer ordering in `project_slice/gcode_emit/layers/schedule.rs`, after `wipe_tower::prepare` (`:292`). The by-object `OrderingScope::Object` orderings (`layers.rs:190-200`) are not assigned |
| `GCode.cpp:4825-4828` (`GCode::process_layer`) | `if (single_object_instance_idx == size_t(-1))`: append `emit_custom_gcode_per_print_z(*this, layer_tools.custom_gcode, current filament, first_extruder_id, config)`. This comes after the second-layer temperature block (`:4777-4823`) and before the extruder loop | `project_slice/gcode_emit/layers/boundary.rs`, right after `machine::append_second_layer_transition` (`:303-307`) and before the BBL eager lift/timelapse (`:308-318`, upstream `:5205-5210`) |
| `GCode.cpp:4174-4258` `ProcessLayer::emit_custom_gcode_per_print_z` | With an item: the color change block `:4203-4231` is `#if 0` (BBS: "inserting color gcode is removed"). The live code: **PausePrint** (`:4233-4241`) gives `;` + `reserved_tag(Pause_Print)` + `\n`, then `machine_pause_gcode` through the placeholder parser, then `\n`. **Every other type**, *including ColorChange* (`:4242-4250`), gives `;` + `reserved_tag(Custom_Code)` + `\n`, then `template_custom_gcode` through the placeholder parser for `Template`, else the raw `extra` (empty for a color change). Both branches then append one more `\n` (`:4251`). The pause message `M117` is commented out (`:4237-4239`) | new `project_slice/gcode_emit/layers/custom_gcode.rs`, rendered with the existing `template::render` and the existing processor tag table (`processor/moves/tags.rs:25-36`, `pause_print` / `custom_code`) |

The ColorChange quirk is taken from the code as written. The Native RED
below must confirm it byte for byte before GREEN relies on it.

### Upstream boundaries with nothing to port

- **`Print.cpp:2507-2508` (`Print::process`).** It assigns custom G-codes
  to a local copy of the ordering, which is used only for the first-layer
  filament maps (`:2510-2512`). Assignment only stores a pointer and never
  changes `extruders`, so the maps are unaffected. Nothing to port.
- **`Print.cpp:528-536` `Print::extruders(conside_custom_gcode = true)`.**
  It appends the extruders of `ToolChange` items only. Its consumers are
  `Print.cpp:3707` (the output-name filament) and `PrintApply.cpp:1127`,
  `:1620` (used filaments). Without `ToolChange` emission (deferred below)
  the result equals `extruders(false)`, so it is deferred with ToolChange.
- **`PrintApply.cpp:1311-1322`.** On a custom G-code change between two
  `Print::apply` calls it invalidates `psWipeTower`/`psGCodeExport`, and
  stores the new plate entry. Ares slices statelessly: each call rebuilds
  the ordering from the `Project`, so there is no step cache to
  invalidate. No port; the stored entry is slice A's `Project` map.

## Included and deferred

**Included:**
- `assign_custom_gcodes` for the by-layer print, every type, all three
  print modes, and the mode-mismatch ignore. ToolChange items are skipped,
  as upstream does.
- The per-layer emission of PausePrint, Custom, Template and ColorChange,
  in the `#if 0`-reduced upstream form.
- No change for projects without the entry, which is every fixture outside
  this slice: `assign_custom_gcodes` returns before touching a layer, so
  their G-code bytes do not change.

**Deferred (each its own later slice):**
- **Per-layer extruder switches from ToolChange items.** `MultiAsSingle`
  `per_layer_extruder_switches` (`ToolOrdering.cpp:464-476`,
  `CustomGCode.cpp:62-72` `custom_tool_changes`) and `Print::extruders(true)`
  (`Print.cpp:528-536`). These change which tool prints a layer and the
  wipe tower; that is a tool-ordering slice, not emission.
- **The CLI `--custom-gcode` JSON override** (`OrcaSlicer.cpp:1918-1950`,
  `:3581-3600`).
- **Sequential printing:** upstream emits nothing there either
  (`GCode.cpp:2758-2760`, `:4825`). Covered by one guard case, no code.
- **The `same-layer` case** (two Custom events inside one layer span, the
  last wins, `ToolOrdering.cpp:1513`). Its emitted body matches Native, but
  Native places `M73 P27`/`P28` differently around the lone `;CUSTOM_GCODE`
  vertex (`GCodeProcessor.cpp:3314-3320` `store_move_vertex(Custom_GCode)`).
  That is a `GCodeProcessor` gap, so the case is the RED of a separate
  processor slice and merges with that slice's GREEN
  (`docs/architecture/gcode-processor-options-z-corrector.md`).
- **Web:** after this slice, slice B's overview and the estimation footer
  show Native times without any web change. Single-load (slice B's
  deferral) stays deferred.

## Allowlist

| File | Edit |
|---|---|
| `crates/ares-core/src/project_slice/gcode_emit/tool_ordering.rs` | `LayerTools::custom_gcode` field, `mod custom_gcodes;` |
| `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/custom_gcodes.rs` | new: `assign_custom_gcodes` |
| `crates/ares-core/src/project_slice/gcode_emit/layers/schedule.rs` | one call after `wipe_tower::prepare`, by-layer only |
| `crates/ares-core/src/project_slice/gcode_emit/layers/boundary.rs` | `LayerBoundary::custom_gcode` field, one call after the second-layer transition |
| `crates/ares-core/src/project_slice/gcode_emit/layers/custom_gcode.rs` | new: `emit_custom_gcode_per_print_z` |
| `crates/ares-core/src/project_slice/gcode_emit/layers/drive.rs` | **one field** in the `boundary::LayerBoundary { .. }` literal (`:128-136`): `custom_gcode: tool_ordering.for_layer(group_z).custom_gcode.as_ref()` |
| `crates/ares-core/src/project_slice/gcode_emit/layers.rs` | `mod custom_gcode;` |
| `crates/ares-cli/tests/orca_parity/runner/plate_custom_gcode_emission.rs` (+ `mod` in `runner.rs`; under `runner/` because `ExportedCase` fields are runner-private) | new Native comparison |
| `tests/parity/plate-custom-gcode-emission/*.3mf`, `README.md` | new fixtures |
| `tools/chain-probe/plate-custom-gcode-emission-fixtures.mjs` | new fixture writer (same form as slice A's) |
| `docs/architecture/plate-custom-gcode-emission.md`, `docs/roadmap.md` | this spec, status line |

**Read-only notes:**
- `crates/ares-core/src/project/load.rs`: read-only. Slice A's
  `Project::curr_plate_custom_gcodes()` is the only input.
- `layers/drive.rs`: the single added field above is the only edit. It
  sits inside the `boundary::append` call, and no other drive.rs line
  changes.
- `gcode_emit/skirt.rs`, `layers/skirt_schedule.rs`,
  `layers/skirt_emit.rs`: read-only, no edit. `schedule.rs` is shared with
  the skirt schedule (`:284-291`); the one new call sits after `:292`.
- `project_slice/state.rs`, `print/validate.rs` and `plate_selection` are
  not touched.

## Native RED plan

**Oracle.** Official OrcaSlicer 2.4.2 run by the orca_parity runner in both
modes (official wrapper sha `1689a2ff4656…`, seeded replay). It slices each
fixture project, and the comparison is ordered G-code bytes against Ares.
`ARES_REQUIRE_ORCA=1`. Nothing Native is committed.

**Fixtures.** No ColorChange uses `extruder` 0 in a non-single mode:
upstream would index `extruder_printing_above[-1]` (`:1508`). Each one is a
base project plus a
`Metadata/custom_gcode_per_layer.xml` written by the fixture script. Ticks
are placed at layer `print_z` values of the base project's own layers.

| Case | Base | Entry | Upstream path checked |
|---|---|---|---|
| `control` (guard) | `orca_cli_ender3.3mf` unchanged | none | base parity of the fixture base |
| `pause` | `orca_cli_ender3.3mf` (1 filament, `machine_pause_gcode` = `M25`) | PausePrint at layer 3, `extra` = message | `:4233-4241`, message not emitted |
| `custom` | same | Custom, multi-line `extra` | `:4242-4250` raw `extra` |
| `template` | same, with `template_custom_gcode` set in `project_settings.config` | Template | `:4245-4246` placeholder render |
| `color-change` | same | ColorChange, extruder 1 | the `#if 0` quirk: Custom_Code tag + empty `extra` |
| `bounds` | same | Custom below half of the first layer; Custom above the top layer | the first is never assigned (`:1498`, `print_z_below = 0`); the second lands on the top layer (`print_z_above = lowest()`, `:1478`, `:1486`) |
| `mode-mismatch` | same | model mode `MultiExtruder` + ColorChange + PausePrint | color change ignored (`:1473`), pause kept |
| `tool-change` | same | model mode `MultiAsSingle` + ToolChange + Custom on another layer | ToolChange skipped (`:1487`), so only the Custom is emitted |
| `two-filament-control` (guard) | runtime: the vg1 X1C two-filament cube10 export (`vg1_inputs::bind_two_filaments`), all on filament 1 | none | base parity of the two-filament base |
| `multi-as-single` | runtime: the same export. Not `validation/multi-filament-bridge-anchor-narrowed.3mf`: its official appimage run has an uninitialized object id, so that fixture is byte-compared only in seeded mode (`narrowed_x2d_whole_file.rs:14-21`) | explicit `<mode value="MultiAsSingle"/>` (without it, slice A's mode check turns `extruder > 1` into `MultiExtruder` and the mismatch ignore drops both); ColorChange extruder 2, ColorChange extruder 1 on separate layers | `extruder_printing_above` (`:1508`): extruder 2 dropped, extruder 1 kept |
| `by-object` (guard) | `orca_cli_ender3.3mf` with `print_sequence` = `by object` | PausePrint | no emission (`:2758-2760`, `:4825`) |

**Native witness.** Before the byte comparison, each case checks Native
itself: how many pause/custom tag lines Native writes, and which payload
lines are present or absent (for example `M117 second` but not
`M117 first`). That way the upstream readings above are confirmed by Native,
not only by the Ares diff.

**RED expectation.** On `main` every case except the three guards
(`control`, `two-filament-control`, `by-object`) must fail on the
ordered-bytes assertion, at the first line Native adds (`;PAUSE_PRINT` /
`;CUSTOM_GCODE` …), in both modes. Two things fail the RED: a failure that
is not that assertion (a panic, a missing Native binary), and a case that
already passes. The three guards are expected to pass at RED and stay as
guards. If a guard fails, the base is not at parity, and that is root-caused
before the RED counts.

**Ways it could fail (checked by the cases):**
- **Midpoints:** the midpoint rule off by one layer, using `>=`, or
  skipping an event above the top layer.
- **Mode:** print mode from filament count alone (missing `MultiAsSingle`),
  or the mismatch rule applied to pauses.
- **ToolChange:** assigning it, or emitting it as a color change.
- **Extruder ids:** a 0-based vs 1-based mix-up in `extruder_printing_above`,
  or a `MultiExtruder` color change for an extruder that is out of range.
- **Emission:** emitting the `#if 0` M600 block; a missing final `\n`;
  emitting the pause message; rendering `extra` through the placeholder
  parser.
- **Placement:** before the second-layer M140, after the BBL timelapse, or
  emitting in sequential mode.
- **Processor:** the processed output's M73 stop times. `PAUSE_PRINT` /
  `CUSTOM_GCODE` already go through the ported processor
  (`processor/stop_times.rs`). A mismatch there is reported as a finding;
  this slice does not change the processor. The `same-layer` finding is
  deferred above.

## Gates

| Stage | Gates |
|---|---|
| RED | the cases above, official + seeded, assertion failures only |
| GREEN | the same cases pass in both modes; `cargo fmt`, strict clippy, wasm32; nextest `ares-core` + `ares-cli` directed; the full parity suite (official + seeded, fresh cache) with 0 new failures; workspace |
