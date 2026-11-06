# Filament variant selection on a single-variant printer

Rewrite slice of the filament variant gate in `Print::apply` (`PrintApply.cpp:1421-1455`) and in the filament map refresh (`Print.cpp:4679-4684`, `:4725-4745`) at OrcaSlicer v2.5.0-beta `345efec64`. The gate is `has_multi_variant_filament` (`PrintConfig.cpp:10829-10834`).

## Problem
A Voron 2.4 project lists one filament, `Generic PLA @System`, with `filament_self_index = [1]`. The CLI refresh loads that system filament. The preset defines six variants in `filament_extruder_variant`: Direct Drive and Bowden, each in Standard, High Flow and Extra High Flow.
- **Both Native and Ares** rebuild `filament_self_index` as `[1,1,1,1,1,1]` (`OrcaSlicer.cpp:3836-3845`). Every per-variant filament option then holds six values.
- **Native** prints `filament_self_index = 1` and `filament_max_volumetric_speed = 12` (the Direct Drive Standard value).
- **Ares** prints `1,1,1,1,1,1` and `12,21,40,12,21,30`.

The same layout gives six values to any key added to `filament_options_with_variant`, so the difference grows with each new per-variant key.

## Upstream boundary
- **`Print::apply`** (`PrintApply.cpp:1421-1455`):
  - Printer and process variants are selected only when `(extruder_count > 1) || different_extruder` (`:1424-1430`). Otherwise `print_variant_index` is `[0]` (`:1431-1432`).
  - After the `m_ori_full_print_config` snapshot (`:1434`), filament variants are selected when `(extruder_count > 1) || different_extruder || new_full_config.has_multi_variant_filament()` (`:1453-1455`). The selection covers every `filament_options_with_variant` key and `filament_self_index`.
  - `GCode::append_full_config` (`GCode.cpp:8244-8246`) prints `print.full_print_config()`, which is the selected config (`PrintApply.cpp:1676`).
- **`has_multi_variant_filament`** (`PrintConfig.cpp:10829-10834`): true when `filament_extruder_variant` holds more values than `filament_diameter`, that is, when some filament defines more than one variant.
- **Filament map refresh** (`Print.cpp:4679-4684`): `expand_filaments` uses the same three-way condition. It gates the selection from `m_full_print_config` (`:4725-4726`) and its application to `m_config` (`:4742-4746`).

## Ares destination
- **`options/project_variants/index.rs`:** `resolve_activation` returns `None` only when the extruder condition and `has_multi_variant_filament` are both false. When it returns an activation, the result also records whether the extruder condition held.
- **`options/project_variants.rs`:**
  - `materialize_project_variants` selects printer and process variants only under the extruder condition, and otherwise uses process variant index `[0]`. It then selects the filament variants under the activation.
  - `refresh_filament_map` uses the same activation for its filament selection.

The existing typed selection in `options/project_variants/filament.rs` is unchanged. It already covers every key in Ares's `filament_options_with_variant` list. The change only decides when that selection runs, so every key a later slice adds to both the list and the selection is covered on single-variant printers without further changes here.

## Included and deferred
- **Included:** the `has_multi_variant_filament` arm of both gates, and the split of the printer/process selection from the filament selection.
- **Deferred:**
  - The layered nozzle group branch (`PrintApply.cpp:1444-1450`).
  - The 26 beta keys still missing from Ares's `filament_options_with_variant` (see `load-filament-variant-normalize.md`).

## Oracle and RED
`crates/ares-cli/tests/orca_parity/option_coverage/single_variant_printer_filament_selection.rs` is a `#[path]` child of `option_coverage.rs`. It slices through the API and the CLI and compares labelled footer lines with Native. The keys checked are `filament_self_index`, `filament_extruder_variant`, `filament_max_volumetric_speed`, `activate_air_filtration` and `nozzle_temperature`. The inputs:
- **`acceleration-raw-f64/voron-outer1234_5.3mf`:** Voron 2.4, one filament, a six-variant system filament after the refresh. Native writes `1`, `"Direct Drive Standard"`, `12`, `0`, `220`. Main writes six values for each.
- **`scarf-clip-end/voron-all-inner0.3mf`:** the same printer and filament with other process settings.

Expected output changes, from a re-slice of the official catalog-path inputs against `64ffad3b2`:
- 3 outputs change, all in footer lines: `acceleration-raw-f64-voron-outer1234`, `initial-travel-uint-voron-travel1234` and `scarf-clip-end-voron-all-inner0`.
- Each becomes identical to Native, apart from the generation timestamp.
- 18 keys per output move toward Native. No line moves away from Native, and no G-code body line changes.
- Their whole-output tests (`voron_fractional_outer_acceleration_percentage_reference`, `voron_fractional_initial_travel_acceleration`, `voron_scarf_clip_end_displacement`) fail on main, official and seeded, at the first differing footer line.
