// Same source project/user-template route as slice.spec.ts, without changing
// either emitter. Native G-code is always the unmodified official slicer.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";

const root = fileURLToPath(new URL("../../", import.meta.url));
const out = process.argv[2];
if (!out) throw new Error("usage: node colorChangeOracle.mjs <artifact-directory>");
const template = "{if layer_num == 2}SYNC T2.75\n;COLOR_CHANGE,T0,#FF0000\nM600\n{endif}{if layer_num == 4}SYNC T1.25\n;PAUSE_PRINT\n{endif}{if layer_num == 6};COLOR_CHANGE,T0,#0000FF\nM600\n{endif}{if layer_num == 8};CUSTOM_GCODE\n{endif}";
let zero = readFileSync(`${root}tests/parity/color-change/zero-before.gcode`, "utf8");
for (const [bbl, compatible] of [[" CHANGE_LAYER", "LAYER_CHANGE"], [" LAYER_HEIGHT: ", "HEIGHT:"], [" LINE_WIDTH: ", "WIDTH:"], [" FEATURE: ", "TYPE:"], [" COLOR_CHANGE", "COLOR_CHANGE"]]) {
  zero = zero.replaceAll(`;${bbl}`, `;${compatible}`);
}
for (const [name, overrides] of [["normal", { layer_change_gcode: template }], ["stop-times", { layer_change_gcode: template.replace(/SYNC T[\d.]+\n/g, "") }], ["zero-before", { machine_start_gcode: zero }]]) {
  const dir = `${out}/${name}`;
  mkdirSync(dir, { recursive: true });
  const entries = unzipSync(readFileSync(`${root}tests/parity/orca_cli_ender3.3mf`));
  const key = "Metadata/project_settings.config";
  entries[key] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[key])), ...overrides }));
  writeFileSync(`${dir}/project.3mf`, zipSync(entries));
  writeFileSync(`${dir}/overrides.json`, JSON.stringify(overrides, null, 2));
  const stdout = execFileSync(`${root}scripts/orca-parity.sh`, ["--slice", "0", "--outputdir", dir, `${dir}/project.3mf`], { cwd: root, encoding: "utf8" });
  writeFileSync(`${dir}/official-slice.log`, stdout);
}
