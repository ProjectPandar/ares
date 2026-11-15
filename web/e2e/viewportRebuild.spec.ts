import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { choose, PRESETS } from "./lhPresets";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/viewport-rebuild/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates. */
const PROJECT = `${repositoryRoot}tests/parity/custom-gcode-overview/two-plates.3mf`;
const CUBE = `${repositoryRoot}tests/parity/cube10.stl`;
const SWITCHED_PROCESS = "0.20mm Quiet @LH Stinger";

/** Counts shader programs linked and buffers uploaded from page start; an upload marks a rebuilt geometry. */
function countGl() {
  const gl = { links: 0, uploads: 0 };
  (window as unknown as { gl: typeof gl }).gl = gl;
  for (const context of [WebGLRenderingContext, WebGL2RenderingContext]) {
    const { linkProgram, bufferData } = context.prototype;
    context.prototype.linkProgram = function (this: WebGLRenderingContext, program: WebGLProgram) {
      gl.links += 1;
      linkProgram.call(this, program);
    };
    context.prototype.bufferData = function (this: WebGLRenderingContext, ...args: unknown[]) {
      gl.uploads += 1;
      (bufferData as (...values: unknown[]) => void).apply(this, args);
    };
  }
}

const counts = (page: Page) => page.evaluate(() => ({ ...(window as unknown as { gl: { links: number; uploads: number } }).gl }));

/** The counts once no draw is waiting (data-drawing, Viewport.render) and nothing uploaded between two polls. */
async function quiet(page: Page) {
  let last = "";
  await expect.poll(async () => {
    if ((await page.getByTestId("viewport").getAttribute("data-drawing")) !== null) return false;
    const [previous, next] = [last, JSON.stringify(await counts(page))];
    last = next;
    return previous === next;
  }, { intervals: [500], timeout: 60_000 }).toBe(true);
  return JSON.parse(last) as { links: number; uploads: number };
}

/** Links and uploads of `edit` once the viewport is quiet again; the edit must upload a rebuilt geometry. */
async function rebuild(page: Page, edit: () => Promise<void>) {
  const before = await quiet(page);
  await edit();
  await expect.poll(async () => (await counts(page)).uploads, "the edit rebuilds geometry").toBeGreaterThan(before.uploads);
  const after = await quiet(page);
  return { links: after.links - before.links, uploads: after.uploads - before.uploads };
}

/** A fresh artifact directory of one test. */
function artifactsOf(name: string) {
  const directory = `${artifacts}${name}/`;
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  return directory;
}

test("a new scene and a new plate rebuild plates and instances without compiling shaders, as GLShadersManager::init compiles them once", async ({ page }) => {
  test.setTimeout(300_000);
  const directory = artifactsOf("project");
  const results: Record<string, { links: number; uploads: number }> = {};
  await page.addInitScript(countGl);
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(PROJECT);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  const tree = page.getByTestId("object-list");
  await expect(tree.getByRole("treeitem")).toHaveCount(4);

  results["delete object"] = await rebuild(page, async () => {
    await tree.getByRole("treeitem").nth(1).click({ button: "right" });
    await page.getByRole("menu", { name: "Object menu" }).getByRole("menuitem", { name: "Delete" }).click();
    await expect(tree.getByRole("treeitem")).toHaveCount(3);
  });
  results["add plate"] = await rebuild(page, async () => {
    await page.getByRole("button", { name: "Add plate", exact: true }).click();
    await expect(tree.getByRole("treeitem", { name: /^Plate 3\b/ })).toBeVisible();
  });
  await page.screenshot({ path: `${directory}project.png` });
  writeFileSync(`${directory}result.json`, `${JSON.stringify(results, null, 2)}\n`);

  expect(results["delete object"].links, "programs linked by the object delete").toBe(0);
  expect(results["add plate"].links, "programs linked by the added plate").toBe(0);
});

test("a preset change that opens a new plater session for an STL keeps the shader programs", async ({ page }) => {
  test.setTimeout(300_000);
  const directory = artifactsOf("stl");
  const results: Record<string, { links: number; uploads: number }> = {};
  await page.addInitScript(countGl);
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(CUBE);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: `All ${PRESETS.vendor} models`, exact: true }).click();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await choose(page, "mouse", "Printer preset", "System presets", PRESETS.printer);
  await choose(page, "mouse", "Process preset", "System presets", PRESETS.process);
  await choose(page, "mouse", "Filament 1 preset", "System presets", PRESETS.filament);
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);

  results["process preset"] = await rebuild(page, () => choose(page, "mouse", "Process preset", "System presets", SWITCHED_PROCESS));
  await page.screenshot({ path: `${directory}stl.png` });
  writeFileSync(`${directory}result.json`, `${JSON.stringify(results, null, 2)}\n`);

  expect(results["process preset"].links, "programs linked by the new plater session").toBe(0);
});
