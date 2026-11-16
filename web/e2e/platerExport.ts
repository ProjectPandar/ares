import { readFileSync, writeFileSync } from "node:fs";
import { strToU8, unzipSync, zipSync } from "fflate";
import { type NativeSelection, PROJECT_CONFIG } from "./presetSelectOracle";

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
