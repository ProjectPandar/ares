import { readFileSync, writeFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { strToU8, unzipSync, zipSync } from "fflate";
import { type NativeSelection, PROJECT_CONFIG } from "./presetSelectOracle";

/** The groups full_fff_config stores only when some entry is non-empty (PresetBundle.cpp:5826-5838). */
const GROUPS = ["compatible_machine_expression_group", "compatible_process_expression_group", "inherits_group", "different_settings_to_system"];

/**
 * The 3MF `out` the plater saves after the session `selection`, which the preset-select probe replayed on the
 * opened 3MF `opened`: the model is the opened one, the project config is full_config_secure() (Plater.cpp:19779),
 * the probe's full_fff_config, with the sidebar edits `edit` on top.
 */
export function platerExport(selection: NativeSelection, opened: string, out: string, edit: Record<string, unknown> = {}) {
  const entries = unzipSync(readFileSync(opened));
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify({ ...JSON.parse(selection.full_fff_config), ...edit }));
  writeFileSync(out, zipSync(entries));
  return out;
}

/**
 * `gcode`, the Native CLI slice of the plater export after `selection`, without the group lines the plater's
 * config lacks: the plater composes each group from its presets and stores it only when some entry is non-empty
 * (PresetBundle.cpp:5776-5838), while the Native CLI prints it for a config without it (OrcaSlicer.cpp:3202-3205
 * creates inherits_group and different_settings_to_system). Every entry of a removed line must be empty.
 */
export function withoutCliGroups(gcode: string, selection: NativeSelection) {
  const plater = JSON.parse(selection.full_fff_config) as Record<string, unknown>;
  return gcode.split("\n").filter((line) => {
    const key = GROUPS.find((group) => line.startsWith(`; ${group} = `));
    if (key === undefined || key in plater) return true;
    const entries = line.slice(`; ${key} = `.length).split(";");
    expect(entries.every((entry) => entry === "" || entry === '""'), `every Native ${key} entry is empty`).toBe(true);
    return false;
  }).join("\n");
}
