# Web preview: Summary view

Status: approved (`eea439ac6`); data-source ruling option A. Branch `pi/web-summary` from
`042fe5687`.

## Slice selection

Two candidates remained from the actual-speed slice: the "Actual speed profile"
table and plot, and Summary.

**Summary goes first.**
- Upstream lists 16 view types in the preview combo (`GCodeViewer.cpp:1077-1093`).
  The web combo has 15 of them; Summary is the only one missing.
- The actual-speed profile is compiled in (`Technologies.hpp:52`,
  `ENABLE_ACTUAL_SPEED_DEBUG 1`), but it lives inside the tool-marker
  properties panel (`GCodeViewer.cpp:518-614`, data built at `:1681-1753`).
  That panel is the ToolPosition properties table, which is still deferred.
- So the profile first needs the properties table and stays deferred behind it.

## Upstream boundary

| Upstream | What | Ares destination |
|---|---|---|
| `GCode.cpp:1904-1940` `DoExport::update_print_estimated_stats` (called at `:2176`, after `GCodeProcessor::finalize`) | Builds `PrintStatistics` `total_extruded_volume`, `total_used_filament`, `total_weight`, `total_cost` from `GCodeProcessorResult::print_statistics.total_volumes_per_extruder` and the normal-mode time. These are the values the GUI reads. | ares-core: a `PrintStatistics` value computed after the processor, next to `processor/material_statistics.rs`, and carried on the slice output |
| `Print.hpp:789-816` `PrintStatistics` | Only the fields above; the rest is deferred | ares-core struct, serialized through the existing `takePrintStatistics` JSON (`crates/ares-wasm/src/moves.rs:65-68`) or a sibling getter |
| `GCodeViewer.cpp:4003-4024` legend `case EViewType::Summary` | Rows `Total: %.2f m / <weight>`, `Cost: %.2f`, `Total time: short_time(get_time_dhms(time_mode.time))` | `web/app/src/preview/` legend, in the lazy preview chunk |
| `GCodeViewer.cpp:128-150` `format_compact_weight` | Metric weight format | Existing `LegendFilament.tsx` port, reused |
| `Utils.hpp:494-537` `short_time` and `:540-` `get_time_dhms` (f32) | Time format | Core has a `get_time_dhms` port (`processor/time.rs::duration`), but it is private to the processor. A Rust `short_time(get_time_dhms(t))` will be exposed through wasm so JS does not reimplement the f32 arithmetic. |
| `ViewerImpl.cpp:1543-1548` / `crates/ares-vgcode/src/viewer/colors.rs:243` | Summary colors vertices like ColorPrint | Already ported; `VIEW_TYPES` already has Summary at slot 0 |
| `GCodeViewer.cpp:66-69` | Label "Summary" | `VIEW_TYPE_ITEMS` entry, first in combo order |

### Details of the upstream computation that must be kept

- **Which volumes count:** the loop runs over `total_volumes_per_extruder`.
  Volumes whose extruder id is not in `m_writer.extruders()` are skipped, but
  `total_extruded_volume` is added **before** that skip.
- **Per-filament values:** `Extruder::filament_diameter/density/cost` return
  `double` through `get_at(id)`. This is not the f32 path in
  `material_statistics.rs`, which ports `post_process`.
- **Time cost:** `total_cost += time_cost * normal_time / 3600`. This applies
  even when the legend shows the stealth mode time.
- **Unit conversion:** `total_used_filament` is in mm, and the legend divides
  it by 1000.

## Included and deferred

**Included:**
- The four totals for a single plate.
- The Summary combo entry and its legend rows, metric only.
- Normal time mode.

**Deferred:**
- Imperial units.
- The multi-nozzle grouping block (`GCodeViewer.cpp:4683`).
- The all-plates "Total Estimation" (`:2550-2810`).
- The "Time Estimation" window (`:4551`).
- Other `PrintStatistics` fields: wipe-tower and support filament,
  toolchanges, `filament_stats`.
- The actual-speed profile.

## Scope note: ares-core

The original task said core slicing code belongs to other workers. This slice
adds a read-only post-processor value (`update_print_estimated_stats`) and
changes no slicing or G-code output. I'm asking the coordinator to approve
this core addition here. If it should instead live in ares-wasm, the same
function moves there unchanged.

## Oracle and evidence

1. **Probe:** `tools/chain-probe/print-statistics.nix` builds a probe of
   official 2.4.2 (with `--cores 8`). It patches the CLI after export to dump
   `print.print_statistics()` (the four fields, `%.17g`) plus the normal time.
   The store path and sha are recorded here.
2. **Fixtures:**
   - The existing preset-slice 3MF.
   - A two-filament plate with non-zero `filament_cost`, distinct densities
     and `time_cost > 0`.
   - A plate where one configured filament is unused, to cover the skip
     branch.
3. **Core check:** an integration test in `crates/ares-core/tests/` compares the
   four totals bit for bit (f64) against the probe output. RED is an
   assertion failure on missing or zero values, not a compile failure.
4. **WASM check:** a wasm test asserts the JSON round-trips the exact f64 bits.
   Floats are parsed with `str::parse::<f64>`, as JS does.
5. **E2E:** after the slice, select Summary and assert the three legend rows
   are exactly the oracle values formatted by upstream's printf. The
   artifacts are the oracle JSON and dark and light screenshots.
6. **Gates:** fmt, clippy `-D warnings`, `cargo nextest run --workspace`. The
   seeded full set and official same-round base/candidate run, plus
   profile_entry, because core changes; the expectation is identical
   ares.gcode bytes per case. Then web build and lint, and the full web E2E in
   a quiet window. `bundle.spec` must still pass with the new code in the
   lazy chunk only.

## Ways it could fail (checked by the evidence above)

- **Volume source:** using processor volumes from the wrong stage, the
  pre-finalize `update_print_stats_and_format_filament_stats` (`GCode.cpp:2313`)
  instead of `:1904`.
- **Precision:** f32 instead of f64 per-filament values, or summing in a
  different order. Upstream iterates a `std::map`, ascending by extruder id.
- **Time cost:** missing it, or using the stealth time.
- **Skip branch:** counting volume from unused extruders in weight or cost.
- **Formatting:** JS rounding at the `%.2f` boundary. `printf` uses the
  existing port in `toolPositionFormat.ts`.
