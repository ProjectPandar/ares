# Web preview: legend estimation footer

The next slice
after this one is S2, the ToolPosition actual-speed profile.

## Upstream boundary

The footer is the last block of `GCodeViewer::render_legend`
(`GCodeViewer.cpp:3111`), drawn after the view-type rows for every view type
(`:4547-4644`). The web legend (`web/app/src/preview/Legend.tsx`) has no part
of it.

| Upstream | What | Ares destination |
|---|---|---|
| `GCodeViewer.cpp:4551-4570` | Title `Total Estimation` for FeatureType and `Time Estimation` for every other view. `[Normal mode]` is appended only when `can_show_mode_button`, that is when Normal and Stealth times format to different strings | lazy legend chunk, `web/app/src/preview/` |
| `GCodeViewer.cpp:4596-4626` (`ps`/`koef` at `:3558-3560`) | FeatureType only: `Total Filament: %.2f m  <format_compact_weight(ps.total_weight)>`; `Model Filament:` the same values minus support, flush and wipe tower usage; `Cost: %.2f` of `ps.total_cost` | same |
| `GCodeViewer.cpp:4627-4644` | `Prepare time:` shown only when `prepare_time != 0`; `Model printing time: short_time(get_time_dhms(time - prepare_time))`; `Total time: short_time(get_time_dhms(time))` | same |
| `GCodeViewer.cpp:3541-3625` | Support, flush and wipe-tower metres and grams: per used extruder, through `get_used_filament_from_volume` (`:3443-3447`, f32 `m_filament_diameters`/`m_filament_densities`) | same; it reuses the `LegendFilament.tsx` arithmetic and the transported volume vectors |
| `Print.hpp:789-816` `PrintStatistics`, filled by `GCode.cpp:1904-1940` `DoExport::update_print_estimated_stats` | `total_used_filament`, `total_weight`, `total_cost` (`ps`) | already in ares-core (`estimated_stats.rs`, `8a0116f54`) and carried on `ProcessedGCode::print_statistics`, but not yet bound in ares-wasm. Binding it is part of this slice |
| `GCodeProcessor.hpp:56-67` `PrintEstimatedStatistics::Mode::prepare_time`; `GCodeProcessor.cpp:475-476` (f32 accumulation over `flags.prepare_stage` blocks), `:2727-2730` `get_prepare_time`, `:6016-6022` `update_estimated_times_stats` | Normal-mode `prepare_time` (f32) | ares-core: new `PrintEstimatedStatistics::normal_prepare_time: f32`, next to the existing `normal_time`. The value is `estimate.prepare`, which ares-core already computes in f32 for the G-code header (`processor/estimate.rs:368-376`) |
| `Utils.hpp:494-537` `short_time`, `:540-562` `get_time_dhms` | Time text | TS display port. The f32 dhms decomposition already exists in `toolPositionFormat.ts::formatMarkerTime`; it is factored out and reused |
| `GCodeViewer.cpp:128-150` `format_compact_weight` | Weight text | existing `LegendFilament.tsx` port |

## Scope rules

- The ares-core change is limited to that statistics port: one f32 field set
  from an already computed value. It changes no slicing and no G-code bytes;
  the full pair is expected to show identical `ares.gcode` per case.
- ares-wasm only serializes `PrintStatistics` (JSON, the same shape as
  `takePrintStatistics`) and the new field. It has no logic.
- Core and wasm form one commit pair (RED, then GREEN). Web forms a separate
  pair (RED, then GREEN).
- This is a production advance. It needs workspace, strict and the full
  official/seeded pair.

## Included and deferred

**Included:**
- The title.
- The FeatureType rows Total Filament, Model Filament and Cost.
- Prepare time, Model printing time and Total time for every view type.
- Normal mode, metric units.

**Deferred:**
- Stealth mode: the Stealth time, the `[Normal mode]` suffix and the "Show
  stealth mode" button (`:4646-4670`). Ares has no stealth machine yet, and the fixtures
  must have Native `stealth_time == 0`, which the oracle test asserts.
- Imperial units.
- The multi-nozzle recommendation block (`:4682-4684`).
- The ColorPrint Options/Display table (`:4672-4680`).
- The Summary legend rows (`:4003-4024`). The Summary web part was never
  built on top of core `8a0116f54`; it reuses the binding added here and
  can follow as its own small slice.

## Ways it could fail

- **Time source:** using the f64 machine time or a stealth value instead of
  `modes[Normal]` f32 `time` and `prepare_time`.
- **Model printing time:** the subtraction must be done in f32
  (`time_mode.time - time_mode.prepare_time`, both float).
- **Prepare row:** shown when `prepare_time == 0`, or hidden when it is not
  zero.
- **Mixed precision in Model Filament:** upstream subtracts the f32-based
  support, flush and wipe-tower metres from the f64
  `ps.total_used_filament / 1000`. Reusing PrintStatistics' f64 per-filament
  values for the subtrahend would differ in the low bits.
- **Extruder set:** only `get_used_extruders_ids()` contribute. A volume on an
  unused id must not be subtracted.
- **Title:** for example, using "Total Estimation" for every view type.
- **Formatting:** `%.2f` half-to-even (`printf` port); `short_time`'s
  rounding to whole minutes when days or hours are present; the `<1s` and
  `0s` branches.
- **Bundle:** the new code must live in the lazy legend chunk; `bundle.spec`
  initial gzip must stay strictly below 125,000 B.

## RED-first plan and evidence

1. **Native oracle, generated at runtime and never stored.** Extend
   `tools/chain-probe/print-statistics.patch` to also write
   `es.modes[0].prepare_time` (`%.9g`) into `plate_1.print_statistics.json`.
   The probe is rebuilt with Nix, and its output stays in the `ARES_ORCA_CACHE_DIR` cache.
2. **Fixtures:** `tests/parity/summary/{time-cost,unused-filament}.3mf`.
   Native `prepare_time > 0` and `stealth_time == 0` are asserted.
   If neither fixture has a nonzero prepare stage, the BBL preset-slice 3MF
   is added, since its start G-code is tagged Custom before the first move.
3. **Core RED:** `crates/ares-cli/tests/orca_parity/print_estimated_stats.rs`
   compares `normal_prepare_time` to Native f32 bits. It must fail as an
   assertion (field present and zero), not as a compile error. With
   `ARES_REQUIRE_ORCA=1` a missing producer fails.
4. **Core GREEN:** set the field; the wasm getter serializes
   `PrintStatistics`. A wasm test round-trips the exact f64/f32 bits.
5. **Web RED:** a preview E2E reads the oracle JSON (runtime Native, same
   cache) and the volumes, and asserts the exact footer text:
   - In FeatureType: title, Total and Model Filament, Cost, Prepare, Model
     printing and Total time.
   - In one other view: the `Time Estimation` title and only the three time
     rows.
   - The fixtures include a nonzero support or wipe-tower volume, so that
     Model Filament differs from Total Filament.
   - RED fails because the footer is missing.
6. **Web GREEN:** the footer is implemented. The artifacts are the oracle
   JSON, the expected and actual rows, and dark and light legend
   screenshots.
7. **Gates:**
   - Rust: fmt, clippy `-D warnings`, `cargo nextest run --workspace`, and
     the wasm32 build.
   - Parity: workspace, strict and the full official/seeded base/candidate
     pair, with identical G-code bytes expected.
   - Web: build and lint, then the full browser base/candidate pair, with
     wasm rebuilt on the current main. Screenshot diffs must stay in the
     known categories plus the legend footer.
