# Web preview: Summary view

Data-source ruling: option A.

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
| `Utils.hpp:494-537` `short_time` and `:540-` `get_time_dhms` (f32) | Time format | Superseded by the estimation footer ruling (`web-estimation.md`): `short_time`/`get_time_dhms` are display-only TS ports in `toolPositionFormat.ts`; there is no Rust binding. |
| `ViewerImpl.cpp:1543-1548` / `crates/ares-vgcode/src/viewer/colors.rs:243` | Summary colors vertices like ColorPrint | Already ported; `VIEW_TYPES` already has Summary at slot 0 |
| `GCodeViewer.cpp:66-69` | Label "Summary" | `VIEW_TYPE_ITEMS` entry, first in combo order |

### Shared data source, independent formula (ruled: option A)

The four values reuse the data source, not the material-statistics
calculation. They read the same `estimate.print_statistics` that
`processor/material_statistics.rs::MaterialStatistics::new` reads: the same
object from the same finalize, so volumes are never estimated twice.

Two fields come from that object:
- `total_volumes_per_extruder`;
- `modes[Normal].time`.

The formula is a separate port of `GCode.cpp:1904-1940`. `MaterialStatistics`
ports a different upstream function (`GCodeProcessor::post_process`,
`GCodeProcessor.cpp:993-1008`, which produces the G-code tail lines), and
reusing its totals would drift from upstream in three ways:

| | `post_process` → `MaterialStatistics` | `update_print_estimated_stats` → Summary |
|---|---|---|
| Per-filament values | `m_result.filament_diameters/densities/costs`, `std::vector<float>` (`GCodeProcessor.hpp:252-255`) | `Extruder::filament_*()` → `double get_at(id)` (`Extruder.cpp:148-161`) |
| `time_cost` | not added | `+= time_cost * normal_time / 3600` |
| Extruders not in `m_writer.extruders()` | counted | skipped for used filament, weight and cost |

For example, density 1.24 is 1.2400000095367432 in f32, so reused totals
would differ in the low bits. Cost would also be wrong whenever
`time_cost > 0`.

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

Ruled: `update_print_estimated_stats` lives in ares-core; ares-wasm only binds
it. It is a read-only post-processor value and changes no slicing or G-code
output.

## Oracle and evidence

1. **Probe:** `tools/chain-probe/print-statistics.nix` builds a probe from the
   2.4.2 source of the seeded replay producer (`1k0p5nlb…`, source store path
   `g3wj93ly…-orca-slicer-2.4.2-seed-replay-source`; its `GCode.cpp` and
   `OrcaSlicer.cpp` match `OrcaSlicer/src`). It patches the CLI after export to dump
   `print.print_statistics()` (the four fields, `%.17g`) plus the normal time
   into the `--outputdir`. Its output is never committed: tests run it
   through `OrcaRunner` (`ARES_PRINT_STATISTICS_BIN`, cached under
   `ARES_ORCA_CACHE_DIR`) in a process pinned to one CPU, since a
   multi-threaded Native takes a different travel order than the single-CPU
   reference Ares matches.
2. **Fixtures:**
   - The existing preset-slice 3MF.
   - A two-filament plate with non-zero `filament_cost`, distinct densities
     and `time_cost > 0`.
   - A plate where one configured filament is unused, to cover the skip
     branch.
3. **Core check:** `crates/ares-cli/tests/orca_parity/print_estimated_stats.rs`
   compares the four totals bit for bit (f64) against native `PrintStatistics`
   from the probe; `ARES_REQUIRE_ORCA=1` fails without the probe, otherwise
   it prints `SKIPPED`. RED covers at least the `time_cost > 0` fixture and the
   unused-filament fixture. RED is an
   assertion failure on missing or zero values, not a compile failure.
4. **WASM check:** a wasm test asserts the JSON round-trips the exact f64 bits.
   Floats are parsed with `str::parse::<f64>`, as JS does.
5. **E2E:** after the slice, select Summary and assert the three legend rows
   are exactly the oracle values formatted by upstream's printf. The
   artifacts are the oracle JSON and dark and light screenshots.
6. **Gates:** fmt, clippy `-D warnings`, `cargo nextest run --workspace`. The
   seeded full set and official same-round base/candidate run, plus
   profile_entry, because core changes; the expectation is identical
   ares.gcode bytes per case. Then web build and lint, and the full web E2E.
   `bundle.spec` must still pass with the new code in the
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

## Web slice: Summary combo entry and legend rows

Web plus the ares-wasm adapter's `VIEW_TYPES`, with the view type ruling
below; no ares-core or ares-vgcode change.

The core part above is in place. The legend
estimation footer slice (`web-estimation.md`) delivers
`Toolpaths.estimation` with `totalUsedFilament`, `totalWeight`, `totalCost`
(`PrintStatistics`) and `time` (`modes[Normal].time`). ares-vgcode already
colors Summary like ColorPrint (`crates/ares-vgcode/src/viewer/colors.rs:243`,
`:276`), but the wasm adapter does not offer it: `viewTypes()` and
`vertexColors` come from `crates/ares-wasm/src/colors.rs` `VIEW_TYPES`, which
has 15 entries without Summary, and the web combo filters `VIEW_TYPE_ITEMS`
by `viewTypes()`. The web app has no Summary entry and no Summary rows.

View type ruling: `VIEW_TYPES` gains `ViewType::Summary` first (`EViewType`
order, `GCodeViewer.cpp:1076-1077`). `crates/ares-wasm/src` is production
Rust (the wasm adapter), but orca_parity does not depend on ares-wasm, so the
full reference run is not needed.

| Upstream | What | Ares destination |
|---|---|---|
| `GCodeViewer.cpp:66-69` `get_view_type_string` | Label "Summary" | `VIEW_TYPE_ITEMS` entry |
| `GCodeViewer.cpp:1076-1077` `view_type_items` | Summary is first in the combo | First in `VIEW_TYPE_ITEMS` |
| `GCodeViewer.cpp:1049` | The default view stays FeatureType | Unchanged |
| `GCodeViewer.cpp:4003-4024` `case EViewType::Summary` | `Total:` `"%.2f m / %s"` (`ps.total_used_filament / koef`, `format_compact_weight(ps.total_weight)`), `Cost:` `"%.2f"`, `Total time:` `short_time(get_time_dhms(time_mode.time))` | `web/app/src/preview/LegendSummary.tsx`, lazy preview chunk |
| `:3558` `ps`, `:3559` `koef`, `:3147` `time_mode` | Inputs | `toolpaths.estimation` |
| `:4547-4644` footer | Drawn for every view (`show_estimated_time`, `:3152`, is never read). Outside FeatureType it is "Time Estimation" with the time rows only | Existing `EstimationRows`, unchanged |

**Included:** the Summary combo entry, first; the three rows, metric; Normal
mode; Summary in the wasm `VIEW_TYPES`, so `vertexColors("Summary")` uses the
ported vgcode colors.

**Deferred:** imperial units, stealth mode and the mode button, the
multi-nozzle block (`:4683`), the all-plates Total Estimation
(`:2550-2810`), the Time Estimation window.

### Evidence

`web/e2e/estimation.spec.ts` with the Native probe
(`ARES_PRINT_STATISTICS_BIN`, pinned to the CPU `ORCA_REPLAY_CPU` names; unset fails) and the two `tests/parity/summary` fixtures (`time-cost`, `unused-filament`),
per fixture:

1. The `view-type` option values, read without waiting, equal upstream's
   `view_type_items` order filtered to the engine's `viewTypes`. RED is a
   deep-equality failure, not a `selectOption` timeout.
2. With Summary selected, the `legend-summary` rows equal `Total:`
   `printf(total_used_filament / 1000, 2) m / weight(total_weight)`, `Cost:`
   `printf(total_cost, 2)` and `Total time:` Native `footer_times[2]`
   (upstream's own `short_time(get_time_dhms(...))` from the probe).
3. The footer for Summary is "Time Estimation" with time rows only.
4. Artifacts: `summary` rows in `result.json`, `legend-summary-{dark,light}.png`.

Ways it could fail: `"  "` instead of `" / "`; Summary appended last or made
the default; the model time (`time - prepare`) or stealth time instead of
the total; `toFixed` instead of the printf port at `%.2f`; missing rows
surfacing as a timeout; the new code landing in the initial chunk
(`bundle.spec`, gzip < 125000).

The wasm tests that pin `view_types()` (`crates/ares-wasm/tests/actual_speed.rs`,
`motion_attributes.rs`) expect Summary first; on the old `VIEW_TYPES` that is an
assertion failure. Their other assertions are unchanged.

Gates: fmt, clippy `--workspace --all-targets -D warnings`, wasm32
check, `nextest -p ares-wasm -p ares-vgcode`; web build, lint and
typecheck; browser base and candidate with identical
arguments (the full web E2E). No workspace or full reference run.
