import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/import/", import.meta.url));
const prisms = `${repositoryRoot}tests/parity/arachne/default-prisms.3mf`;
const cube = `${repositoryRoot}tests/parity/cube10.stl`;

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/** Drops files on the workspace through a synthetic DataTransfer, as a desktop drag would. */
async function dropFiles(page: Page, files: { name: string; bytes: Buffer }[]) {
  const payload = files.map(({ name, bytes }) => ({ name, data: bytes.toString("base64") }));
  const transfer = await page.evaluateHandle((entries) => {
    const data = new DataTransfer();
    for (const { name, data: base64 } of entries) {
      data.items.add(new File([Uint8Array.from(atob(base64), (char) => char.charCodeAt(0))], name));
    }
    return data;
  }, payload);
  const target = page.getByTestId("workspace");
  await target.dispatchEvent("dragenter", { dataTransfer: transfer });
  await target.dispatchEvent("dragover", { dataTransfer: transfer });
  await expect(page.getByTestId("drop-overlay")).toBeVisible();
  await target.dispatchEvent("drop", { dataTransfer: transfer });
  await expect(page.getByTestId("drop-overlay")).toHaveCount(0);
}

/** Fraction of viewport pixels that are saturated, i.e. model colour rather than grey bed/background. */
function modelCoverage(page: Page) {
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const probe = document.createElement("canvas");
    probe.width = 320;
    probe.height = 180;
    const context = probe.getContext("2d")!;
    context.drawImage(canvas, 0, 0, 320, 180);
    const pixels = context.getImageData(0, 0, 320, 180).data;
    let saturated = 0;
    for (let index = 0; index < pixels.length; index += 4) {
      const [r, g, b] = [pixels[index], pixels[index + 1], pixels[index + 2]];
      if (Math.max(r, g, b) - Math.min(r, g, b) > 60) saturated++;
    }
    return saturated / (pixels.length / 4);
  });
}

/** FNV-1a of the full-resolution canvas pixels: changes whenever a new frame differs. */
function canvasFingerprint(page: Page) {
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const copy = document.createElement("canvas");
    copy.width = canvas.width;
    copy.height = canvas.height;
    const context = copy.getContext("2d")!;
    context.drawImage(canvas, 0, 0);
    const pixels = context.getImageData(0, 0, copy.width, copy.height).data;
    let hash = 0x811c9dc5;
    for (let index = 0; index < pixels.length; index++) hash = Math.imul(hash ^ pixels[index], 0x01000193);
    return hash >>> 0;
  });
}

test("drag-drop import, object list, camera views", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });
  const status = page.getByTestId("status");
  const rows = page.getByTestId("object-list").getByRole("treeitem");

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });

  // Only .3mf/.stl are importable; anything else is rejected before the worker.
  await dropFiles(page, [{ name: "notes.txt", bytes: Buffer.from("not a model") }]);
  await expect(status).toHaveText("Import failed: notes.txt is not a 3MF or STL file");
  await dropFiles(page, [
    { name: "a.stl", bytes: readFileSync(cube) },
    { name: "b.stl", bytes: readFileSync(cube) },
  ]);
  await expect(status).toHaveText("Import failed: drop one 3MF or STL file at a time");

  // STL by drag-drop.
  await dropFiles(page, [{ name: "cube10.stl", bytes: readFileSync(cube) }]);
  await expect(status).toHaveText("Ready to slice");
  await expect(rows).toHaveCount(1);
  await expect(rows.first()).toContainText("cube10");

  // Multi-object 3MF by drag-drop, dropped on the Preview page.
  await page.getByRole("link", { name: "Preview" }).click();
  await dropFiles(page, [{ name: "default-prisms.3mf", bytes: readFileSync(prisms) }]);
  await expect(status).toHaveText("Ready to slice");
  await page.getByRole("link", { name: "Prepare" }).click();
  // ObjectList::reload_all_plates: the plate node, then its objects.
  await expect(rows).toHaveCount(4);
  await expect(rows.first()).toHaveText("Plate 1");
  for (const name of ["wide.stl", "neck.stl", "hole.stl"]) await expect(rows.filter({ hasText: name })).toHaveCount(1);
  await expect(page.getByTestId("object-count")).toHaveText("3");

  // Default Prepare camera frames the plate (zoom_to_plate); Fit frames the objects (zoom_to_volumes).
  await expect.poll(() => modelCoverage(page)).toBeGreaterThan(0.001);
  const plateCoverage = await modelCoverage(page);
  await page.screenshot({ path: `${artifacts}prepare-plate.png` });
  await page.getByRole("button", { name: "Fit to objects" }).click();
  await expect.poll(() => modelCoverage(page)).toBeGreaterThan(plateCoverage * 4);
  const fitCoverage = await modelCoverage(page);
  await page.screenshot({ path: `${artifacts}prepare-fit.png` });

  // GLCanvas3D::on_char Ctrl+digit view keys: 1 top, 3 front, 0 plate. The Top button presses
  // before the top frame draws, so wait for a settled frame that differs from the fit one
  // (data-drawing marks a queued render, Viewport.render).
  const viewport = page.getByTestId("viewport");
  await expect(viewport).not.toHaveAttribute("data-drawing");
  const fitFrame = await canvasFingerprint(page);
  await page.keyboard.press("ControlOrMeta+1");
  await expect(page.getByTestId("view-toolbar").getByRole("button", { name: "Top", pressed: true })).toBeVisible();
  await expect
    .poll(async () => (await viewport.getAttribute("data-drawing")) === null && (await canvasFingerprint(page)) !== fitFrame)
    .toBe(true);
  await page.screenshot({ path: `${artifacts}prepare-top.png` });
  await page.getByRole("button", { name: "Front" }).click();
  await expect(page.getByTestId("view-toolbar").getByRole("button", { name: "Front", pressed: true })).toBeVisible();
  await page.keyboard.press("ControlOrMeta+0");
  await expect.poll(() => modelCoverage(page)).toBeLessThan(fitCoverage / 4);

  // Selecting in the list highlights the object and its row. The row commits before the
  // viewport effect queues the highlight frame, so wait for a settled frame that differs
  // from the unselected one.
  await expect(viewport).not.toHaveAttribute("data-drawing");
  const unselected = await canvasFingerprint(page);
  await rows.filter({ hasText: "neck.stl" }).click();
  await expect(rows.filter({ hasText: "neck.stl" })).toHaveAttribute("aria-selected", "true");
  await expect
    .poll(async () => (await viewport.getAttribute("data-drawing")) === null && (await canvasFingerprint(page)) !== unselected)
    .toBe(true);
  await page.screenshot({ path: `${artifacts}prepare-selected.png` });

  // Multi-object plate slices byte-identical to native ares-cli.
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(status).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Export G-code/ }).click(),
  ]);
  const gcodePath = `${artifacts}${download.suggestedFilename()}`;
  await download.saveAs(gcodePath);
  const gcode = readFileSync(gcodePath);
  const nativePath = `${artifacts}native.gcode`;
  execFileSync(`${repositoryRoot}target/release/ares`, ["slice", "--plate", "1", "-o", nativePath, prisms]);
  expect(sha256(gcode), "browser G-code must match native ares-cli bytes").toBe(sha256(readFileSync(nativePath)));

  // Preview opens fitted to the objects.
  await page.getByRole("link", { name: "Preview" }).click();
  await expect.poll(() => modelCoverage(page)).toBeGreaterThan(plateCoverage * 4);
  await page.screenshot({ path: `${artifacts}preview-fit.png` });

  writeFileSync(
    `${artifacts}result.json`,
    `${JSON.stringify({ fixture: "tests/parity/arachne/default-prisms.3mf", objects: 3, bytes: gcode.length, sha256: sha256(gcode), plateCoverage, fitCoverage }, null, 2)}\n`,
  );
  expect(errors).toEqual([]);
});
