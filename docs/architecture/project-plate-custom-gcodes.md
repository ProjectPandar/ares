# Project load: plate custom G-codes (slice A)

Status: Proposed. It is the data source
for slice B, the legend Custom G-code overview
(`web-custom-gcode-overview.md`).

## Upstream boundary

In the editor, the preview reads `model().get_curr_plate_custom_gcodes()`
(`GCodeViewer.cpp:4478-4479`). The model fills that map when the project
3MF is loaded. ares-core reads neither the archive entry nor the map, and
none of the 146 tracked `.3mf` fixtures contains the entry.

| Upstream (v2.4.2, `8500fcdcc`) | What | Ares destination |
|---|---|---|
| `Format/bbs_3mf.cpp:220` | Entry `Metadata/custom_gcode_per_layer.xml` | new `crates/ares-core/src/project/plate_custom_gcodes.rs` |
| `bbs_3mf.cpp:1948-1951` | The entry is read only when the config is loaded (`!dont_load_config`), matched case-insensitively (`iequals`) | same; Ares project load always loads the config. Called from `project/load.rs` next to `layer_config_ranges::load` (`:153`) |
| `bbs_3mf.cpp:3208-3292` `_extract_custom_gcode_per_print_z_from_archive` | An empty entry is ignored. A root other than `custom_gcodes_per_layer` is ignored. The map is cleared. With `<plate>` children, each one's `<plate_info id>` selects key `id - 1`, and a plate without `plate_info` is skipped. Without any `<plate>`, the root's children go to key 0. `<mode value>`: `SingleExtruder`/`MultiAsSingle`, anything else is `MultiExtruder`. `<layer>`: `top_z` (f64), `extruder` (int), `color`; then `type` (int) + `extra`, or, when `type` is absent, the legacy `gcode` (`M600` ColorChange, `M601` PausePrint, `tool_change` ToolChange, else Custom; `extra` = color for pause, the gcode for custom, else empty). Items stay in file order | same; XML through the existing `project::xml` reader |
| `Model.cpp:242`, `:326`, `:365-366`; `CustomGCode.cpp:41-58` `check_mode_for_custom_gcode_per_print_z` | After load, each plate whose mode is `Undef` gets one: any ToolChange gives `MultiAsSingle`; else a ColorChange with `extruder > 1` gives `MultiExtruder`; else `SingleExtruder` | same module |
| `CustomGCode.hpp:24-43` `Item`, `:68-92` `Mode`/`Info` | `{print_z, type, extruder, color, extra}`, `{mode, gcodes}` | `Item` reuses the existing `gcode_processor::CustomGCodeItem` / `CustomGCodeType` (same upstream struct); new `CustomGCodeMode` and `CustomGCodeInfo` |
| `Model.hpp:1562-1570` `curr_plate_index`, `plates_custom_gcodes`, `get_curr_plate_custom_gcodes` | A map keyed by plate index; the current plate's entry, or an empty `Info` | `Project::plates_custom_gcodes()` and `Project::curr_plate_custom_gcodes()`, keyed by the plate the project slice already selects (`plate_origin`'s plate; the CLI sets `curr_plate_index` at `OrcaSlicer.cpp:5690`) |

## Included and deferred

**Included:**
- The load, the legacy form, the plate keying, the fallback to plate 1,
  and the mode check.
- The public `Project` accessors.
- No slicing change: projects without the entry (every current fixture)
  load as before, so G-code bytes are unchanged.

**Deferred (each one is its own later slice):**
- **Emission.** The ticks are not yet acted on when slicing:
  - `ToolOrdering.cpp:1453-1500` `assign_custom_gcodes`;
  - the tool-change extruders at `Print.cpp:528-536`;
  - `PrintApply.cpp:1311-1322`;
  - the `GCode.cpp` per-layer insertion.

  Until that lands, an Ares slice of a project with ticks lacks Native's
  M600/M601/custom lines. Slice A's fixtures are compared on the loaded
  data only, never on G-code.
- The PrusaSlicer `3mf.cpp` path (`Model.cpp:388-392`,
  `Prusa_Slicer_custom_gcode_per_print_z.xml`), which only
  `read_from_archive` reaches for Prusa-made files. Ares loads BBL/Orca
  projects.
- The CLI `--custom-gcode` JSON override (`OrcaSlicer.cpp:1918-1950`,
  `:3581-3600`) and `skip_modified_gcodes`.
- The export side (`bbs_3mf.cpp:8462-8510`).

## Ways it could fail

- **Keys:** keying plates by `id` instead of `id - 1`, or not skipping a
  plate without `plate_info`.
- **No-plate fallback:** missing it, or applying it when at least one
  `<plate>` exists.
- **Mode:** treating an unknown mode string as `Undef` (upstream gives
  `MultiExtruder`), or running the mode check before every plate is read.
- **Legacy form:** reading `type` as 0 when it is absent, or a wrong
  `extra` for pause and custom items.
- **Order and precision:** sorting items instead of keeping file order;
  parsing `top_z` as f32.
- **Entry name:** case-sensitive matching.
- **Current plate:** returning a non-empty list for a plate without an
  entry, or using the 1-based plate number as the key.
- **Bad input:** a missing required attribute or a non-numeric `type` at
  this system boundary. Ares returns `SliceError::InvalidInput`.
  Upstream's `ptree::get` throws there and the load fails; the oracle
  confirms that on one fixture.

## RED-first plan and evidence

1. **Native oracle, generated at runtime, never stored.** A new
   `tools/chain-probe/plate-custom-gcodes.{nix,patch}`. Right after
   `Model::read_from_file` (`OrcaSlicer.cpp:1640`), which includes the
   `:365-366` mode check, it writes `model.plates_custom_gcodes` as JSON to
   a path from an env var: per key, the mode, then each item's `print_z`
   (`%.17g`), type, extruder, color and extra. Then it exits. The probe is
   built with Nix and pinned. The env
   var is `ARES_PLATE_CUSTOM_GCODES_BIN`.
2. **Fixtures:** `tests/parity/plate-custom-gcodes/*.3mf`, made by a
   committed Node script from `tests/parity/orca_cli_ender3.3mf` plus an XML
   entry:
   - `plates`: two `<plate>` blocks (ids 1 and 2) with all six types, and a
     third `<plate>` without `plate_info`.
   - `no-plate`: layers directly under the root.
   - `legacy`: the `gcode` attribute with `M600`, `M601`, `tool_change` and
     other.
   - `modes`: plates with no mode element that resolve to SingleExtruder,
     MultiExtruder and MultiAsSingle, plus an unknown mode string.
   - `wrong-root` and an empty entry.
   - `missing-attr`: the error case.
3. **Core RED:** `crates/ares-cli/tests/orca_parity/plate_custom_gcodes.rs`
   compares `load_project(..).plates_custom_gcodes()` with the Native JSON:
   modes, items, and `print_z` f64 bits. The loader exists and returns an
   empty map, so RED fails on an assertion. With `ARES_REQUIRE_ORCA=1` a
   missing producer fails. A second test checks `curr_plate_custom_gcodes()`
   for the selected plate of `plates`.
4. **Core GREEN:** the port.
5. **Gates:** this is a production advance, but it changes no G-code.
   - Rust: fmt, clippy `-D warnings`, the wasm32 build, and `nextest -p
     ares-core -p ares-cli` for the new test with the producer.
   - The workspace run and the full official/seeded pair. Identical
     `ares.gcode` is expected for every case.

## Open question

1. **Where B gets the list.** B receives it from A's
   `Project::curr_plate_custom_gcodes()` through the wasm slice adapter.
   The transport (on the processed output, or read from the project the
   browser Web Worker already holds) is decided and cited in B's spec, not here.
