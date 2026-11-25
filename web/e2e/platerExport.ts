import { readFileSync, writeFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { type NativeSelection, PROJECT_CONFIG } from "./presetSelectOracle";
import { setConfigLine } from "./selectedPresetLines";

/** The groups full_fff_config stores only when some entry is non-empty (PresetBundle.cpp:5826-5838). */
const GROUPS = ["compatible_machine_expression_group", "compatible_process_expression_group", "inherits_group", "different_settings_to_system"];

/**
 * The 3MF `out` the plater saves after the session `selection`, which the preset-select probe replayed on the
 * opened 3MF `opened`: the model is the opened one, the project config is full_config_secure() (Plater.cpp:20493),
 * the probe's full_fff_config, with the sidebar edits `edit` on top.
 */
export function platerExport(selection: NativeSelection, opened: string, out: string, edit: Record<string, unknown> = {}) {
  const entries = unzipSync(readFileSync(opened));
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify({ ...JSON.parse(selection.full_fff_config), ...edit }));
  writeFileSync(out, zipSync(entries));
  return out;
}

/** The printable_area bounding box size and printable_height, truncated (OrcaSlicer.cpp:1899-1909, 4371-4382). */
function bedSize(config: Record<string, unknown>): [number, number, number] {
  const points = (config.printable_area as string[]).map((point) => point.split("x").map(Number));
  const size = (axis: number) => Math.trunc(Math.max(...points.map((point) => point[axis])) - Math.min(...points.map((point) => point[axis])));
  return [size(0), size(1), Math.trunc(Number(config.printable_height))];
}

/**
 * The plater export of `selection`, a session that switched the opened BBL 3MF `opened` to a never-used printer.
 * The 3MF keeps its instance transforms: PartPlateList::reset_size passes adjust_position=false to
 * PartPlate::set_pos_and_size (PartPlate.cpp:6423-6440, 7083-7102, 4180-4235).
 * Returns `selection` with the exported project config:
 * - curr_bed_type is the printer's default_bed_type, which Sidebar::update_presets selects for a printer without a
 *   stored bed type (Plater.cpp:3880-3900, Preset.cpp:1140-1150), where the probe keeps the wizard's;
 * - nozzle_volume_type stays the session's, one entry per head from load_selections (PresetBundle.cpp:4196-4216).
 * The one-colour prime-tower rule (2531-2535, 4316-4343) is CLI::run's alone: the plater applies full_fff_config to
 * the print as it is (Plater.cpp:11429-11442), so the export keeps the session's enable_prime_tower.
 */
export function switchedPlaterExport(selection: NativeSelection, opened: string, out: string): NativeSelection {
  const entries = unzipSync(readFileSync(opened));
  const model = strFromU8(entries["3D/3dmodel.model"]);
  expect(model, "a BBL 3MF (bbs_3mf.cpp:4170-4186)").toMatch(/<metadata name="Application">(BambuStudio|OrcaSlicer)-/);
  expect(strFromU8(entries["Metadata/model_settings.config"]).match(/<plate>/g), "one plate").toHaveLength(1);
  const project = JSON.parse(strFromU8(entries[PROJECT_CONFIG])) as Record<string, unknown>;
  const plater = JSON.parse(selection.full_fff_config) as Record<string, unknown>;
  const [oldWidth, oldDepth, oldHeight] = bedSize(project);
  const [width, depth, height] = bedSize(plater);
  expect(oldWidth <= width && oldDepth <= depth && oldHeight <= height && (oldWidth < width || oldDepth < depth), "shrink_to_new_bed 1").toBe(true);
  const bedType = plater.default_bed_type as string;
  expect(bedType, "a default_bed_type (Preset.cpp:1142-1150)").not.toBe("");
  const config = { ...plater, curr_bed_type: bedType };
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify(config));
  writeFileSync(out, zipSync(entries));
  return { ...selection, full_fff_config: JSON.stringify(config) };
}

/**
 * The opened 3MF `opened` saved to `out` with the session `selection`'s curr_bed_type, a project option
 * (PresetBundle.cpp:134) that full_fff_config takes from the project config (PresetBundle.cpp:5542). Picking the
 * presets runs load_selections, which sets it to the selected printer's saved or default bed type
 * (PresetBundle.cpp:4046-4055, Preset.cpp:1140-1171) over the one the 3MF stores, which a CLI-route slice keeps.
 */
export function withPlaterBedType(selection: NativeSelection, opened: string, out: string) {
  const entries = unzipSync(readFileSync(opened));
  const { curr_bed_type } = JSON.parse(selection.full_fff_config) as { curr_bed_type: string };
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[PROJECT_CONFIG])), curr_bed_type }));
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

/**
 * `gcode`, a Native --load-settings slice, with the config lines full_fff_config decides for the plater session
 * `session`: the scalar `filament_plugin_config_overrides` from the first filament (PresetBundle.cpp:5726-5735), no
 * `compatible_printers_condition` (PresetBundle.cpp:5790-5794), and only the groups `session` stores.
 */
export function withPlaterConfigLines(gcode: string, session: NativeSelection) {
  const lines = gcode.split("\n");
  setConfigLine(lines, "filament_plugin_config_overrides", session.filament.edited.config.filament_plugin_config_overrides?.serialized);
  setConfigLine(lines, "compatible_printers_condition", undefined);
  return withoutCliGroups(lines.join("\n"), session);
}

/**
 * `session` keeping `different_settings_to_system`: an edit dirties the process column as the plater's edited print
 * does (PresetBundle.cpp:5586-5595), and a CLI `--load-filaments` reference keeps the column the one-colour
 * prime-tower rule records for withPlaterTower.
 */
export function withDifferences(session: NativeSelection): NativeSelection {
  return { ...session, full_fff_config: JSON.stringify({ different_settings_to_system: [], ...JSON.parse(session.full_fff_config) }) };
}

const TOWER_LINES = ["; enable_prime_tower = ", "; different_settings_to_system = "];

/** The entries of a config line value, unquoted (Config.cpp:99-139 escape_strings_cstyle). */
function configEntries(value: string) {
  return [...value.matchAll(/"((?:[^"\\]|\\.)*)"|([^;]*)/g)].filter((match) => match.index === 0 || value[match.index - 1] === ";")
    .map((match) => match[1] ?? match[2]);
}

/**
 * `expected`, a Native CLI `--load-filaments` reference for the one-filament plater `session`, with the plater's
 * prime tower. CLI::run alone turns the tower off for one mapped preset (OrcaSlicer.cpp:2531-2535, 4316-4343); the
 * plater applies full_fff_config as it is (Plater.cpp:11429-11442). With one filament the setting prints no tower
 * (Print.cpp:5205), so only two config lines differ, and `browser` must equal `expected` without them:
 * - enable_prime_tower stays 0: Print::apply runs normalize_fdm_2 on both routes, which clears it for one used
 *   filament, no mixed filament, no smooth timelapse and no wrapping detection (PrintConfig.cpp:10478-10505);
 * - different_settings_to_system loses the enable_prime_tower the rule appends to the process column (4330-4340),
 *   and the line goes when every entry is empty, as the plater stores no empty group (PresetBundle.cpp:5826-5838).
 */
export function withPlaterTower(expected: string, browser: string, session: NativeSelection) {
  expect(session.filament_presets, "one filament prints no tower (Print.cpp:5205)").toHaveLength(1);
  const others = (gcode: string) => gcode.split("\n").filter((line) => !TOWER_LINES.some((key) => line.startsWith(key))).join("\n");
  expect(others(browser) === others(expected), "the browser differs from the CLI reference only in the two tower lines").toBe(true);
  const plater = JSON.parse(session.full_fff_config) as Record<string, unknown>;
  expect((plater.filament_is_mixed as string[] | undefined) ?? [], "normalize_fdm_2: no mixed filament").not.toContain("1");
  expect(plater.timelapse_type, "normalize_fdm_2: no smooth timelapse").not.toBe("1");
  expect(plater.enable_wrapping_detection, "normalize_fdm_2: no wrapping detection").not.toBe("1");
  const lines = expected.split("\n");
  expect(lines.find((line) => line.startsWith(TOWER_LINES[0])), "normalize_fdm_2 prints the tower off").toBe(`${TOWER_LINES[0]}0`);
  const dirty = lines.findIndex((line) => line.startsWith(TOWER_LINES[1]));
  const columns = configEntries(lines[dirty].slice(TOWER_LINES[1].length));
  const keys = columns[0].split(";");
  expect(keys.at(-1), "the CLI rule appends enable_prime_tower last").toBe("enable_prime_tower");
  columns[0] = keys.slice(0, -1).join(";");
  if (columns.every((column) => column === "")) lines.splice(dirty, 1);
  else lines[dirty] = `${TOWER_LINES[1]}${columns.map((column) => (column.includes(";") ? `"${column}"` : column)).join(";")}`;
  return lines.join("\n");
}
