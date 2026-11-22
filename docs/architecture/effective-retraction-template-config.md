# Effective retraction configuration in G-code templates

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r`:

- `PrintApply.cpp:295–317` selects the filament-prefixed options for the keys in `PrintConfig.cpp:9030–9052::PrintConfigDef::init_extruder_option_keys`. `PrintConfig.cpp:12356–12387::compute_filament_override_value` applies the nullable vectors to cloned machine values (`Config.hpp:767–814`). `PrintApply.cpp:1652–1692` makes the merged values available to both `m_config` and `m_placeholder_parser` while retaining the full configuration independently. `Print.cpp:4843–4881` repeats the merge after filament mapping.
- `GCode.cpp:3582` copies the print parser for G-code export. The same parser is used for machine start/end and tool change (`GCode.cpp:4381,4878,11344–11355`). Indexed floats, percentages, Booleans and enums retain their native parser types (`PlaceholderParser.cpp:1350–1361`); a speed in this context has not been rounded for emission by the motion writer.
- `GCodeProcessor.cpp:5206–5223` reads the feed word of the expanded movement. `GCodeProcessor.cpp:1201–1204,1286–1287,1454–1472` computes remaining minutes and progress placement from the whole movement stream. Changing the speed of a machine-end retract can therefore affect earlier progress commands. The processor is downstream and is not changed.

## Rust destination and key coverage

`project_slice/gcode_emit/placeholders.rs::base_config` seeds the shared parser from the serialized configuration. Keep it as the compatibility shell for the upstream parser snapshot, but admit the effective retraction options from `options/project_config_views` before expanding templates. Read the typed runtime G-code and print views directly; do not reconstruct the override merge from template strings or use motion feedrates. The serialized config retains the original machine and filament values.

| Native keys | Parser representation |
| --- | --- |
| `deretraction_speed`, `retract_length_toolchange`, `retract_lift_above`, `retract_lift_below`, `retract_restart_extra`, `retract_restart_extra_toolchange`, `retraction_length`, `retraction_minimum_travel`, `retraction_speed`, `wipe_distance`, `z_hop` | Float lists from the runtime config; `retract_length` remains the effective length alias. |
| `retract_after_wipe`, `retract_before_wipe` | Float lists in percentage units, not fractions. |
| `retract_when_changing_layer`, `wipe`, `long_retractions_when_cut` | Boolean lists. |
| `z_hop_types`, `retract_lift_enforce` | Integer lists using the enum ordinals defined at `PrintConfig.hpp:583–597`. |
| `retraction_distances_when_cut` | Float list supplied through the existing cut-retraction path; retain its alias and per-filament selection. |
| `travel_slope` | Machine-only: `PrintConfig.cpp:6233` defines the machine vector, but `97–123,8783–8810` registers no `filament_travel_slope`. `PrintApply.cpp:306–317` therefore takes the ordinary machine-key path instead of making a filament override. Leave its existing machine vector in the parser. |

Use the existing cut-retraction runtime branches: outside `EnableFilament`, the Boolean resolves against the machine values and the distance remains on the existing machine path. `PrintConfig.cpp:12361–12375` handles these keys separately; the distance branch fills the Boolean temporary rather than the float temporary in this revision, and this slice does not copy that mistake into new code. Preserve the established cut aliases. No registered override key needs a new option type. Changes to the shared config merge, dynamic variant remapping, firmware commands, wipe and tower rules, and timing logic are outside this boundary.

## Verification

Through public project slicing, use deterministic geometry and templates that expose decimal speed and deretraction overrides and a nullable slot with a different machine value. Check the expanded G-code against the pinned Native output and require unchanged config serialization. Exercise machine start and end on the affected stock projects, compare intact output with Native in both reference modes, and pair the existing placeholder, retraction and progress comparisons before and after the port.
