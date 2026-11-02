#!/usr/bin/env node
import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve, sep } from "node:path";
import { parseArgs } from "node:util";

const lists = ["machine_model_list", "machine_list", "process_list", "filament_list"];
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

async function main() {
  const { values } = parseArgs({
    options: { profiles: { type: "string" }, out: { type: "string" } },
    allowPositionals: false,
  });
  if (!values.profiles || !values.out) {
    throw new Error("usage: create-project-preset-catalog.mjs --profiles <profiles> --out <new-directory>");
  }
  const source = resolve(values.profiles);
  const output = resolve(values.out);
  if (output === source || output.startsWith(source + sep)) {
    throw new Error("catalog output must not modify the original profiles tree");
  }
  const manifests = [];
  for (const entry of (await readdir(source, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name, "en"))) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
    const bytes = await readFile(join(source, entry.name));
    const index = JSON.parse(bytes.toString("utf8"));
    if (!lists.some((list) => Array.isArray(index[list]))) continue;
    manifests.push({ id: entry.name.slice(0, -5), index, bytes });
  }
  if (manifests.length === 0) {
    throw new Error("profiles tree has no original vendor manifests; stage the pinned source documents, not an exported-settings snapshot or caches");
  }
  // Require a fresh destination: earlier runtime evidence is never overwritten.
  await mkdir(output);
  const system = join(output, "native", "system");
  await mkdir(system, { recursive: true });
  const vendors = [];
  const hashes = {};
  for (const { id, index, bytes } of manifests) {
    await writeFile(join(system, `${id}.json`), bytes, { flag: "wx" });
    hashes[`${id}.json`] = digest(bytes);
    const files = {};
    const paths = new Set(lists.flatMap((list) => (index[list] ?? []).map((entry) => entry.sub_path)));
    for (const path of [...paths].sort()) {
      const original = resolve(source, id, path);
      const within = relative(join(source, id), original);
      if (within === ".." || within.startsWith(".." + sep) || resolve(join(source, id), within) !== original) {
        throw new Error(`vendor profile escapes its source tree: ${id}/${path}`);
      }
      const target = join(system, id, within);
      const contents = await readFile(original);
      files[path] = JSON.parse(contents.toString("utf8"));
      await mkdir(dirname(target), { recursive: true });
      await copyFile(original, target);
      hashes[`${id}/${path}`] = digest(contents);
    }
    vendors.push({ id, index, files });
  }
  const catalog = JSON.stringify({ vendors, imports: [] });
  await writeFile(join(output, "catalog.json"), catalog + "\n", { flag: "wx" });
  await writeFile(join(output, "source-hashes.json"), JSON.stringify({
    catalog_sha256: digest(Buffer.from(catalog + "\n")),
    vendors: vendors.length,
    documents: Object.keys(hashes).length,
    sha256: hashes,
  }, null, 2) + "\n", { flag: "wx" });
  console.log(JSON.stringify({ vendors: vendors.length, documents: Object.keys(hashes).length }));
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
