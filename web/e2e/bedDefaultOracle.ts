import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { loadVendor, ORCA_SLICER_COMMIT } from "@ares-slicer/profiles";

export interface BedSelection {
  printer: string;
  process: string;
  filaments: string[];
}

export function bedCommands(gcode: string) {
  return gcode.split(/\r?\n/).filter((line) => /^M(?:140|190)\b/.test(line));
}

export function firstBedCommands(gcode: string) {
  const commands = bedCommands(gcode);
  return [commands.find((line) => line.startsWith("M140 ")), commands.find((line) => line.startsWith("M190 "))];
}

/** Independent native CLI inputs from unchanged pinned resource files. */
export async function nativeBedReference(binary: string, selection: BedSelection, model: string, directory: string) {
  mkdirSync(directory, { recursive: true });
  const vendors = await Promise.all([loadVendor("Snapmaker"), loadVendor("OrcaFilamentLibrary")]);
  const repository = fileURLToPath(new URL("../../", import.meta.url));
  const source = process.env.ORCA_SLICER_DIR ?? `${repository}OrcaSlicer`;
  const inputs = new Map<string, string>();
  for (const vendor of vendors) {
    const paths = [`${vendor.id}.json`, ...Object.keys(vendor.files).map((path) => `${vendor.id}/${path}`)];
    for (const relative of paths) {
      const path = `${source}/resources/profiles/${relative}`;
      inputs.set(path, createHash("sha256").update(readFileSync(path)).digest("hex"));
    }
  }
  inputs.set(model, createHash("sha256").update(readFileSync(model)).digest("hex"));
  const locate = (kind: "machine" | "process" | "filament", name: string) => {
    for (const vendor of vendors) {
      const entry = vendor.index[`${kind}_list`].find((item) => item.name === name);
      if (entry) return `${source}/resources/profiles/${vendor.id}/${entry.sub_path}`;
    }
    throw new Error(`Missing ${kind} preset ${name}`);
  };
  const machine = locate("machine", selection.printer);
  const processFile = locate("process", selection.process);
  const filaments = selection.filaments.map((name) => locate("filament", name));
  const inheritedHint = (name: string): string => {
    const document = JSON.parse(readFileSync(locate("machine", name), "utf8"));
    return "default_bed_type" in document ? document.default_bed_type : inheritedHint(document.inherits);
  };
  const hint = inheritedHint(selection.printer);
  // Preset.cpp:1146–1149, PrintConfig.cpp:606–615: the U1 hint takes the concrete-name branch.
  const bedType = ["Cool Plate", "Engineering Plate", "High Temp Plate", "Textured PEI Plate", "Textured Cool Plate", "Supertack Plate"].find((name) => name === hint);
  if (!bedType) throw new Error(`The U1 resource no longer declares a concrete bed name: ${hint}`);
  const args = ["--slice", "0", "--curr-bed-type", bedType, "--load-settings", `${machine};${processFile}`, "--load-filaments", filaments.join(";"), "--outputdir", directory, model];
  writeFileSync(`${directory}/command.json`, JSON.stringify({ binary, binary_sha256: createHash("sha256").update(readFileSync(binary)).digest("hex"), args, cwd: directory, selection, gui_bed_initialization: { hint, bedType, source: "Plater.cpp:3876–3908; Preset.cpp:1140–1175" }, resource_commit: ORCA_SLICER_COMMIT, inputs: Object.fromEntries(inputs) }, null, 2));
  const result = spawnSync(binary, args, { cwd: directory, env: { ...process.env, ORCA_JOB_APPDIR: `${directory}/appdir` }, encoding: "utf8", timeout: 240_000, maxBuffer: 16 * 1024 * 1024 });
  writeFileSync(`${directory}/stdout.log`, result.stdout ?? "");
  writeFileSync(`${directory}/stderr.log`, result.stderr ?? "");
  writeFileSync(`${directory}/status.json`, JSON.stringify({ status: result.status, signal: result.signal, error: result.error?.message }));
  if (result.status !== 0) throw new Error(`Native bed reference failed; see ${directory}/status.json and stderr.log`);
  return readFileSync(`${directory}/plate_1.gcode`, "utf8");
}
