# Filament (ColorPrint) preview rewrite slice

## Source and destination

- `OrcaSlicer/src/libvgcode/src/ViewerImpl.cpp:873, 1017-1022, 1363-1370`:
  `m_used_extruders` / `get_used_extruders_ids` → `ares-vgcode::ViewerImpl`.
  Keep sorted extrude-only ids and clear them on reload. `update_colors`
  (`:1265-1273`) pads tool colors through the highest used id with `DUMMY_COLOR`.
- `src/slic3r/GUI/Plater.cpp:16889-16901` and `GUI_Preview.cpp:680-690`:
  processed `extruder_colors` supplies tool colors → `ares-wasm::SliceOutput`.
  Remove the caller tool-palette parameter, not a second/fallback palette source.
- `src/slic3r/GUI/GCodeViewer.cpp:3563-3625, 3746-3790, 4026-4141`:
  ColorPrint `render_legend` used extruders, `displayed_columns` and table order
  → `ares-vgcode::filament_legend`; volume statistics come from the core result.
  Dense vectors' zero/absent entries represent absent nonzero caches
  (`GCodeProcessor.cpp:1558-1600`).

## Non-negotiable boundary

Rust owns used extruders, column selection/order, tool colors and volume data.
WASM only serializes this data and forwards `GCodeProcessorResult`
`filament_diameters` / `filament_densities` (upstream load: `GCodeViewer.cpp:2622-2627`).
The TS package transports typed data and has no React/Three dependency. The SPA
alone converts mm³ to m/g (`:3443-3448`), sums the displayed units (float row
accumulators, double column totals), and formats them (`:4044-4141`). No TS
G-code parsing, statistics accumulation from moves, or slicer logic.

The UI extends the existing dense preview legend, using its colors, spacing and
swatches; Filament follows Line Type in the combo (`:1077-1093`). The component
and its CSS are lazy-loaded; the initial JS gzip budget remains 125,000 bytes.
This replaces the temporary FeatureType-only palette shell, not a new pipeline.

## Included and deferred

Included: used-filament rows, Model/Support/Flushed/Tower/Total headers, metric
usage cells, multiple-extruder Total row, result-derived path and swatch colors.
Deferred: custom ColorChange ranges/palettes and imperial unit preference.
The upstream ColorPrint row callback is empty (`:4097`); this slice does not
invent a per-filament visibility toggle under its Display header.

## Verification

Browser RED `3e64c5ef3` derives legend expectations from upstream processor moves
and G-code config, not browser observations. It checks headers, values, swatches,
recolor/restore and records a screenshot and JSON next to G-code hash artifacts.
A Rust public-boundary fixture additionally covers sparse multi-tool ids,
unused-tool statistics, every volume column, tool-color padding and reload.
Its updated raw-volume/metadata contract failed before implementation. Full
browser specs and Rust/workspace/profile reference gates must pass.

### SEMM fixtures

`tests/parity/orca_cli_ender3.3mf` and
`tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf`, ZIP member
`Metadata/project_settings.config`, both explicitly contain
`"single_extruder_multi_material": "1"`. The Ender3 browser output also
serializes `; single_extruder_multi_material = 1`; the LH profile-entry
3MF/STL and profile-slice outputs do too. `ironing_type` in these outputs is
`no ironing`. The E2E fixtures therefore enable SEMM, even though these
particular fixtures use one filament.

### Nonempty plates

The actual slice selections are nonempty. The SPA slices `scene.plates[0]?.id`
(`web/app/src/state/actions.ts`, `presets.ts`); the package consumer and profile
slice page omit the plate argument, selecting the first plate. Native fixture
oracles explicitly select plate 1.

`Metadata/model_settings.config` in the checked-in 3MF fixtures records:

| Fixture | First `plater_id` | Object/instance references on that plate |
| --- | --- | --- |
| `tests/parity/orca_cli_ender3.3mf` | 1 | (2, 0) |
| `tests/parity/arachne/default-prisms.3mf` | 1 | (2, 0), (4, 0), (6, 0) |
| `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` | 1 | (2, 0) |

The remaining sliced inputs are `tests/parity/cube10.stl` or the native-exported
`profile-entry.3mf` / `override.3mf` built from that cube
(`crates/ares-cli/tests/orca_parity/runner/profile_entry_3mf.rs:54-83, 209-238`).
Sidebar/config edits preserve these model/plate assignments. The full browser
suite asserts successful slices and compares the resulting G-code to native
outputs; it does not select an empty plate.