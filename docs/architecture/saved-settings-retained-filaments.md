# Retained filament presets in standalone settings export

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/OrcaSlicer.cpp:1880–1890,2793–2817`, resolves the retained filament name for each unreplaced slot of a saved BBL-family project. `src/libslic3r/PresetBundle.cpp:724–756` returns the resolved system preset. A missing preset leaves the imported slot intact. `PresetBundle.cpp:484–487,2632–2650` and `Preset.cpp:1524,1766–1776,1818–1823` establish filament defaults, membership, and inheritance before resolution. The ordinary float-vector `pellet_flow_coefficient` belongs to this collection (`PrintConfig.cpp:3275–3279`). It can come from a resolved default or from an explicit preset value.

`OrcaSlicer.cpp:3509–3510` enters the existing filament merge for resolved retained slots even without explicit preset arguments. The merge places `compatible_prints_condition` into the positional `compatible_process_expression_group` (`3723–3728`) and ordinary vectors into their logical filament slots (`3736–3753,3770–3772`). Both `--export-settings` (`6326–6334`) and `Print::apply` (`6924,6986`) consume the effective configuration. The G-code footer writes that configuration without synthesizing these keys (`PrintApply.cpp:1677`, `GCode.cpp:8272–8274,8370–8381`).

## Rust destination

The saved-project action in `ares-cli/src/settings_export.rs` must inject the same installation catalog already used for saved-project slicing. `ares-core/src/project/settings_export.rs` must share the retained project catalog application from `project_slice/source.rs` (`profiles/catalog.rs`, `profiles/project_refresh/preset.rs` and `project_refresh/filament/slot.rs`) before standalone serialization. The current standalone saved-config loader is a temporary approximation of the common CLI preset-application stage. Keep core resource-free, preserve the no-catalog library behavior, and apply the existing BBL-family guard. No footer/default synthesis, preset matching changes, variant layout changes, or changes to GUI, non-BBL, or explicit replacement paths.

## Public behavior

Use the installation-relative resource layout to verify that a plain saved stock slice already matches the Native G-code, and compare the complete standalone settings output for a one-slot stock project and a synthetic two-slot BBL-family project. For the two-slot case, Native without resolution (only the two saved names changed to an unknown name) matches all 32 fields on which current standalone export differs: seven absent entries and 25 one-slot vectors. These differences arise from the retained-preset stage, not a separate vector padding rule. Compare the unresolved two-slot project as a control. Differences observed independently in non-BBL exports are outside this slice.
