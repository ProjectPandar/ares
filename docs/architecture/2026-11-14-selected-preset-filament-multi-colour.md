# `filament_multi_colour` in a selected-preset slice

A browser slice with selected system presets writes `; filament_multi_colour = ` into the
G-code config block. The OrcaSlicer CLI `--load-settings` slice of the same project omits
that line. Citations refer to OrcaSlicer 2.5.0 (`40ecc66f7`), `src/libslic3r` unless noted.

## Upstream boundary

- `GCode::append_full_config` (`GCode.cpp:8272`) dumps every key of
  `print.full_print_config()`.
- `filament_multi_colour` is registered in `PrintConfig.cpp:3079` (`coStrings`, default
  `{""}`), but no static config class declares it (`PrintConfig.hpp`). The dump carries
  it only when the loaded dynamic config does.
- In the GUI, `PresetBundle::full_fff_config` applies `project_config`
  (`PresetBundle.cpp:5542`). `project_config` always holds the `s_project_options` keys
  (`PresetBundle.cpp:125-131`), so a GUI slice writes the line.
- The CLI builds its config from the 3MF `project_settings.config`
  (`src/OrcaSlicer.cpp:1758`) and the loaded preset files. It then completes only the
  `FullPrintConfig` keys (`src/OrcaSlicer.cpp:4253-4256`). A project that does not carry
  `filament_multi_colour` therefore slices without the line, and an STL never has it.

The selected-preset slice is checked against the CLI (`web/e2e/presetSlice.spec.ts`), so
the key belongs in the slice only when the loaded project carries it.

## Ares boundary

The Ares core already follows this. `slice_with_profiles` with the LH Stinger selection
and no override:

- writes no `filament_multi_colour` line for an STL, or for a 3MF without the key;
- writes the project's own value for a 3MF that carries it (`#123456` on both sides).

The same call with the override `filament_multi_colour = [""]` writes the empty line.

The web adds that override. `web/app/src/state/presets.ts` `updateCompatible`:

1. stores the result of `normalizeFilamentSlots` (the `FilamentSlotState` port of
   `PresetBundle::update_multi_material_filament_presets`) as `filamentProject`;
2. that state always has `filament_multi_colour`, because the core fills absent fields
   with their defaults;
3. `profileRequest` and `sliceWithPresets` (`state/presetSlice.ts`) send all of
   `filamentProject` as slice overrides.

The web cannot tell whether the project carried the key: its project arrays come from
the scene settings, which hold the resolved project options with defaults filled in.

## Included

- `updateCompatible` leaves `filament_multi_colour` out of `filamentProject`, so the web
  never sends it as a slice override. The core keeps the loaded project's own value.

## Deferred

- The comma-space form of `thumbnails` in the same comparison. It is tracked separately;
  the whole-output comparison still fails on it.
- Slot padding (`PresetBundle::set_num_filaments`, `PresetBundle.cpp:4328-4372`, which
  copies `filament_colour` into `filament_multi_colour`) is a GUI preset-bundle step; the
  CLI (`src/OrcaSlicer.cpp`) never calls it. A one-slot 3MF sliced on the four-slot
  Snapmaker U1 with four `--load-filaments` ends Native with SIGSEGV, with or without the
  key, and the core rejects the selection ("selection has 4 filaments but the project
  embeds 1"). With no Native output, this case is not tested.

## Verification

`web/e2e/presetSlice.spec.ts` checks the `; filament_multi_colour` config line of the
browser G-code against the Native G-code before each whole-output comparison: the
selected-preset 3MF slice, the sidebar-edited slice, and the bed-centered STL slice. It
runs in official and seeded Native modes.
