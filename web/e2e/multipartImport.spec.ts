import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test as diagnostics } from "./failureDiagnostics";
import { choose, PRESETS } from "./lhPresets";

const test = diagnostics.extend<{ servedBodies: void }>({
  servedBodies: [async ({ context }, use, info) => {
    const captures: Promise<unknown>[] = [];
    context.on("response", (response) => {
      if (!/\.(js|wasm)(\?|$)/.test(response.url())) return;
      const index = captures.length;
      captures.push((async () => {
        const bytes = await response.body();
        const path = `served-${index}.${response.url().includes(".wasm") ? "wasm" : "js"}`;
        writeFileSync(info.outputPath(path), bytes);
        return { url: response.url(), status: response.status(), path, bytes: bytes.length, sha256: sha256(bytes) };
      })());
    });
    await use();
    const responses = await Promise.allSettled(captures);
    writeFileSync(info.outputPath("served-bodies.json"), `${JSON.stringify(responses, null, 2)}\n`);
    expect(responses.filter((response) => response.status === "rejected")).toEqual([]);
  }, { auto: true }],
});

const cube = readFileSync(fileURLToPath(new URL("../../tests/parity/cube10.stl", import.meta.url)));
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function part(name: string, scale: readonly number[], offset: readonly number[]) {
  const bytes = Buffer.from(cube);
  for (let facet = 84; facet < bytes.length; facet += 50) {
    for (let vertex = facet + 12; vertex < facet + 48; vertex += 12) {
      for (let axis = 0; axis < 3; axis++) {
        const position = vertex + axis * 4;
        bytes.writeFloatLE(bytes.readFloatLE(position) * scale[axis] + offset[axis], position);
      }
    }
  }
  // The template has inward winding; author outward faces before either loader reads them.
  for (let facet = 84; facet < bytes.length; facet += 50) {
    const first = Buffer.from(bytes.subarray(facet + 12, facet + 24));
    bytes.copy(bytes, facet + 12, facet + 24, facet + 36);
    first.copy(bytes, facet + 24);
    const points = [12, 24, 36].map((offset) => [0, 4, 8].map((axis) => bytes.readFloatLE(facet + offset + axis)));
    const a = points[1].map((value, axis) => value - points[0][axis]);
    const b = points[2].map((value, axis) => value - points[0][axis]);
    const normal = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    const length = Math.hypot(...normal);
    normal.forEach((value, axis) => bytes.writeFloatLE(value / length, facet + axis * 4));
  }
  return { name, bytes };
}

async function openWithPresets(page: Page) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "System presets", exact: true });
  await wizard.getByRole("checkbox", { name: PRESETS.vendor, exact: true }).check();
  await wizard.getByRole("button", { name: "Done", exact: true }).click();
  await choose(page, "mouse", "Printer preset", "System presets", PRESETS.printer);
  await choose(page, "mouse", "Process preset", "System presets", PRESETS.process);
  await choose(page, "mouse", "Filament 1 preset", "System presets", PRESETS.filament);
}

async function drop(page: Page, files: { name: string; bytes: Buffer }[]) {
  const transfer = await page.evaluateHandle((inputs) => {
    const data = new DataTransfer();
    for (const { name, base64 } of inputs) {
      data.items.add(new File([Uint8Array.from(atob(base64), (character) => character.charCodeAt(0))], name));
    }
    return data;
  }, files.map(({ name, bytes }) => ({ name, base64: bytes.toString("base64") })));
  const workspace = page.getByTestId("workspace");
  await workspace.dispatchEvent("dragenter", { dataTransfer: transfer });
  await workspace.dispatchEvent("dragover", { dataTransfer: transfer });
  await expect(page.getByTestId("drop-overlay")).toBeVisible();
  await workspace.dispatchEvent("drop", { dataTransfer: transfer });
  await transfer.dispose();
  await expect(page.getByTestId("drop-overlay")).toHaveCount(0);
}

test("dropping two STLs loads one object with both parts and slices it", async ({ page }, info) => {
  const files = [part("base.stl", [0.8, 0.3, 0.2], [0, 0, 0]), part("upper.stl", [0.3, 0.8, 0.2], [12, 4, 2])];
  const inputs = files.map(({ name, bytes }) => {
    writeFileSync(info.outputPath(name), bytes);
    return { name, bytes: bytes.length, sha256: sha256(bytes) };
  });
  writeFileSync(info.outputPath("inputs.json"), `${JSON.stringify(inputs, null, 2)}\n`);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await openWithPresets(page);
  await drop(page, files);
  writeFileSync(info.outputPath("import-status.txt"), await page.getByTestId("status").innerText());
  const dialog = page.getByRole("dialog", { name: "An object with multiple parts was detected", exact: true });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("button", { name: "Close", exact: true })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(dialog.getByRole("button", { name: "No", exact: true })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(dialog.getByRole("button", { name: "Close", exact: true })).toBeFocused();
  await expect(dialog.getByText("Load these files as a single object with multiple parts?", { exact: true })).toBeVisible();
  await expect(dialog.getByRole("checkbox", { name: "Auto-Drop", exact: true })).toBeChecked();
  await page.screenshot({ path: info.outputPath("question.png") });
  await page.keyboard.press("Tab");
  await expect(dialog.getByRole("checkbox", { name: "Auto-Drop", exact: true })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(dialog.getByRole("button", { name: "Yes", exact: true })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect(page.getByTestId("object-count")).toHaveText("1");
  const list = page.getByTestId("object-list");
  await list.getByRole("button", { name: "Toggle parts", exact: true }).click();
  const parts = list.locator(".object-part");
  await expect(parts).toHaveCount(2);
  await expect(parts.filter({ hasText: "base.stl" })).toHaveCount(1);
  await expect(parts.filter({ hasText: "upper.stl" })).toHaveCount(1);
  await page.screenshot({ path: info.outputPath("parts.png") });
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  const gcodePath = info.outputPath("multipart.gcode");
  await download.saveAs(gcodePath);
  const gcode = readFileSync(gcodePath);
  expect(gcode.length).toBeGreaterThan(0);
  expect(gcode.toString()).toContain("\nG1 ");
  const objects = await list.getByRole("treeitem").allTextContents();
  writeFileSync(info.outputPath("result.json"), `${JSON.stringify({ inputs, objects, gcode: { bytes: gcode.length, sha256: sha256(gcode) }, errors }, null, 2)}\n`);
  await page.screenshot({ path: info.outputPath("sliced.png") });
  expect(errors).toEqual([]);
});

for (const autoDrop of [true, false]) {
  test(`picker loads separate objects with Auto-Drop ${autoDrop ? "on" : "off"}`, async ({ page }, info) => {
    // Off keeps Z: use grounded solids of different heights, not unsupported floating objects.
    const files = autoDrop
      ? [part("base.stl", [0.8, 0.3, 0.2], [0, 0, 0]), part("upper.stl", [0.3, 0.8, 0.2], [12, 4, 2])]
      : [part("base.stl", [0.8, 0.3, 0.2], [0, 0, 1]), part("upper.stl", [0.3, 0.8, 1], [12, 4, 5])];
    const inputs = files.map(({ name, bytes }) => {
      writeFileSync(info.outputPath(name), bytes);
      return { name, bytes: bytes.length, sha256: sha256(bytes) };
    });
    writeFileSync(info.outputPath("inputs.json"), `${JSON.stringify({ inputs, autoDrop }, null, 2)}\n`);
    await openWithPresets(page);
    await page.getByTestId("import-input").setInputFiles(files.map(({ name, bytes: buffer }) => ({ name, buffer, mimeType: "model/stl" })));
    const dialog = page.getByRole("dialog", { name: "An object with multiple parts was detected", exact: true });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole("checkbox", { name: "Auto-Drop", exact: true })).toBeChecked();
    if (!autoDrop) await dialog.getByRole("checkbox", { name: "Auto-Drop", exact: true }).uncheck();
    await dialog.getByRole("button", { name: "No", exact: true }).click();
    await expect(page.getByTestId("object-count")).toHaveText("2");
    await expect(page.getByTestId("status")).toHaveText("Ready to slice");
    await page.screenshot({ path: info.outputPath("separate.png") });
    await page.getByRole("button", { name: "Slice plate", exact: true }).click();
    await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
    const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code file", exact: true }).click()]);
    const gcodePath = info.outputPath("separate.gcode");
    await download.saveAs(gcodePath);
    expect(readFileSync(gcodePath, "utf8")).toContain("\nG1 ");
  });
}

test("batch replacement, cancellation and malformed input preserve the right project", async ({ page }, info) => {
  const files = [part("base.stl", [0.8, 0.3, 0.2], [0, 0, 0]), part("upper.stl", [0.3, 0.8, 0.2], [12, 4, 2])];
  await openWithPresets(page);
  await page.getByTestId("import-input").setInputFiles({ name: "old.stl", buffer: cube, mimeType: "model/stl" });
  await expect(page.getByTestId("plater-toolbar")).toBeVisible();
  await expect(page.getByTestId("object-count")).toHaveText("1");
  const add = page.getByTestId("plater-toolbar").getByRole("button", { name: "Add", exact: true });
  const [picker] = await Promise.all([page.waitForEvent("filechooser"), add.click()]);
  await picker.setFiles(files.map(({ name, bytes: buffer }) => ({ name, buffer, mimeType: "model/stl" })));
  const dialog = page.getByRole("dialog", { name: "An object with multiple parts was detected", exact: true });
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(add).toBeFocused();
  await expect(page.getByTestId("object-list")).toContainText("old.stl");
  await drop(page, files);
  await expect(dialog.getByRole("checkbox", { name: "Auto-Drop", exact: true })).toBeChecked();
  await dialog.getByRole("button", { name: "Yes", exact: true }).click();
  await expect(page.getByTestId("object-count")).toHaveText("1");
  await expect(page.getByTestId("object-list")).not.toContainText("old.stl");
  await drop(page, [files[0], { name: "broken.stl", bytes: Buffer.from("not an STL") }]);
  await dialog.getByRole("button", { name: "Yes", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Import failed");
  await expect(page.getByTestId("object-count")).toHaveText("1");
  await drop(page, [files[0], { name: "other.3mf", bytes: Buffer.from("not a project") }]);
  await expect(page.getByTestId("status")).toContainText("Import failed");
  await expect(dialog).toHaveCount(0);
  await page.getByTestId("object-list").getByRole("button", { name: "Toggle parts", exact: true }).click();
  await expect(page.getByTestId("object-list").locator(".object-part")).toHaveCount(2);
  await page.screenshot({ path: info.outputPath("preserved.png") });
});

test("single STL and 3MF imports keep their single-file entry", async ({ page }, info) => {
  await openWithPresets(page);
  await page.getByTestId("import-input").setInputFiles({ name: "single.stl", buffer: cube, mimeType: "model/stl" });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect(page.getByTestId("object-count")).toHaveText("1");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page.screenshot({ path: info.outputPath("single-stl.png") });
  const project = readFileSync(fileURLToPath(new URL("../../tests/parity/arachne/default-prisms.3mf", import.meta.url)));
  await page.getByTestId("import-input").setInputFiles({ name: "prisms.3mf", buffer: project, mimeType: "model/3mf" });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect(page.getByTestId("object-count")).toHaveText("3");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  writeFileSync(info.outputPath("inputs.json"), `${JSON.stringify({ stl: sha256(cube), threeMf: sha256(project) }, null, 2)}\n`);
  await page.screenshot({ path: info.outputPath("single-3mf.png") });
});
