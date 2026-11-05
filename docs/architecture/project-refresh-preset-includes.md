# Project refresh applies vendor preset includes

Rewrite slice of the include layering in vendor preset resolution (`PresetBundle.cpp:7627-7640`), as used by the CLI filament refresh (`OrcaSlicer.cpp:2800-2807`), together with the `filament_self_index` remap of the multiple-filament variant gather (`PrintConfig.cpp:11986-11992`), at OrcaSlicer v2.5.0-beta `345efec64`.

## Problem
`Bambu PLA Basic @BBL X1C` includes `fdm_filament_template_direct_dual`. That template sets two filament variants ("Direct Drive Standard", "Direct Drive High Flow") and `filament_preheat_temperature_delta = 10,10`; no preset in the inherits chain sets that key.
- **Native:** the refreshed filament has two variants. `Print::apply` gathers them to the X1C's extruder and remaps `filament_self_index`. The footer has `filament_preheat_temperature_delta = 10` and `filament_self_index = 1`.
- **Ares:** the project refresh seeds the system filament from its inherits chain without the include. The footer has `filament_preheat_temperature_delta = 0`.

## Upstream boundary
- **Vendor preset resolution** (`PresetBundle.cpp:7627-7640`): a preset starts from its parent (or the collection default). Each listed include's recorded diff is applied in order (`:7632-7639`), then the preset's own keys (`:7640`).
- **CLI filament refresh** (`OrcaSlicer.cpp:2800-2807`): `resolve_system_preset` (`PresetBundle.cpp:724-757`) returns that resolved config. The merge (`OrcaSlicer.cpp:3635-3768`) expands the project slot to the preset's variant count. `filament_self_index` is rebuilt per variant (`:3836-3846`).
- **Variant gather** (`PrintApply.cpp:1436-1455`): `filament_keys` is `filament_options_with_variant` plus `filament_self_index`. On a single-extruder printer it runs once a filament has more variants than filaments (`has_multi_variant_filament`, `PrintConfig.cpp:10829-10834`). `update_values_to_printer_extruders_for_multiple_filaments` (`PrintConfig.cpp:11884-11995`) gathers every key to one value per filament. It then remaps the id list itself: `new_values[f] = ids[variant_index[f]]` (`:11986-11992`). `Print::update_filament_maps_to_config` (`Print.cpp:4722-4726`) repeats it on the full config.

The seeded Native replay of `avoid-crossing-external-frame-x1c-detour0`, observed with breakpoints only, shows the following. Its output is byte-identical to the unobserved run, apart from the timestamp.
- Both gather calls (`PrintApply.cpp:1454`, `Print.cpp:4726`) receive two variants: `filament_extruder_variant` length 2, `filament_preheat_temperature_delta = [10, 10]`, `filament_flow_ratio = [0.98, 0.98]`.
- The id remap runs (`has_id_key`) and produces `[1]`.

## Ares destination
- **`profiles/project_refresh/preset.rs` `system_config`:** layer `Catalog::resolved_layers(position)` (the existing `catalog/includes.rs` port of `:7627-7640`) instead of the raw inherits-chain documents.
- **`options/project_variants.rs`:** after `filament::materialize` in `materialize_project_variants` and in `refresh_filament_map`, remap `project.preset.filament_self_index` to the id at each filament's selected variant slot (`PrintConfig.cpp:11986-11992`). `filament::materialize` already gathers the `filament_options_with_variant` keys; only the id remap is missing. Before the include reached the refresh, every affected slot had one variant, so the missing remap was invisible.
- `profiles/project_refresh/keys.rs` and `options/project_variants/filament.rs` do not change.

## Included and deferred
- **Included:** includes in the project refresh seed, and the `filament_self_index` remap of the multiple-filament gather.
- **Deferred:**
  - **The gather activation condition.** Ares activates the gather from the printer variant tokens. Native's `extruder_count > 1 || different_extruder || has_multi_variant_filament()` gate (`PrintApply.cpp:1452`) is not ported here.
  - **Variant arity of footer keys that are not gathered** (for example `filament_preheat_temperature_delta = 0,0` vs `0` on J1, Ender-3 and Kobra 3 Max).

## Oracle and RED
`crates/ares-cli/tests/orca_parity/option_coverage/refresh_preset_includes.rs` (a `#[path]` child of `option_coverage.rs`) slices two checked-in inputs through the API and the CLI. It compares labelled footer lines with Native, using the preset catalog path:
- **`avoid-crossing-external-frame/x1c-detour0.3mf`** (the X1C replay witness above): `filament_extruder_variant`, `filament_preheat_temperature_delta` and `filament_self_index`. On main, preheat is `0` where Native has `10`.
- **`variant-retraction/l59.3mf`** (Ender-3, two filaments on one physical extruder): `filament_self_index`. On main it is `1,1,2,2` where Native has `1,2`.

The X1C `filament_self_index` line guards against the include layering alone, which yields `1,1`. No whole-output test changes status: the affected inputs still differ from Native in other footer lines (for example `hotend_cooling_rate` and `machine_max_junction_deviation` arity).

Expected output changes, from a re-slice of 546 official catalog-path inputs:
- 65 Ares outputs change, and only in footer lines.
- 60 gain `filament_preheat_temperature_delta` equal to Native (BBL X1C, P1S, X2D).
- 5 gain `filament_self_index` equal to Native (`1,1,2,2` → `1,2` on the Ender-3 `variant-retraction-*` and `nozzle-temperature-footer-variant-column` inputs).
- No line moves away from Native, and no G-code body line changes.
