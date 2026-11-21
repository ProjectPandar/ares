import { readFileSync, writeFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { type NativeSelection, nativeSelection, PROJECT_CONFIG, presetSelectProbe } from "./presetSelectOracle";

/**
 * The G-code lines the plater's preset selection decides, by key; a CLI-route slice of the same project
 * differs from the plater's only in these:
 * - `printer_settings_id`, `print_settings_id`, `filament_settings_id`: full_fff_config names the selected
 *   presets (PresetBundle.cpp:5818-5820);
 * - `filament_plugin_config_overrides`: a scalar filament option, taken from the first filament preset
 *   (PresetBundle.cpp:5726-5735); the plater's presets hold it, a project config may not;
 * - `pellet_flow_coefficient`: a filament option the plater's project config always holds, loaded over
 *   FullPrintConfig::defaults() (Plater.cpp:9463); the GUI slices full_config(false) (Plater.cpp:11442), which
 *   Print::apply keeps as full_print_config (PrintApply.cpp:1677) and GCode::append_full_config prints
 *   (GCode.cpp:8272-8382), while a CLI-route slice of a project without it does not print it;
 * - `filament_multi_colour`: the plater's project config holds it after sync_loaded_project_filament_config
 *   (Plater.cpp:8905-8959, 9755), so the GUI prints it; a CLI-route slice prints only what the 3MF stores;
 * - `Printer`, `Profile`: start G-code that prints `[printer_preset]` and `[print_preset]`, the selected names
 *   (PrintApply.cpp:1660-1662).
 */
export const SELECTED_PRESET_KEYS = [
  "printer_settings_id",
  "print_settings_id",
  "filament_settings_id",
  "filament_plugin_config_overrides",
  "pellet_flow_coefficient",
  "filament_multi_colour",
  "Printer",
  "Profile",
] as const;

/** ConfigOptionStrings::serialize, escape_strings_cstyle (Config.cpp:99-147). */
export function cstyle(values: string[]) {
  return values.map((value) => {
    const quote = (values.length === 1 && value === "") || /[ \t;\\"\r\n]/.test(value);
    return quote ? `"${value.replace(/[\\"]/g, "\\$&").replace(/\r/g, "\\r").replace(/\n/g, "\\n")}"` : value;
  }).join(";");
}

type SelectedPresetKey = (typeof SELECTED_PRESET_KEYS)[number];

/**
 * The config line of `key` in the G-code `lines` set to `value`: replaced, or inserted at its sorted place in the
 * config block; removed when `value` is undefined.
 */
export function setConfigLine(lines: string[], key: string, value: string | undefined) {
  const start = lines.indexOf("; CONFIG_BLOCK_START");
  const end = lines.indexOf("; CONFIG_BLOCK_END");
  const block = lines.slice(start + 1, end);
  const at = block.findIndex((line) => line.startsWith(`; ${key} = `));
  if (value === undefined) {
    if (at >= 0) lines.splice(start + 1 + at, 1);
  } else if (at >= 0) {
    lines[start + 1 + at] = `; ${key} = ${value}`;
  } else {
    const after = block.findIndex((line) => /^; (\S+) = /.exec(line)![1] > key);
    lines.splice(start + 1 + (after < 0 ? block.length : after), 0, `; ${key} = ${value}`);
  }
}

/** The value of each key under `selection`; `undefined` where the selected presets do not hold the option. */
function selectedValues(selection: NativeSelection): Record<SelectedPresetKey, string | undefined> {
  // Native premise: the filament collection's selection is the first filament, whose edited copy find_preset returns.
  expect(selection.filament.selected.name, "Native selects the first filament").toBe(selection.filament_presets[0]);
  const plater = JSON.parse(selection.full_fff_config) as { pellet_flow_coefficient?: string[]; filament_multi_colour?: string[] };
  return {
    printer_settings_id: selection.printer.selected.name,
    print_settings_id: selection.process.selected.name,
    filament_settings_id: selection.filament_presets.map((name) => JSON.stringify(name)).join(";"),
    filament_plugin_config_overrides: selection.filament.edited.config.filament_plugin_config_overrides?.serialized,
    pellet_flow_coefficient: plater.pellet_flow_coefficient?.join(","),
    filament_multi_colour: plater.filament_multi_colour && cstyle(plater.filament_multi_colour),
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
 * where the reference has it. The `compatible_printers_condition` line is removed: the Native CLI keeps the key a
 * 3MF stores, while the GUI's project config, full_fff_config, erases it (PresetBundle.cpp:5790-5794); this models
 * the GUI's config in the reference. Every other byte is the reference's.
 *
 * The plater's own export is no reference here: the projects this serves open with no vendor installed besides
 * Custom, so the plater selects project-local presets "<name>(<file>.3mf)", and the Native CLI rejects their export
 * with CLI_PROCESS_NOT_COMPATIBLE, -17 (OrcaSlicer.cpp:2575-2620): the process lists the system printer name, not
 * the project-local one. Those presets hold the project's own config, so the CLI slice of the opened 3MF is the
 * closest reference.
 */
export function withSelectedPresets(reference: string, selection: NativeSelection): string {
  const values = selectedValues(selection);
  const lines = reference.split("\n");
  for (const key of ["Printer", "Profile"] as const) {
    for (const [index, line] of lines.entries()) if (line.startsWith(`; ${key} : `)) lines[index] = `; ${key} : ${values[key]}`;
  }
  for (const key of ["printer_settings_id", "print_settings_id", "filament_settings_id", "filament_plugin_config_overrides", "pellet_flow_coefficient", "filament_multi_colour"] as const) {
    setConfigLine(lines, key, values[key]);
  }
  const start = lines.indexOf("; CONFIG_BLOCK_START");
  const erased = lines.findIndex((line, index) => index > start && line.startsWith("; compatible_printers_condition = "));
  if (erased >= 0) lines.splice(erased, 1);
  return lines.join("\n");
}

/**
 * The opened 3MF `opened` saved to `out` with each project config key it lacks set from `selection`'s
 * full_fff_config, the plater's project config. Where the GUI finds no system base for its presets it keeps
 * FullPrintConfig::defaults() for those keys (Plater.cpp:9463, Preset.cpp:1090-1115), while the Native CLI fills
 * them from the system preset (OrcaSlicer.cpp:3163-3196). With no key missing and no different_settings_to_system,
 * the CLI keeps every project key (OrcaSlicer.cpp:2775-2791), so it slices the GUI's values under the 3MF's preset
 * names. Returns the added values.
 */
export function withGuiMissingKeys(opened: string, selection: NativeSelection, out: string) {
  const entries = unzipSync(readFileSync(opened));
  const project = JSON.parse(strFromU8(entries[PROJECT_CONFIG])) as Record<string, unknown>;
  const added = Object.fromEntries(Object.entries(JSON.parse(selection.full_fff_config) as Record<string, unknown>)
    .filter(([key]) => !(key in project)));
  const config = { ...project, ...added };
  expect("different_settings_to_system" in config, "no different_settings_to_system turns on refresh_unlisted").toBe(false);
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify(config));
  writeFileSync(out, zipSync(entries));
  return added as Record<string, string | string[]>;
}
