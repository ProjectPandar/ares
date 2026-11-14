import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { basename } from "node:path";
import { fileURLToPath } from "node:url";
import { strFromU8, unzipSync } from "fflate";
import { oraclePath } from "./oracleEnv";
import { nativeConfigDef } from "./settingsOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const orca = process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`;
export const PROJECT_CONFIG = "Metadata/project_settings.config";

export interface NativePreset {
  name: string;
  config: Record<string, { serialized: string }>;
}

/** One `PresetCollection` after the load: its selected preset and the edited copy the plater slices. */
export interface NativeCollection {
  selected: NativePreset;
  edited: NativePreset;
  dirty: boolean;
}

export interface NativeSelection {
  process: NativeCollection;
  printer: NativeCollection;
  filament: NativeCollection;
  filament_presets: string[];
  project_config: Record<string, { serialized: string }>;
}

/** The preset-select probe, or null where CI does not provision it. */
export const presetSelectProbe = () =>
  oraclePath("ARES_ORCA_PRESET_SELECT_BIN", "the preset-select probe", "tools/chain-probe/preset-select.patch built at 40ecc66f7");

/**
 * The project config as ConfigBase::load_from_json reads its JSON arrays (Config.cpp:1032-1052): ConfigOptionStrings
 * quoted and `;`-joined, ConfigOptionPointsGroups `#`-joined, other vectors `,`-joined.
 */
function serialized(config: Record<string, string | string[]>) {
  const defs = nativeConfigDef(repositoryRoot);
  const unknown: string[] = [];
  const options: Record<string, string> = {};
  for (const [key, value] of Object.entries(config)) {
    const def = defs.get(key);
    if (!def) unknown.push(key);
    else if (typeof value === "string") options[key] = value;
    else if (def.type === "strings") options[key] = value.map((item) => JSON.stringify(item)).join(";");
    else options[key] = value.join(def.type === "points_groups" ? "#" : ",");
  }
  return { options, unknown };
}

/**
 * PresetBundle::load_config_model (PresetBundle.hpp:590-591) as Plater.cpp:9622 calls it when the plater
 * opens the 3MF `project`, run by the preset-select probe in `directory`. The system presets are those of
 * `vendors` and of the two bundles the GUI always installs, Custom and OrcaFilamentLibrary
 * (PresetUpdater.cpp:1063-1067).
 */
export function nativeSelection(probe: string, directory: string, project: string, vendors: string[]): NativeSelection {
  rmSync(directory, { recursive: true, force: true });
  const system = `${directory}data/system/`;
  mkdirSync(system, { recursive: true });
  for (const vendor of ["Custom", "OrcaFilamentLibrary", ...vendors]) {
    symlinkSync(`${orca}resources/profiles/${vendor}.json`, `${system}${vendor}.json`);
    symlinkSync(`${orca}resources/profiles/${vendor}`, `${system}${vendor}`);
  }
  const entries = unzipSync(readFileSync(project));
  // bbs_3mf.cpp:1539-1551: the OrcaSlicer metadata tag is the file version, else the Application's.
  const model = strFromU8(entries["3D/3dmodel.model"]);
  const version = (/<metadata name="OrcaSlicer">([^<]+)/.exec(model) ?? /<metadata name="Application">[^<]*-([^<-]+)/.exec(model))![1];
  const { options, unknown } = serialized(JSON.parse(strFromU8(entries[PROJECT_CONFIG])));
  writeFileSync(`${directory}unknown-keys.json`, JSON.stringify(unknown));
  const request = { data_directory: `${directory}data`, projects: [{ filename: basename(project), version, options }], output: `${directory}native.json` };
  writeFileSync(`${directory}request.json`, JSON.stringify(request, null, 2));
  execFileSync(probe, { cwd: directory, env: { ...process.env, ARES_PRESET_SELECT_REQUEST: `${directory}request.json` }, stdio: ["ignore", "pipe", "pipe"] });
  return JSON.parse(readFileSync(`${directory}native.json`, "utf8")).projects[0].result;
}
