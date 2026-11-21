import { readFileSync, writeFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
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

/** The printable_area bounding box size and printable_height, truncated (OrcaSlicer.cpp:1899-1909, 4371-4382). */
function bedSize(config: Record<string, unknown>): [number, number, number] {
  const points = (config.printable_area as string[]).map((point) => point.split("x").map(Number));
  const size = (axis: number) => Math.trunc(Math.max(...points.map((point) => point[axis])) - Math.min(...points.map((point) => point[axis])));
  return [size(0), size(1), Math.trunc(Number(config.printable_height))];
}

/**
 * The plater export of `selection`, a session that switched the opened BBL 3MF `opened` to a never-used printer,
 * saved to `out` with the CLI machine-switch steps the web slice still runs on the plater's full_fff_config
 * (OrcaSlicer.cpp at 40ecc66f7). Returns `selection` with that project config. Each change is computed from
 * `opened` and `selection`:
 * - the instances and wipe tower move onto the larger bed (shrink_to_new_bed 1, 1899-1909, 4371-4382,
 *   4414-4443, 4619-4685): by the difference of the bed centres;
 * - curr_bed_type is the printer's default_bed_type, which Sidebar::update_presets selects for a printer without a
 *   stored bed type (Plater.cpp:3880-3900, Preset.cpp:1140-1150), where the probe keeps the wizard's;
 * - nozzle_volume_type stays the session's, one entry per head from load_selections (PresetBundle.cpp:4196-4216);
 * - filament_plugin_config_overrides, a scalar filament option the CLI skips (3715-3734), comes only from the 3MF;
 * - one filament preset in every slot, of one colour, turns the prime tower off and records it in the process
 *   column of different_settings_to_system (2531-2535, 3880-3888, 4316-4343), sized filament count + 2 (3202-3205).
 */
export function cliSwitchedExport(selection: NativeSelection, opened: string, out: string): NativeSelection {
  const entries = unzipSync(readFileSync(opened));
  const model = strFromU8(entries["3D/3dmodel.model"]);
  expect(model, "a BBL 3MF (bbs_3mf.cpp:4170-4186)").toMatch(/<metadata name="Application">(BambuStudio|OrcaSlicer)-/);
  expect(strFromU8(entries["Metadata/model_settings.config"]).match(/<plate>/g), "one plate").toHaveLength(1);
  const project = JSON.parse(strFromU8(entries[PROJECT_CONFIG])) as Record<string, unknown>;
  const plater = JSON.parse(selection.full_fff_config) as Record<string, unknown>;
  const [oldWidth, oldDepth, oldHeight] = bedSize(project);
  const [width, depth, height] = bedSize(plater);
  expect(oldWidth <= width && oldDepth <= depth && oldHeight <= height && (oldWidth < width || oldDepth < depth), "shrink_to_new_bed 1").toBe(true);
  const [x, y] = [width / 2 - oldWidth / 2, depth / 2 - oldDepth / 2];
  entries["3D/3dmodel.model"] = strToU8(model.replace(/(<item [^>]*transform="(?:\S+ ){9})(\S+) (\S+)/g,
    (_, head: string, tx: string, ty: string) => `${head}${Number(tx) + x} ${Number(ty) + y}`));
  const moved = (values: string[], offset: number) => [String(Number(values[0]) + offset), ...values.slice(1)];
  const bedType = plater.default_bed_type as string;
  expect(bedType, "a default_bed_type (Preset.cpp:1142-1150)").not.toBe("");
  const { filament_plugin_config_overrides: _, ...config } = plater;
  Object.assign(config, {
    wipe_tower_x: moved(plater.wipe_tower_x as string[], x),
    wipe_tower_y: moved(plater.wipe_tower_y as string[], y),
    curr_bed_type: bedType,
  });
  if ("filament_plugin_config_overrides" in project) config.filament_plugin_config_overrides = project.filament_plugin_config_overrides;
  // 4291-4320 keep the tower for wrapping detection, a mixed filament or a smooth timelapse.
  expect(plater.enable_wrapping_detection !== "1" && !(plater.filament_is_mixed as string[]).includes("1") && plater.timelapse_type !== "1",
    "no tower kept by 4291-4320").toBe(true);
  const colours = new Set((plater.filament_colour as string[]).filter((colour) => colour !== ""));
  if (new Set(selection.filament_presets).size === 1 && colours.size === 1) {
    config.enable_prime_tower = "0";
    config.different_settings_to_system = ["enable_prime_tower", ...selection.filament_presets.map(() => ""), ""];
  }
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
