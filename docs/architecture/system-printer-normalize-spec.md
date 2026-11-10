# Normalized system printer preset in the project refresh

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths below are
relative to `src/libslic3r` unless they start with `src/`.

- `PresetBundle::resolve_vendor_preset` (`PresetBundle.cpp:7582-7755`) builds every vendor
  preset the system preset resolver returns. Its steps:
  1. it applies the preset's own keys over its parent;
  2. it pads the per-variant keys (`extend_default_config_length`, `:7649`);
  3. it drops the keys the collection does not define (`remove_invalid_keys`, `:7653`);
  4. it calls `Preset::normalize` (`:7666`).

  `PresetBundle::resolve_system_preset` (`:724-759`) returns that preset's config.
- `Preset::normalize` (`Preset.cpp:524-571`):
  - When `single_extruder_multi_material` exists and is false, it calls
    `set_num_extruders(nozzle_diameter.size())` (`:535-541`).
  - Otherwise (`:527-533`), it calls `set_num_filaments(filament_diameter.size())` when
    `filament_diameter` exists.
  - When `filament_diameter` exists, it resizes the filament option keys (`:544-568`).
- `DynamicPrintConfig::set_num_extruders` (`PrintConfig.cpp:10669-10686`):
  1. It calls `extend_extruder_variant(config, n)` (`:10630-10667`):
     - `extruder_variant_list` is created with `n` copies of `"Direct Drive Standard"` when
       missing, then resized to `n`;
     - `printer_extruder_id` and `printer_extruder_variant` are rebuilt. For each extruder
       `i` in `0..n`, the extruder's `extruder_variant_list` entry is split on `,` with
       `boost::token_compress_on`. Each token appends `i + 1` to `printer_extruder_id` and
       the token itself to `printer_extruder_variant`;
     - each `printer_options_with_variant_2` key that exists is resized to
       `get_parameter_size(key, n)`, padded from the defaults.
  2. Each `extruder_option_keys()` key except `default_filament_profile` is resized to
     `get_parameter_size(key, n)`, padded from the defaults (`:10673-10685`).
- `DynamicPrintConfig::get_parameter_size` (`PrintConfig.cpp:10598-10625`) gives the size of
  each kind of key:
  - `printer_options_with_variant_1` keys: the `printer_extruder_variant` size;
  - `printer_options_with_variant_2` keys: twice that size;
  - filament and process variant keys: their own variant sizes;
  - every other key: `n`.

  A missing variant key counts as size 1.
- `ConfigOptionVector::resize` (`Config.hpp:687-715`) truncates, or pads with the vector's
  first value. It pads from the default only when the vector is empty.

How the CLI reaches it, when it slices a project with `--slice` and no `--load-settings`:

- The project refresh `load_project_preset` (`src/OrcaSlicer.cpp:3167-3195`) loads the
  project's printer through `Preset::load_external_config` (`Preset.cpp:1059-1088`), with the
  resolved system preset as the base.
- It then copies into the print config every key the project lacks (`:3187-3194`). For a
  project without `extruder_variant_list` or `default_nozzle_volume_type`, those two values
  come from the normalized system printer.
- `Print::apply` (`PrintApply.cpp:1429`) then calls `update_values_to_printer_extruders` for
  the process keys. That function first runs `ensure_process_variant_columns`
  (`PrintConfig.cpp:11631-11659`, from `:11702-11703`). With two `extruder_variant_list`
  entries, `ensure_process_variant_columns` rebuilds `print_extruder_id` as `1,2`.

## Rust destination

- `crates/ares-core/src/profiles/project_refresh/preset.rs`: `system_config` stays the Rust
  form of the resolved system preset. It builds the defaults, the resolved layers filtered to
  the kind's keys, and the vector-shape normalization. For a machine preset it then calls
  `profiles/preset_normalization.rs::normalize_machine`.
- `normalize_machine` is the printer branch of `Preset::normalize` (`Preset.cpp:535-541`). It
  runs `SliceOptions::set_num_extruders` (`options/extruder_count.rs`), which calls
  `SliceOptions::extend_extruder_variant` (`options/extruder_variant.rs`), and writes the
  resized keys back into the raw preset. Those functions are the Rust form of:
  - `set_num_extruders`;
  - `extend_extruder_variant`;
  - `get_parameter_size`;
  - the `ConfigOptionVector::resize` padding.

  This slice adds no second copy of them. Its test is the witness that the normalized system
  printer reaches the project refresh.
- `ensure_process_variant_columns` is already ported
  (`options/project_variants/index.rs`) and is not changed. It produces `print_extruder_id`
  once the printer's `extruder_variant_list` has two entries.

No processor, G-code or export code changes.

## Included

- The printer branch of `Preset::normalize`: `single_extruder_multi_material` is present and
  `"0"`, and `nozzle_diameter` sets `n`.
- `extend_extruder_variant`, all three steps.
- The extruder-option resize of `set_num_extruders`, including the skip of
  `default_filament_profile`.
- The process and printer presets with `single_extruder_multi_material` on are no-ops,
  exactly as upstream. A process preset has no `filament_diameter`, and a printer preset has
  none either.

## Deferred

- The filament branch of `Preset::normalize` (`Preset.cpp:527-533,544-568`):
  `set_num_filaments` and the filament option resize for system filament presets. It is
  reached only by the filament refresh, and the case of this slice does not observe it.
- `extend_default_config_length` (`PresetBundle.cpp:7649`): the per-variant padding before
  `Preset::normalize`.
- `handle_legacy_sla` (`Preset.cpp:570`): no FFF effect.

## Premises

1. The two-extruder J1 project of the `tool_templates` fixture (`Snapmaker J1 (0.4
   nozzle)`, `filament_map` `1,2`) holds neither `extruder_variant_list` nor
   `default_nozzle_volume_type`. The J1 machine chain (`Snapmaker J1 (0.4 nozzle)` →
   `fdm_idex` → `fdm_common`) sets neither, and has `single_extruder_multi_material = 0`.
2. Both modes slice it through the supplied preset catalog (`slice_with_profiles` with the
   project-derived selection), so the Ares refresh runs.
3. Without the machine normalization, the only whole-output difference between Native and
   Ares in both modes is three config lines:
   - `default_nozzle_volume_type`: Native has one entry per extruder, Ares has one;
   - `extruder_variant_list`: the same;
   - `print_extruder_id`: Native counts extruders, Ares repeats `1`.

   Every other J1 `tool_templates` fixture differs from Native by the same 33 bytes at the
   same column.

## Observable behavior

The `; default_nozzle_volume_type`, `; extruder_variant_list` and `; print_extruder_id` lines
of the config block equal Native's. Every other line is unchanged.

## Failure modes

- A printer preset with `single_extruder_multi_material` on, or without `nozzle_diameter`,
  must stay unchanged.
- `extruder_variant_list` longer than `n` truncates. An empty entry still yields one empty
  token, as `boost::split` does.
- Every `extruder_option_keys` key is a machine key, and `system_config` starts from all their
  defaults. So upstream's `option(key, false)`/`opt != nullptr` skip is never taken here.
- An empty vector pads from the definition default. A non-empty one pads with its first
  value.
- A project that already holds the keys keeps its own values, unless the refresh replaces
  unlisted keys (`refresh_unlisted`). In that case the normalized system values replace
  them, as upstream.

## Test plan

There is one new test in
`crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/`, on the shared
`configured_project` fixture, gated by `ARES_PARITY_REGION_FILAMENT_REFERENCE`:

- Runtime premises:
  - the project lacks `extruder_variant_list` and `default_nozzle_volume_type`;
  - Native's `nozzle_diameter` line lists two nozzles.
- The CLI `slice` output of the same project equals the API output.
- The whole Ares output equals Native under `golden::compare_ordered_bytes_generator_only`.

| Test | Without the normalization | With it |
|---|---|---|
| `project_without_variant_columns_matches_native_output` (new) | F | P |

The base already calls `normalize_machine`, so this slice has no fix commit. The test is
expected to pass in both modes, with no PASS turning into FAIL.
