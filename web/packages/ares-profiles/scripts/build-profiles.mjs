// Generates dist/ from an OrcaSlicer checkout's resources/profiles: one ES
// module per vendor holding its index file and every sub_path file the index
// lists (PresetBundle::load_vendor_configs_from_json, BBL_JSON_KEY_*_LIST in
// Preset.hpp), plus an index module with vendor summaries and lazy loaders.
// Usage: ORCA_SLICER_DIR=<OrcaSlicer dir> node scripts/build-profiles.mjs
// (defaults to the repository's OrcaSlicer/ checkout).
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const orca = process.env.ORCA_SLICER_DIR ?? fileURLToPath(new URL("../../../../OrcaSlicer/", import.meta.url));
const profiles = `${orca}/resources/profiles`;
const dist = fileURLToPath(new URL("../dist/", import.meta.url));
const LISTS = ["machine_model_list", "process_list", "filament_list", "machine_list"];

const readJson = (path) => JSON.parse(readFileSync(path, "utf8").replace(/^\uFEFF/, ""));
const moduleName = (id) => id.replace(/[^\w.-]/g, "_");

const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: orca, encoding: "utf8" }).trim();
const ids = readdirSync(profiles).filter((file) => file.endsWith(".json")).map((file) => file.slice(0, -5)).sort();

rmSync(dist, { recursive: true, force: true });
mkdirSync(`${dist}vendors`, { recursive: true });
const summaries = ids.map((id) => {
  const index = readJson(`${profiles}/${id}.json`);
  const files = {};
  for (const list of LISTS) {
    for (const { sub_path: subPath } of index[list] ?? []) files[subPath] = readJson(`${profiles}/${id}/${subPath}`);
  }
  // JSON.parse of a string literal parses faster than the equivalent object literal.
  const json = JSON.stringify(JSON.stringify({ id, index, files }));
  writeFileSync(`${dist}vendors/${moduleName(id)}.js`, `export default JSON.parse(${json});\n`);
  return { id, name: index.name, version: index.version, description: index.description };
});

const loaders = ids.map((id) => `  ${JSON.stringify(id)}: () => import("./vendors/${moduleName(id)}.js"),`).join("\n");
writeFileSync(`${dist}index.js`, `export const ORCA_SLICER_COMMIT = ${JSON.stringify(commit)};

export const vendors = ${JSON.stringify(summaries, null, 2)};

const loaders = {
${loaders}
};

export async function loadVendor(id) {
  const load = loaders[id];
  if (!load) throw new Error("Unknown OrcaSlicer vendor: " + id);
  return (await load()).default;
}
`);
copyFileSync(fileURLToPath(new URL("../src/index.d.ts", import.meta.url)), `${dist}index.d.ts`);
console.log(`@ares-slicer/profiles: ${ids.length} vendors from OrcaSlicer ${commit.slice(0, 10)}`);
