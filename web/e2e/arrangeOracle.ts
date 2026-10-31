import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import type { Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { oraclePath } from "./oracleEnv";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

/** `[archive entry, text, replacement]`; each text must be present. */
export type Edit = readonly [string, string, string];

/** `fixture` with `edits` applied to its text entries. */
export function editedProject(fixture: string, edits: readonly Edit[]): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${fixture}`));
  for (const [name, from, to] of edits) {
    const text = strFromU8(entries[name]);
    if (!text.includes(from)) throw new Error(`${fixture}: ${name} has no ${JSON.stringify(from)}`);
    entries[name] = strToU8(text.replaceAll(from, to));
  }
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}

/**
 * An arrange as the exported project records it: per build item in build order its object name,
 * transform at the exporter's 9 significant digits (`bbs_3mf.cpp:7150-7151`) and printable flag,
 * and per plate its `[object name, instance index]` entries.
 */
export type Arranged = { items: [string, number[], boolean][]; plates: [string, number][][] };

/** A value at 9 significant digits, as `std::setprecision(float max_digits10)` writes it; `-0` reads as `0`. */
const digits = (value: number) => Number(value.toPrecision(9)) + 0;

const attribute = (element: string, name: string) => new RegExp(`${name}="([^"]*)"`).exec(element)![1];

/** The arrange Native exported to `project`. */
function exported(project: Uint8Array): Arranged {
  const entries = unzipSync(project);
  const objects = strFromU8(entries["Metadata/model_settings.config"]);
  const name = (id: string) => new RegExp(`<object id="${id}"[\\s\\S]*?key="name" value="([^"]*)"`).exec(objects)![1];
  const items = [...strFromU8(entries["3D/3dmodel.model"]).matchAll(/<item [^>]*\/>/g)].map(([item]): [string, number[], boolean] =>
    [name(attribute(item, "objectid")), attribute(item, "transform").split(" ").map((value) => digits(Number(value))), attribute(item, "printable") === "1"]);
  const plates = [...objects.matchAll(/<plate>([\s\S]*?)<\/plate>/g)].map(([, plate]) =>
    [...plate.matchAll(/key="object_id" value="(\d+)"\/>\s*<metadata key="instance_id" value="(\d+)"/g)]
      .map(([, object, instance]): [string, number] => [name(object), Number(instance)]));
  return { items, plates };
}

/**
 * The official OrcaSlicer CLI global arrange of `project` (`OrcaSlicer.cpp:5619-6098` at
 * v2.5.0-beta) with `flags`; `--slice <n>` arranges and exports plate `n` alone. `null` where CI
 * provides no oracle.
 */
export function orcaArrange(project: string, flags: readonly string[]): Arranged | null {
  if (!oraclePath("ORCA_APPDIR", "the official plater oracle", "an extraction of the official OrcaSlicer AppImage, see scripts/orca-parity.sh")) return null;
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-arrange-`)}/`;
  const output = `${work}arranged.3mf`;
  execFileSync(`${repositoryRoot}scripts/orca-parity.sh`, ["--arrange", "1", ...flags, "--export-3mf", output, project], {
    cwd: work,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const bytes = readFileSync(output);
  writeFileSync(`${project.replace(/\.3mf$/, "")}-arranged-native.3mf`, bytes);
  return exported(bytes);
}

type Probe = { objects: { name: string; instances: { printable: boolean; matrix: number[] }[] }[]; plates: [number, number][][] };

/**
 * The app's plater as Native would export it, from the viewport's instance probe; with `plate`
 * (0-based) only that plate's instances, the other plates listed empty, as `--slice` exports them.
 */
export async function appArranged(page: Page, plate?: number): Promise<Arranged> {
  const probe = JSON.parse((await page.locator(".viewport").getAttribute("data-instances"))!) as Probe;
  const kept = (object: number, instance: number) =>
    plate === undefined || probe.plates[plate].some(([o, i]) => o === object && i === instance);
  const items = probe.objects.flatMap(({ name, instances }, object) => instances.flatMap(({ printable, matrix }, instance): [string, number[], boolean][] =>
    kept(object, instance) ? [[name, [0, 1, 2, 3].flatMap((column) => [0, 1, 2].map((row) => digits(matrix[column * 4 + row]))), printable]] : []));
  const plates = probe.plates.map((entries) => entries.filter(([object, instance]) => kept(object, instance))
    .map(([object, instance]): [string, number] => [probe.objects[object].name, instance]));
  return { items, plates };
}
