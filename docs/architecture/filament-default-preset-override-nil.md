# Default filament preset sets nullable override keys to nil

Rewrite slice of the default filament preset nil pass in `PresetBundle::PresetBundle` (`PresetBundle.cpp:508-518`) and `is_filament_extruder_override_key` (`PrintConfig.cpp:124-139`), at OrcaSlicer v2.5.0-beta `345efec64`. When a filament system preset is resolved, every key that its inherits chain does not set comes from the default filament preset. For nullable filament override keys, that value is nil, not the `PrintConfigDef` default.

## Problem
`tests/parity/overall-chamber-placeholder/{control,target}.3mf` use an Anycubic Kobra 3 filament system preset whose inherits chain does not set `filament_retract_length_nc`.
- **Native:** the refreshed filament value is nil. `GCode::append_full_config` skips nil options (`GCode.cpp:8342-8343`), so the config footer has no `; filament_retract_length_nc` line.
- **Ares:** the project refresh seeds the system filament config with the registry default `10`. The footer gains `; filament_retract_length_nc = 10`; nothing else differs.

## Upstream boundary
- **`PresetBundle::PresetBundle`** (`PresetBundle.cpp:508-518`) visits every key of `filaments.default_preset().config`. If `is_filament_extruder_override_key(opt_key)` holds and the option is `nullable()`, it deserializes `"nil"` into it.
- **`is_filament_extruder_override_key`** (`PrintConfig.cpp:130-139`) is true for the 19 `filament_extruder_override_keys` (`PrintConfig.cpp:97-122`) and for `filament_retract_length_nc`, `filament_ironing_flow`, `filament_ironing_spacing`, `filament_ironing_inset` and `filament_ironing_speed`.
- **`filament_retract_length_nc`** (`PrintConfig.cpp:8928-8937`) is a nullable `coFloats` whose `PrintConfigDef` default is `10`. The other 23 keys already default to nil.
- **Vendor preset load** (`PresetBundle.cpp:7627-7629`) starts a root preset from `default_preset_for(entry.config_src).config`, and a child preset starts from its parent's config (`:7598-7606`). Each preset's own keys are applied on top (`:7640`), so a key that no preset in the chain sets keeps the default preset's nil.
- **The CLI filament refresh** (`OrcaSlicer.cpp:2800-2807`) takes that preset config through `resolve_system_preset` (`PresetBundle.cpp:724-757`) and applies it over the loaded project, so the nil reaches the print config.
- **`GCode::append_full_config`** (`GCode.cpp:8342-8343`) writes only options for which `!is_nil()`.

## Ares destination
`profiles/project_refresh/keys.rs` `default_value`, which `preset.rs` `system_config` uses to seed every member key before the inherits chain is applied:
- **Port the nil pass:** return `nil` when `is_filament_extruder_override_key(key)` holds and the registry kind is nullable, in addition to the existing `default_is_nil` case.
- **Port `is_filament_extruder_override_key`** as a private literal key match in the same file.
- The footer writer already skips nil raw vectors, so `full_config.rs` does not change.

## Included and deferred
- **Included:** the `PresetBundle.cpp:508-518` nil pass for the project refresh that runs whenever a preset catalog is supplied: the CLI `--project-preset-catalog` and the byte API with a catalog.
- **Deferred:**
  - **Slicing without a catalog.** With only the embedded 3MF config there are no system presets to refresh from, so the registry default stays.
  - **Variant arity of the key.** Some Native footers write `10,10,10,10` where Ares writes `10` (for example `auto-brim-adhesion-pla`). That belongs to the filament variant expansion and is separate.

## Oracle and RED
The existing whole-output tests `gcode_regression_cases::overall_chamber_placeholder::chamber_even_control_matches_native` and `chamber_odd_target_matches_native` compare with Native through `--project-preset-catalog`. Both fail on main with the extra footer line as the only difference, and both pass with this port. The Kobra 3 `brim_bed_region` and `brim_bed_region_controls` tests and `empty_selected_plate_control_populated_plate_two_preserves_native_stream` slice with a catalog too, fail on the same single line, and pass with this port. No new test is needed.

Expected output changes: across the 538 official catalog-path cases, 47 Ares outputs change, and every change is the removal of the `; filament_retract_length_nc` line. Native omits that line in all 47 outputs. 2 outputs become byte-identical to Native (the two tests above). The other 45 still differ for other reasons; examples are `retract-internal-regions-routed-*`, the `thin-gap-j1-*` cases and the Voron cases. No output gains a difference.
