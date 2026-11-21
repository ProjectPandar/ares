# Manifest-loaded system preset effective config: variant padding and thumbnail spelling

## Problem

A manifest-loaded system preset merges the collection default, inherited system parent, includes,
and the leaf preset before its effective config is exposed. The current Ares result differs from
OrcaSlicer in that path:

- `machine_max_junction_deviation` is one value (`0.01`) instead of two (`0.01,0.01`).
- `thumbnails` is `48x48/PNG, 300x300/PNG` instead of `48x48/PNG,300x300/PNG`.

The differences are visible in `loaded_system_inherits_group_preserves_cli_source_status` and
propagate to the load-settings, direct-STL, and project-catalog CLI entry pairs.

## Upstream boundary and timing

- `PrintConfig.cpp:5444-5452` registers `machine_max_junction_deviation` as `coFloats` with
  the single-value default `ConfigOptionFloats{0.01f}`.
- `PrintConfig.cpp:10273-10290` places that key in
  `printer_options_with_variant_2`. These are two values per machine variant.
- `PresetBundle::resolve_vendor_preset` (`PresetBundle.cpp:7750-7850`) first starts with the
  collection default or inherited parent, then applies includes and the leaf config. Only after
  that complete merge does it call `extend_default_config_length(out.config, true, *default_config)`.
  The helper in `Preset.cpp:313-370` derives the machine variant count from
  `printer_extruder_variant` (or `nozzle_diameter`) and resizes every
  `printer_options_with_variant_2` key to twice that count, using the option's first/default
  value to fill a short vector. This is the required point for the effective machine config.
- `PrintConfigDef::handle_legacy_composite` (`PrintConfig.cpp:10046-10074`) parses an authored
  `thumbnails` value after a config dictionary is fully loaded and rewrites a non-empty parsed
  list with `", "`. `ConfigBase::load` and the JSON/G-code loading paths call it only after their
  input dictionary has been read (`Config.cpp:1153`, `1245`, and `1516`). In this fixture the
  Native effective value comes from the untouched default, whose spelling is
  `48x48/PNG,300x300/PNG`; it is not an authored thumbnail value that should be rewritten while
  resolving the preset layers.

## Current Ares diagnosis

`crates/ares-core/src/profiles/catalog/compose.rs` resolves the profile layers and invokes
`preset_normalization::extend_default_config_length` while each layer is being composed. The
variant classifier in `crates/ares-core/src/options/extruder_variant.rs` already recognizes the
same four upstream variant classes, but the completed manifest-loaded effective config does not
reliably receive the upstream machine-vector completion after the final parent/include/leaf
merge. The final completion must apply the upstream machine count and the `* 2` size rule to
all present `printer_options_with_variant_2` vectors, including a one-value
`machine_max_junction_deviation`.

The thumbnail path in `crates/ares-core/src/profiles/fragment/payload.rs` normalizes a machine
payload before it is overlaid with collection defaults and inherited layers. That is earlier than
the upstream whole-dictionary composite conversion and can canonicalize a default-present value
into the space-containing spelling. Thumbnail normalization must distinguish an authored
`thumbnails` field from a value supplied by defaults/inheritance, preserving the untouched
Native default spelling for this effective-config path while retaining upstream parsing for an
authored value.

The other upstream per-variant sets (`print_options_with_variant`,
`filament_options_with_variant`, and `printer_options_with_variant_1`) use their existing
process, filament, and machine lengths. The paired machine set is the only set implicated by
this fixture; all its keys share the same `machine_variant_count * 2` rule and must be checked
for regressions rather than adding a one-key special case.

## Included

- Complete the loaded system preset's effective machine config after all resolved layers, with
  the upstream per-variant lengths and first/default-value padding.
- Preserve the Native default spelling of `thumbnails` when no layer authored that field, while
  keeping parsing and canonical formatting for an authored thumbnail list.
- Verify the public API and CLI produce the same effective config and complete output in both
  supported comparison modes for the focused loaded-system case.

## Deferred

- No changes to unrelated option ownership, thumbnail validation, or non-manifest project
  configuration formatting.
- No new compatibility fallback or independent Ares preset-resolution behavior.

## Verification

The public-API E2E parity test
`loaded_system_inherits_group_preserves_cli_source_status` must pass in both modes. The
load-settings, direct-STL, and project-catalog CLI pairs must each have zero newly introduced
Native-to-Ares effective-config failures, with the focused vector and thumbnail values matching
exactly.
