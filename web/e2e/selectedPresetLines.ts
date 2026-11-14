import { expect } from "@playwright/test";
import { type NativeSelection, nativeSelection, presetSelectProbe } from "./presetSelectOracle";

/**
 * The G-code lines the plater's preset selection decides, by key; a CLI-route slice of the same project
 * differs from the plater's only in these:
 * - `printer_settings_id`, `print_settings_id`, `filament_settings_id`: full_fff_config names the selected
 *   presets (PresetBundle.cpp:5818-5820);
 * - `filament_plugin_config_overrides`: a scalar filament option, taken from the first filament preset
 *   (PresetBundle.cpp:5726-5735); the plater's presets hold it, a project config may not;
 * - `Printer`, `Profile`: start G-code that prints `[printer_preset]` and `[print_preset]`, the selected names
 *   (PrintApply.cpp:1660-1662).
 */
export const SELECTED_PRESET_KEYS = [
  "printer_settings_id",
  "print_settings_id",
  "filament_settings_id",
  "filament_plugin_config_overrides",
  "Printer",
  "Profile",
] as const;

type SelectedPresetKey = (typeof SELECTED_PRESET_KEYS)[number];

/** The value of each key under `selection`; `undefined` where the selected presets do not hold the option. */
function selectedValues(selection: NativeSelection): Record<SelectedPresetKey, string | undefined> {
  // Native premise: the filament collection's selection is the first filament, whose edited copy find_preset returns.
  expect(selection.filament.selected.name, "Native selects the first filament").toBe(selection.filament_presets[0]);
  return {
    printer_settings_id: selection.printer.selected.name,
    print_settings_id: selection.process.selected.name,
    filament_settings_id: selection.filament_presets.map((name) => JSON.stringify(name)).join(";"),
    filament_plugin_config_overrides: selection.filament.edited.config.filament_plugin_config_overrides?.serialized,
    Printer: selection.printer.selected.name,
    Profile: selection.process.selected.name,
  };
}

/**
 * The probe's selection, run in `directory`, when the app opens the 3MF `opened` with the first-run vendors (none
 * besides Custom); null where CI does not provision the probe.
 */
export function openedSelection(opened: string, directory: string): NativeSelection | null {
  const probe = presetSelectProbe();
  return probe === null ? null : nativeSelection(probe, directory, opened, []);
}

/**
 * `reference`, a CLI-route slice of the 3MF the browser opened, with the lines of SELECTED_PRESET_KEYS set from
 * `selection`, the preset-select probe's result for that 3MF: a config line the selected presets hold is replaced,
 * or inserted at its sorted place in the config block; one they lack is removed. A start G-code line is replaced
 * where the reference has it. Every other byte is the reference's.
 */
export function withSelectedPresets(reference: string, selection: NativeSelection): string {
  const values = selectedValues(selection);
  const lines = reference.split("\n");
  for (const key of ["Printer", "Profile"] as const) {
    for (const [index, line] of lines.entries()) if (line.startsWith(`; ${key} : `)) lines[index] = `; ${key} : ${values[key]}`;
  }
  for (const key of ["printer_settings_id", "print_settings_id", "filament_settings_id", "filament_plugin_config_overrides"] as const) {
    const start = lines.indexOf("; CONFIG_BLOCK_START");
    const end = lines.indexOf("; CONFIG_BLOCK_END");
    const block = lines.slice(start + 1, end);
    const value = values[key];
    const at = block.findIndex((line) => line.startsWith(`; ${key} = `));
    if (value === undefined) {
      if (at >= 0) lines.splice(start + 1 + at, 1);
      continue;
    }
    if (at >= 0) {
      lines[start + 1 + at] = `; ${key} = ${value}`;
      continue;
    }
    const after = block.findIndex((line) => /^; (\S+) = /.exec(line)![1] > key);
    lines.splice(start + 1 + (after < 0 ? block.length : after), 0, `; ${key} = ${value}`);
  }
  return lines.join("\n");
}
