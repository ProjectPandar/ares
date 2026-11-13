import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { build, preview } from "vite";

const web = fileURLToPath(new URL("../", import.meta.url));
const orca = process.env.ORCA_SLICER_DIR ?? fileURLToPath(new URL("../../OrcaSlicer/", import.meta.url));
const profilesDir = `${orca}/resources/profiles/`;
const artifacts = fileURLToPath(new URL("../../target/web-e2e/profiles/", import.meta.url));
const consumer = `${artifacts}consumer/`;
const installed = `${consumer}node_modules/@ares-slicer/profiles/`;

const LISTS = ["machine_model_list", "process_list", "filament_list", "machine_list"] as const;
type Entry = { name: string; sub_path: string };
type VendorIndex = Record<(typeof LISTS)[number], Entry[]> & Record<string, unknown>;

const run = (command: string, args: string[], cwd: string) =>
  execFileSync(command, args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
const readJson = (path: string) => JSON.parse(readFileSync(path, "utf8").replace(/^\uFEFF/u, ""));

test("@ares-slicer/profiles mirrors OrcaSlicer resources/profiles as lazy per-vendor data", async ({ page }) => {
  test.setTimeout(300_000);
  expect(existsSync(profilesDir), `OrcaSlicer checkout with resources/profiles at ${orca}`).toBe(true);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(installed, { recursive: true });

  // npm pack (prepack regenerates dist from the checkout): data only, no runtime dependencies.
  const pack = run("npm", ["pack", "--json", "--pack-destination", artifacts], `${web}packages/ares-profiles`);
  const [packed] = JSON.parse(pack.slice(pack.lastIndexOf("\n[\n"))) as { filename: string; files: { path: string }[] }[];
  run("tar", ["-xzf", `${artifacts}${packed.filename}`, "--strip-components=1", "-C", installed], artifacts);
  const manifest = readJson(`${installed}package.json`);
  expect(manifest).toMatchObject({ name: "@ares-slicer/profiles", license: "AGPL-3.0-or-later", publishConfig: { access: "public" } });
  expect(manifest.dependencies ?? {}).toEqual({});

  // Fidelity: every vendor index (PresetBundle::load_vendor_configs_from_json loads each top-level
  // *.json) and every sub_path file it lists, equal to the source JSON.
  const ids = readdirSync(profilesDir).filter((file) => file.endsWith(".json")).map((file) => file.slice(0, -5)).sort();
  const data = await import(pathToFileURL(`${installed}dist/index.js`).href);
  expect(data.ORCA_SLICER_COMMIT).toBe(run("git", ["rev-parse", "HEAD"], orca).trim());
  expect(data.vendors.map((vendor: { id: string }) => vendor.id)).toEqual(ids);
  const vendorFiles = packed.files.map((file) => file.path).filter((path) => path.startsWith("dist/vendors/"));
  expect(vendorFiles).toHaveLength(ids.length);
  let profileCount = 0;
  for (const id of ids) {
    const source = readJson(`${profilesDir}${id}.json`) as VendorIndex;
    const { id: loadedId, index, files } = await data.loadVendor(id);
    expect(loadedId).toBe(id);
    expect(index).toEqual(source);
    expect(data.vendors.find((vendor: { id: string }) => vendor.id === id)).toEqual({
      id, name: source.name, version: source.version, description: source.description,
    });
    const subPaths = LISTS.flatMap((list) => (source[list] ?? []).map((entry) => entry.sub_path));
    expect(Object.keys(files).sort(), id).toEqual([...new Set(subPaths)].sort());
    for (const subPath of subPaths) expect(files[subPath], `${id}/${subPath}`).toEqual(readJson(`${profilesDir}${id}/${subPath}`));
    profileCount += subPaths.length;
  }

  // Types resolve for a nodenext consumer; Vite splits one chunk per vendor out of a small entry.
  cpSync(`${web}e2e/profiles-consumer`, consumer, { recursive: true });
  try {
    run(`${web}node_modules/.bin/tsc`, ["-p", "tsconfig.json"], consumer);
  } catch (error) {
    throw new Error(`tsc failed:\n${(error as { stdout: string }).stdout}`, { cause: error });
  }
  const output = await build({ root: consumer, logLevel: "warn", build: { outDir: "dist", target: "es2022", chunkSizeWarningLimit: 8_000 } });
  const chunks = (Array.isArray(output) ? output : [output]).flatMap((result) => ("output" in result ? result.output : []))
    .filter((chunk) => chunk.type === "chunk");
  const entry = chunks.find((chunk) => chunk.isEntry)!;
  const entryGzip = gzipSync(entry.code).length;
  expect(entry.code).not.toContain("Snapmaker A250 (0.4 nozzle)");
  expect(entryGzip).toBeLessThan(8_000);
  const vendorChunks = chunks.filter((chunk) => chunk.isDynamicEntry);
  expect(vendorChunks).toHaveLength(ids.length);
  const vendorGzip = vendorChunks.reduce((total, chunk) => total + gzipSync(chunk.code).length, 0);

  // Browser: the vendor list needs no vendor chunk; loading Snapmaker fetches exactly its chunk.
  const snapmaker = readJson(`${profilesDir}Snapmaker.json`) as VendorIndex;
  const printers = snapmaker.machine_list.map(({ sub_path: subPath }) => readJson(`${profilesDir}Snapmaker/${subPath}`).name as string);
  const scripts: string[] = [];
  page.on("request", (request) => {
    if (request.resourceType() === "script") scripts.push(new URL(request.url()).pathname);
  });
  const server = await preview({ root: consumer, logLevel: "warn", preview: { host: "127.0.0.1", port: 4176, strictPort: true } });
  try {
    await page.goto("http://127.0.0.1:4176/");
    await expect(page.getByTestId("vendors").getByRole("listitem")).toHaveText(ids);
    expect(scripts).toEqual([`/${entry.fileName}`]);
    await page.getByTestId("load").click();
    await expect(page.getByTestId("printers").getByRole("listitem")).toHaveText(printers);
    expect(scripts.slice(1)).toHaveLength(1);
    expect(scripts[1]).toMatch(/\/Snapmaker-[\w-]+\.js$/);
    await page.screenshot({ path: `${artifacts}consumer.png` });
  } finally {
    // PreviewServer.close destroys open sockets; httpServer.close waits on a browser preconnect forever.
    await server.close();
  }

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    tarball: packed.filename,
    orcaSlicerCommit: data.ORCA_SLICER_COMMIT,
    vendors: ids.length,
    profiles: profileCount,
    entryGzipBytes: entryGzip,
    vendorChunksGzipBytes: vendorGzip,
    snapmakerPrinters: printers.length,
  }, null, 2)}\n`);
});
