# Collection defaults for catalog user presets

## Upstream rewrite boundary

Port the default source of `PresetCollection::resolve_user_preset` from
OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), not an
export-format correction or a new configuration pipeline.

`Preset.cpp:1856–1868` constructs the user preset with the collection's `m_type`.
At `:1925–1944`, an inherited preset copies the completed parent configuration;
a root preset copies the collection default before applying authored values.
There is no process-only restriction on these branches.

All three FFF collections therefore require this rule:

| Collection | Construction in PresetBundle.cpp | Default option ownership in Preset.cpp |
|---|---|---|
| Print/process | 485, TYPE_PRINT | print_options at1736; list1183–1522 |
| Filament | 486, TYPE_FILAMENT | filament_options at1737; list1524–1602 |
| Printer/machine | 489, TYPE_PRINTER | printer_options at1746–1754 combines printer, machine-limit and nozzle keys |

Each constructor calls `PresetCollection::add_default_preset`
(`Preset.cpp:1773,1818–1824`). `ConfigBase::apply_only` (`Config.cpp:484–527`)
creates each explicitly owned destination option before reading the static
source. `DynamicConfig::optptr` (`:1725–1745`) obtains the registered default,
even when that option is not a static `FullPrintConfig` member. Filament
constructor handling (`PresetBundle.cpp:508–518`) makes nullable extruder
overrides nil; this remains part of filament defaults, not an export rule.

Vendor root/default layering at `PresetBundle.cpp:7827–7839` and user parent
copying at `Preset.cpp:1925–1933` carry these members through inheritance.
The Rust resolved-layer representation remains a temporary compatibility shell
around those completed parent configurations.

## Rust destination and decisions

`profiles/catalog/compose.rs::Catalog::merge` currently seeds
`project_refresh::default_preset_fields(kind)` only for a selected instantiated
vendor record. Selecting an imported user child bypasses this operation even
when its resolved chain contains vendor parents.

Seed the existing kind-owned collection defaults for catalog user records as
well, before applying resolved ancestor and authored child layers. Use the
existing process, filament and machine key lists and default-value producer;
do not introduce a second table or a special case for `ironing_expansion`.
Preserve ancestor-then-child precedence and authored nil/nondefault values.

Non-negotiable boundaries:

- `direct_imports` remains excluded. A sparse typed file is not a resolved
  catalog preset: `OrcaSlicer.cpp:2121–2123` returns before preset resolution
  when it has no inheritance. Do not treat that Native root-file shortcut as
  the same lifecycle as collection-root loading.
- Preserve `full_config_default=false` for dynamic-only registered options.
  `ironing_expansion` is process-owned (`Preset.cpp:1521`) with registered float
  default0 (`PrintConfig.cpp:5248–5256`), not a static FullPrintConfig member.
  Its applied raw membership must come from preset construction, not footer0.
- Reuse existing filament nil-default construction. Do not broaden the existing
  vendor-only nil replacement or vector-normalization policies as part of this
  default-seeding change. Existing user-process vector completion remains.
- Keep the exporter, comparator, physical-printer variant mapping, preset
  identities, direct sparse STL semantics and ordinary filament assignment
  unchanged. New differences outside default construction require a separate
  source-cited slice.

## Public-interface verification

Use the existing `slice_with_profiles`/Native full-output seam. The existing
`inherited_user_process_scalar_array_whole_file` must retain its Native-only
singleton reference control and strict ordered comparisons. At the starting
revision, both array cases in both modes first differ at byte125623,
line5086 column11: Native emits `ironing_expansion = 0`; Ares lacks that member.

Add an inherited catalog-user whole-output case for each of process, machine
and filament against its matched Native user-preset loading path. Preserve
complete source catalogs, selections, Native receipts and both full streams.
Commit specification, then tests; run them before production changes.

Run a fixed profile/catalog/STL/inheritance regression subset at unchanged
production and final production in both modes, using identical test sources,
inputs, environment bindings and filter. Include direct sparse membership and
nil controls so unrelated sparse paths cannot acquire dynamic-only defaults.
Publish the exact filter, expanded test inventory, per-test transitions and
failure diagnostics. Existing failures are not passes; no formerly passing
case may fail after the change. The original whole-file witness must either
pass or move its first difference beyond the current scalar omission.

Before submission, run workspace formatting, all-target clippy and wasm32
checks. Keep source diffs and final committed content bound to the observations.
