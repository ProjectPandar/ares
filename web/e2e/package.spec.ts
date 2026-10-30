import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { build, preview } from "vite";

const web = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/package/", import.meta.url));
const consumer = `${artifacts}consumer/`;
const installed = `${consumer}node_modules/@ares-slicer/wasm/`;
const fixture = "tests/parity/arachne/default-prisms.3mf";

const PACKED_FILES = [
  "LICENSE",
  "README.md",
  "dist/ares_wasm_bg.wasm",
  "dist/configDef.d.ts",
  "dist/index.d.ts",
  "dist/index.js",
  "dist/platerSession.d.ts",
  "dist/presetCatalog.d.ts",
  "dist/protocol.d.ts",
  "dist/toolpaths.d.ts",
  "dist/types.d.ts",
  "dist/worker.d.ts",
  "dist/worker.js",
  "package.json",
];

const run = (command: string, args: string[], cwd: string) =>
  execFileSync(command, args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });

test("the packed @ares-slicer/wasm tarball works for a third-party consumer", async ({ page }) => {
  test.setTimeout(240_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(installed, { recursive: true });

  // npm pack (runs prepack: build + LICENSE): exactly the publishable files and registry metadata.
  const pack = run("npm", ["pack", "--json", "--pack-destination", artifacts], `${web}packages/ares-wasm-js`);
  const [packed] = JSON.parse(pack.slice(pack.lastIndexOf("\n[\n"))) as { filename: string; files: { path: string }[] }[];
  expect(packed.files.map((file) => file.path).sort()).toEqual(PACKED_FILES);
  run("tar", ["-xzf", `${artifacts}${packed.filename}`, "--strip-components=1", "-C", installed], artifacts);
  const manifest = JSON.parse(readFileSync(`${installed}package.json`, "utf8"));
  expect(manifest).toMatchObject({
    name: "@ares-slicer/wasm",
    license: "AGPL-3.0-or-later",
    repository: { type: "git", url: expect.stringContaining("github.com/ProjectPandar/ares"), directory: "web/packages/ares-wasm-js" },
    publishConfig: { access: "public" },
  });
  expect(manifest.dependencies ?? {}).toEqual({});

  // Type exports: a consumer outside the workspace, under both module resolutions.
  cpSync(`${web}e2e/consumer`, consumer, { recursive: true });
  for (const resolution of ["bundler", "nodenext"]) {
    try {
      run(`${web}node_modules/.bin/tsc`, ["-p", `tsconfig.${resolution}.json`], consumer);
    } catch (error) {
      throw new Error(`tsc (${resolution}) failed:\n${(error as { stdout: string }).stdout}`, { cause: error });
    }
  }

  // Runtime: the README usage, bundled by Vite from the extracted tarball, slices byte-identically to native.
  await build({ root: consumer, logLevel: "warn", build: { outDir: "dist", target: "es2022" } });
  const server = await preview({ root: consumer, logLevel: "warn", preview: { host: "127.0.0.1", port: 4175, strictPort: true } });
  try {
    await page.goto("http://127.0.0.1:4175/");
    await page.getByTestId("input").setInputFiles(`${repositoryRoot}${fixture}`);
    const result = page.getByTestId("result");
    await expect(result).not.toBeEmpty({ timeout: 120_000 });
    const browser = JSON.parse((await result.textContent())!);

    run(`${repositoryRoot}target/release/ares`, ["slice", "--plate", "1", "-o", `${artifacts}native.gcode`, fixture], repositoryRoot);
    const native = run("sha256sum", [`${artifacts}native.gcode`], artifacts).split(" ")[0];
    writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ tarball: packed.filename, files: PACKED_FILES, browser, native }, null, 2)}\n`);
    expect(browser).toEqual({ objects: 3, bytes: readFileSync(`${artifacts}native.gcode`).length, sha256: native });
  } finally {
    // PreviewServer.close destroys open sockets; httpServer.close waits on a browser preconnect forever.
    await server.close();
  }
});
