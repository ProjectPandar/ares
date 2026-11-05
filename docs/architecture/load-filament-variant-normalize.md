# Load-time filament variant layout

Rewrite slice of `normalize_filament_values_to_variants` (`PrintConfig.cpp:11431-11457`) and its two load-time call sites at OrcaSlicer v2.5.0-beta `345efec64`:
1. **3MF project load:** `bbs_3mf.cpp:2806` → `ConfigBase::load_from_json` (`Config.cpp:1149`) → `PrintConfigDef::handle_legacy_composite` (`PrintConfig.cpp:10094-10096`).
2. **CLI defaults completion:** `CLI::run` (`OrcaSlicer.cpp:4250-4259`).

## Problem
Some projects list every filament once in `filament_self_index`, for example `[1,1,1,1]` on a four-slot Ender-3, or `[1,1]` on an MK4. Such a project stores most per-variant filament options as a single value.
- **Native** lays each of those options out to one value per variant slot. The footer has `activate_air_filtration = 0,0,0,0`.
- **Ares** keeps the single value. The footer has `activate_air_filtration = 0`.

## Upstream boundary
- **`normalize_filament_values_to_variants`** (`PrintConfig.cpp:11431-11457`):
  - Let `filament_count` be the largest `filament_self_index` entry.
  - The function returns unless `0 < filament_count < len(filament_self_index)`.
  - It then visits each `filament_options_with_variant` key whose vector holds `filament_count` values or exactly one value.
  - Slot `v` receives `values[self_index[v] - 1]`, or `values[0]` for a single value. `map_variant_indices` (`PrintConfig.cpp:817-831`) with empty source lists gives `id - 1` or `0`. `set_at` (`Config.hpp:451-471`) reads an out-of-range index through `get_at`, which returns the first value.
  - The result holds one value per slot of `filament_self_index`.
- **Call 1 — `handle_legacy_composite`** (`PrintConfig.cpp:10094-10096`) runs at the end of `load_from_json` on the embedded project config. It lays out the values the project stores.
- **Call 2 — `CLI::run`** (`OrcaSlicer.cpp:4250-4259`) applies `FullPrintConfig` defaults to `m_print_config` and normalizes again. This lays out options the project does not store, which arrive as a single default value. It also covers command-line overrides, which `OrcaSlicer.cpp:4168` applied earlier.
- **The project refresh merge** (`OrcaSlicer.cpp:3563-3570`) first creates every per-variant option a loaded filament defines, then makes the same call.

A seeded Native replay of `auto-brim-adhesion-pla`, observed with breakpoints only (output byte-identical to the unobserved run apart from the timestamp), shows the following:
- Only two calls get past the early return: `bbs_3mf.cpp:2806` and `OrcaSlicer.cpp:4259`.
- Both calls see `filament_self_index = [1,1,1,1]`.
- The variant gather (`PrintApply.cpp:1452`) is not entered.

## Ares destination
- **`project/raw_settings/filament_variants.rs`** (new): `ProjectSettingsRaw::normalize_filament_values_to_variants` is the single port of `PrintConfig.cpp:11431-11457`. `ProjectSettingsRaw::complete_filament_variant_defaults` (`OrcaSlicer.cpp:4250-4259`) inserts the preset default of every absent `filament_options_with_variant` key, then normalizes.
- **`project/load.rs`:**
  - Call 1 goes after the embedded-config wire substitutions, which closes the `load_from_json` pass.
  - Call 2 goes after the overrides and vector-shape passes, before `ProjectSettings` is built.
- **`project/domain/preset_refresh.rs`:** call 2 goes before the refreshed `ProjectSettings` is built.
- **`profiles/project_refresh/filament.rs` `normalize_variants`:**
  - It keeps the default creation of `OrcaSlicer.cpp:3563-3569`.
  - For the layout it calls the shared function instead of its own expansion.
  - The shared port reads `filament_self_index` from the config itself, as upstream does. The removed expansion used the slot count.
- **`profiles/project_refresh/keys.rs`:** `default_value` becomes `pub(crate)`. `profiles` re-exports it so that call 2 uses the same `FullPrintConfig` default the refresh already seeds from.

Ares builds `ProjectSettings` while it loads the 3MF. The refreshed path loads, refreshes, then builds again, so call 2 runs before each build. In the refreshed path, the load-time run of call 2 precedes the refresh. Native creates the same keys inside the refresh merge (`OrcaSlicer.cpp:3563-3569`) whenever a filament is reloaded, because a loaded system filament defines every per-variant option. When no filament is reloaded, the refresh does not change the config, and the load-time call 2 stands in for `OrcaSlicer.cpp:4259`.

## Included and deferred
- **Included:** both load-time calls, and one shared port used by the load and by the refresh.
- **Deferred:**
  - **Key-list parity:** Ares's `filament_options_with_variant` (`options/registry/key_lists.rs`) lacks 26 keys of the beta list (`PrintConfig.cpp:10150-10221`). These include the pressure-advance, fan, ramming, pre-cooling and preheat keys. Adding them before the variant gather selects them leaves variant-length vectors in single-extruder outputs. Each gather slice adds its own keys to the list, and this layout then applies to those keys.
    - Pressure advance: `adaptive_pressure_advance`, `adaptive_pressure_advance_bridges`, `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs`, `enable_pressure_advance`, `pressure_advance`.
    - Cooling fan: `additional_cooling_fan_speed`, `fan_max_speed`, `fan_min_speed`.
    - Ramming and multi-tool purge: `filament_minimal_purge_on_wipe_tower`, `filament_multitool_ramming`, `filament_multitool_ramming_flow`, `filament_multitool_ramming_volume`, `filament_ramming_travel_time`, `filament_ramming_travel_time_nc`, `filament_ramming_volumetric_speed`, `filament_ramming_volumetric_speed_nc`.
    - Pre-cooling: `filament_pre_cooling_temperature`, `filament_pre_cooling_temperature_nc`.
    - Preheat: `filament_preheat_temperature_delta`.
    - Filament retraction: `filament_retract_after_wipe`, `filament_retract_length_nc`, `filament_retract_length_toolchange`, `filament_retract_restart_extra_toolchange`.
    - Nozzle temperature range: `nozzle_temperature_range_high`, `nozzle_temperature_range_low`.
  - **The opposite single-filament case:** `filament_self_index = [1]` on a Voron, where Native writes `0` and Ares writes `0,0,0,0,0,0`.

## Oracle and RED
`crates/ares-cli/tests/orca_parity/option_coverage/load_filament_variant_normalize.rs` is a `#[path]` child of `option_coverage.rs`. It slices through the API and the CLI and compares labelled footer lines with Native, using the preset catalog path. The keys checked are `activate_air_filtration`, `during_print_exhaust_fan_speed`, `nozzle_temperature` and `complete_print_exhaust_fan_speed`. The inputs:
- **`auto-brim-object-adhesion/auto_brim-pla.3mf`:** Ender-3, `filament_self_index = [1,1,1,1]`, every checked key stored once. Native writes four values per key; main writes one.
- **`skirt-minlength-multitool/tools2-minimum10.3mf`:** the same layout with two used tools.
- **`mk4-layer-change/mk4base.3mf`:** MK4, `filament_self_index = [1,1]`. Native writes two values; main writes one.
- **`load-filament-variant-normalize/absent-variant-keys.3mf`:** `auto_brim-pla.3mf` with the `activate_air_filtration` and `during_print_exhaust_fan_speed` blocks removed from `Metadata/project_settings.config`. Every other entry is byte-identical. The two keys reach Native only as defaults (call 2). Native writes `0,0,0,0` and `60,60,60,60`; main writes `0` and `60`.

Expected output changes, from a re-slice of 546 official catalog-path inputs against `cbdde9ee6`:
- 10 outputs change, all in footer lines: the nine Ender-3 four-slot inputs and `mk4-layer-change`.
- In each of those outputs, ten keys become equal to Native: `activate_air_filtration`, `activate_air_filtration_during_print`, `activate_air_filtration_on_completion`, `complete_print_exhaust_fan_speed`, `during_print_exhaust_fan_speed`, `long_retractions_when_ec`, `nozzle_temperature`, `nozzle_temperature_initial_layer`, `retraction_distances_when_ec` and `volumetric_speed_coefficients`.
- No line moves away from Native, and no G-code body line changes.
- The remaining footer differences in those outputs are keys outside Ares's list (deferred above).
