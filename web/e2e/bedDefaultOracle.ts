import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { loadVendor, ORCA_SLICER_COMMIT, type ProfileJson } from "@ares-slicer/profiles";

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

/** Independent native CLI inputs from the complete pinned resource documents. */
export async function nativeBedReference(binary: string, selection: BedSelection, model: string, directory: string) {
  mkdirSync(directory, { recursive: true });
  const vendors = await Promise.all([loadVendor("Snapmaker"), loadVendor("OrcaFilamentLibrary")]);
  const repository = fileURLToPath(new URL("../../", import.meta.url));
  const source = process.env.ORCA_SLICER_DIR ?? `${repository}OrcaSlicer`;
  const inputs = new Map<string, string>();
  const flatten = (kind: "machine" | "process" | "filament", name: string): ProfileJson => {
    for (const vendor of vendors) {
      const entry = vendor.index[`${kind}_list`].find((item) => item.name === name);
      if (!entry) continue;
      const path = `${source}/resources/profiles/${vendor.id}/${entry.sub_path}`;
      inputs.set(path, createHash("sha256").update(readFileSync(path)).digest("hex"));
      const document = vendor.files[entry.sub_path];
      return { ...(document.inherits ? flatten(kind, document.inherits) : {}), ...document };
    }
    throw new Error(`Missing ${kind} parent ${name}`);
  };
  const save = (kind: "machine" | "process" | "filament", name: string, filename: string) => {
    const path = `${directory}/${filename}.json`;
    writeFileSync(path, JSON.stringify(flatten(kind, name)));
    return path;
  };
  const machine = save("machine", selection.printer, "machine");
  const processFile = save("process", selection.process, "process");
  const filaments = selection.filaments.map((name, index) => save("filament", name, `filament-${index}`));
  const args = ["--slice", "0", "--load-settings", `${machine};${processFile}`, "--load-filaments", filaments.join(";"), "--outputdir", directory, model];
  writeFileSync(`${directory}/command.json`, JSON.stringify({ binary, args, selection, resource_commit: ORCA_SLICER_COMMIT, inputs: Object.fromEntries(inputs) }, null, 2));
  const output = execFileSync(binary, args, { cwd: directory, env: { ...process.env, ORCA_JOB_APPDIR: `${directory}/appdir` }, stdio: ["ignore", "pipe", "pipe"] });
  writeFileSync(`${directory}/stdout.log`, output);
  return readFileSync(`${directory}/plate_1.gcode`, "utf8");
}
