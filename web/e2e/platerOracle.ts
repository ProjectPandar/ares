import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import type { Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";
import { openedSelection, withSelectedPresets } from "./selectedPresetLines";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

/** One build item of the oracle project: its plate identity and column-major matrix. */
export type OracleItem = { objectId: number; instanceId: number; identifyId: number; matrix: readonly number[] };
/** One plate: the loaded plate (0-based) whose metadata it keeps, or `null` for an added plate, and its items. */
export type OraclePlate = { source: number | null; items: OracleItem[] };
/** The session's prime tower positions, `[wipe_tower_x, wipe_tower_y]` with one entry per plate. */
export type OracleTowers = readonly [readonly number[], readonly number[]];

/** The app session's tower positions from the viewport probe (`PlaterLayout.wipeTower`). */
export async function probedTowers(page: Page): Promise<OracleTowers> {
  const probe = JSON.parse((await page.locator(".viewport").getAttribute("data-instances"))!) as { wipeTower: OracleTowers };
  return probe.wipeTower;
}

/** A column-major 4x4 matrix: rotations applied in order about the origin, then `offset`. */
export function matrix(rotations: readonly (readonly number[][])[], offset: readonly [number, number, number]): number[] {
  const multiply = (a: readonly number[][], b: readonly number[][]) =>
    a.map((row) => [0, 1, 2].map((column) => row.reduce((sum, value, k) => sum + value * b[k][column], 0)));
  const linear = rotations.reduce<number[][]>((acc, rotation) => multiply(rotation, acc), [[1, 0, 0], [0, 1, 0], [0, 0, 1]]);
  return [0, 1, 2, 3].flatMap((column) => column === 3 ? [...offset, 1] : [...[0, 1, 2].map((row) => linear[row][column]), 0]);
}

export const aboutX = (degrees: number) => {
  const [sin, cos] = [Math.sin((degrees * Math.PI) / 180), Math.cos((degrees * Math.PI) / 180)];
  return [[1, 0, 0], [0, cos, -sin], [0, sin, cos]];
};
export const aboutZ = (degrees: number) => {
  const [sin, cos] = [Math.sin((degrees * Math.PI) / 180), Math.cos((degrees * Math.PI) / 180)];
  return [[cos, -sin, 0], [sin, cos, 0], [0, 0, 1]];
};

const attribute = (element: string, name: string) => new RegExp(`${name}="([^"]*)"`).exec(element)![1];

/** `3D/3dmodel.model` build items with the plates' matrices (`bbs_3mf.cpp:8050-8073`, 3MF 3x4 column order). */
function rewriteBuild(model: string, items: OracleItem[]) {
  const seen = new Map<number, number>();
  return model.replace(/<item [^>]*\/>/g, (item) => {
    const objectId = Number(attribute(item, "objectid"));
    const instanceId = seen.get(objectId) ?? 0;
    seen.set(objectId, instanceId + 1);
    const { matrix: m } = items.find((entry) => entry.objectId === objectId && entry.instanceId === instanceId)!;
    const transform = [0, 1, 2, 3].flatMap((column) => [0, 1, 2].map((row) => String(m[column * 4 + row]))).join(" ");
    return item.replace(/transform="[^"]*"/, `transform="${transform}"`);
  });
}

/** `model_settings.config` plates (`bbs_3mf.cpp:7918`): a loaded plate keeps its metadata, an added one has the defaults. */
function rewritePlates(settings: string, plates: OraclePlate[]) {
  const blocks = [...settings.matchAll(/ {2}<plate>\n[\s\S]*?<\/plate>\n/g)].map(([block]) => block);
  const first = settings.indexOf("  <plate>");
  const last = settings.lastIndexOf("</plate>\n") + "</plate>\n".length;
  const written = plates.map(({ source, items }, index) => {
    const metadata = source === null
      ? ["    <metadata key=\"plater_name\" value=\"\"/>", "    <metadata key=\"locked\" value=\"false\"/>"]
      : blocks[source].split("\n").filter((line) => line.startsWith("    <metadata") && !line.includes("\"plater_id\""));
    const instances = items.map(({ objectId, instanceId, identifyId }) =>
      `    <model_instance>\n      <metadata key="object_id" value="${objectId}"/>\n      <metadata key="instance_id" value="${instanceId}"/>\n      <metadata key="identify_id" value="${identifyId}"/>\n    </model_instance>\n`);
    return `  <plate>\n    <metadata key="plater_id" value="${index + 1}"/>\n${metadata.map((line) => `${line}\n`).join("")}${instances.join("")}  </plate>\n`;
  });
  return `${settings.slice(0, first)}${written.join("")}${settings.slice(last)}`;
}

/** `filament_sequence.json`: one entry per plate, kept from its source plate. */
function rewriteSequences(sequences: string, plates: OraclePlate[]) {
  const old = JSON.parse(sequences) as Record<string, unknown>;
  const empty = { nozzle_sequence: [], optimal_assignment: [], sequence: [] };
  return JSON.stringify(Object.fromEntries(plates.map(({ source }, index) =>
    [`plate_${index + 1}`, source === null ? empty : old[`plate_${source + 1}`]])));
}

/**
 * `project_settings.config` with the session's tower positions: the GUI writes its whole project
 * config, each float as `ss << v` prints it (6 significant digits, `Config.hpp:974-976`).
 */
function rewriteTowers(settings: string, [x, y]: OracleTowers) {
  const serialized = (values: readonly number[]) => values.map((value) => String(Number(value.toPrecision(6))));
  return JSON.stringify({ ...JSON.parse(settings), wipe_tower_x: serialized(x), wipe_tower_y: serialized(y) }, null, 4);
}

/**
 * The project the GUI exports after plater edits: `fixture` with the plates' build item
 * matrices, plate map, filament sequences and the session's tower positions; `outside` items lie
 * on no plate. Every other entry, including the plate-index keyed custom G-codes, is unchanged.
 */
export function guiExport(fixture: string, plates: OraclePlate[], towers: OracleTowers, outside: OracleItem[] = []): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${fixture}`));
  const items = [...plates.flatMap((plate) => plate.items), ...outside];
  const rewrite = (name: string, edit: (text: string) => string) => {
    if (entries[name]) entries[name] = strToU8(edit(strFromU8(entries[name])));
  };
  rewrite("3D/3dmodel.model", (model) => rewriteBuild(model, items));
  rewrite("Metadata/model_settings.config", (settings) => rewritePlates(settings, plates));
  rewrite("Metadata/filament_sequence.json", (sequences) => rewriteSequences(sequences, plates));
  rewrite("Metadata/project_settings.config", (settings) => rewriteTowers(settings, towers));
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}

/** `orcaPlate`'s copy of the oracle G-code of `plate` of `project` on `leg`. */
const oracleGcode = (project: string, plate: number, leg: "official" | "seeded") => `${project.replace(/\.3mf$/, "")}-plate_${plate}-${leg}.gcode`;

/**
 * Orca slicing plate `plate` of `project`: official, or the seeded replay with seed 0 as the browser slices. With
 * `opened`, the 3MF the app opened, the lines its preset selection decides are the probe's (selectedPresetLines.ts):
 * the plater's export of these first-run projects names project-local presets, which the Native CLI rejects with
 * -17 (OrcaSlicer.cpp:2575-2620).
 */
export function orcaPlate(project: string, plate: number, leg: "official" | "seeded", opened?: string): string | null {
  const official = oraclePath("ORCA_APPDIR", "the official plater oracle", "an extraction of the official OrcaSlicer AppImage, see scripts/orca-parity.sh");
  const replay = oraclePath("ORCA_REPLAY_BIN", "the seeded plater oracle", "the seed-replay OrcaSlicer, see scripts/orca-seeded-parity.sh");
  if (!(leg === "official" ? official : replay)) return null;
  const selection = opened === undefined ? undefined : openedSelection(opened, `${project.replace(/\.3mf$/, "")}-plate_${plate}-${leg}-selection/`);
  if (selection === null) throw new Error(`ARES_ORCA_PRESET_SELECT_BIN is unset while the ${leg} plater oracle is set: the comparison needs the plater's selected presets`);
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-plater-${leg}-`)}/`;
  // The replay stays on this run's first allowed CPU.
  const cpu = /^Cpus_allowed_list:\s*(\d+)/m.exec(readFileSync("/proc/self/status", "utf8"))![1];
  const script = leg === "official" ? "orca-parity.sh" : "orca-seeded-parity.sh";
  execFileSync(`${repositoryRoot}scripts/${script}`, ["--slice", String(plate), "--outputdir", work, project], {
    cwd: work,
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, ORCA_REPLAY_SEED: "0", ORCA_REPLAY_CPU: cpu },
  });
  const gcode = readFileSync(`${work}plate_${plate}.gcode`, "utf8");
  writeFileSync(oracleGcode(project, plate, leg), gcode);
  const native = normalize(gcode, "OrcaSlicer");
  return selection ? withSelectedPresets(native, selection) : native;
}

/**
 * The oracle that ran `orcaPlate(project, plate, leg)`, whose result is `native`: its install and the version
 * its own G-code header names (`; generated by <version> on`), or "skipped".
 */
export function oracleRun(project: string, plate: number, leg: "official" | "seeded", native: string | null) {
  if (native === null) return "skipped";
  const header = /^; generated by (.+) on /m.exec(readFileSync(oracleGcode(project, plate, leg), "utf8"))!;
  return { path: process.env[leg === "official" ? "ORCA_APPDIR" : "ORCA_REPLAY_BIN"]!, version: header[1] };
}
