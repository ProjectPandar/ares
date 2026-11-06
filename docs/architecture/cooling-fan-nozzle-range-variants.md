# Cooling fan and nozzle temperature range filament variants

Rewrite slice of five entries of `filament_options_with_variant` (`PrintConfig.cpp:10150-10221`) at OrcaSlicer v2.5.0-beta `345efec64`, together with their selection by the filament variant gather (`PrintApply.cpp:1453-1455`):
- **Cooling fan:** `fan_min_speed`, `fan_max_speed`, `additional_cooling_fan_speed` (`:10207-10209`).
- **Nozzle temperature range:** `nozzle_temperature_range_low`, `nozzle_temperature_range_high` (`:10214-10215`).

## Problem
Ares's `filament_options_with_variant` (`options/registry/key_lists.rs`) and its typed filament gather (`options/project_variants/filament.rs`) both lack these five keys.
- **Projects listing one filament for several variant slots:** for example `filament_self_index = [1,1,1,1]` on a four-slot Ender-3. Native lays the keys out to every slot at load (`PrintConfig.cpp:11431-11457`), so its footer has `fan_max_speed = 100,100,100,100`. Ares skips keys that are not in the list and writes `100`.
- **Adding the keys to the list alone breaks other inputs:** the project refresh then lays out the refreshed system filament's variants for them, but no selection follows.
  - On `bbl-type1-tower/two-categories.3mf`, `fan_max_speed` becomes `100,100,30,30,100` instead of Native's `100,30`.
  - Filament 2 then cools at 100 % instead of 30 %.

## Upstream boundary
- **`filament_options_with_variant`** (`PrintConfig.cpp:10150-10221`): this one set drives the load layout (`:11431-11457`), the refresh merge (`OrcaSlicer.cpp:3563-3570`, `:3658-3793`) and the gather.
- **`Print::apply`** (`PrintApply.cpp:1452-1455`): `update_values_to_printer_extruders_for_multiple_filaments` selects every key of the set, plus `filament_self_index`, to one value per filament.
- **Consumers** read the selected value per filament:
  - Fan keys: `CoolingBuffer.cpp:769-820`, `ToolOrdering.cpp:1322-1330`, and the placeholders `GCode.cpp:10806-10808`.
  - `nozzle_temperature_range_high`: interface and flush temperatures (`GCode.cpp:1377`, `:1428`, `:1757`, `:11133`, `:11172`, `WipeTower2.cpp:1149`).
  - `nozzle_temperature_range_low`: `Print.cpp:1878`, `:1917` and the placeholder `GCode.cpp:10813`.

## Ares destination
- **`options/registry/key_lists.rs`:** `FILAMENT_OPTIONS_WITH_VARIANT_KEYS` gains the five keys. The load layout, the refresh merge and the defaults completion then treat them as upstream does.
- **`options/project_variants/filament.rs`:** `materialize` selects the five typed fields (`settings.filament.print.*`) like the other per-variant keys.

Both changes land in one commit: the list without the selection changes G-code output.

## Included and deferred
- **Included:** the five keys in the list and in the selection.
- **Deferred:** the remaining 21 beta keys missing from the list (pressure advance, pre-cooling, preheat, ramming, filament retraction). Their owners are listed in `load-filament-variant-normalize.md`.

## Oracle and RED
`crates/ares-cli/tests/orca_parity/option_coverage/cooling_fan_nozzle_range_variants.rs` is a `#[path]` child of `option_coverage.rs`. It uses the labelled footer check in `variant_footer.rs`, through the API and the CLI.
- **`auto-brim-object-adhesion/auto_brim-pla.3mf` and `skirt-minlength-multitool/tools2-minimum10.3mf`:** Ender-3, `filament_self_index = [1,1,1,1]`.
  - Native writes four values for each key: `70`, `100`, `100`, `230` and `190`.
  - Main writes one, so these tests fail on main.
- **`bbl-type1-tower/two-categories.3mf`:** two filaments with `filament_self_index = [1,2]`.
  - Native writes `fan_max_speed = 100,30`, `fan_min_speed = 100,20`, `additional_cooling_fan_speed = 70,70`, `nozzle_temperature_range_high = 240,240` and `nozzle_temperature_range_low = 190,190`.
  - Main matches. The test guards the selection: with the list change alone it fails.

Expected output changes, from a re-slice of the official catalog-path inputs against `2577ff79c`:
- 12 outputs change, all footer lines: the Ender-3 four-slot inputs (`auto-brim-adhesion-*`, `auto-brim-derived-multi-material`, `no-brim-adhesion-*`, `skirt-minlength-multitool-*`, `load-filament-variant-normalize-*`).
- Each moves the five keys to Native.
- No line moves away from Native, and no G-code body line changes.
