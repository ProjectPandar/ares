import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test as diagnostics } from "./failureDiagnostics";

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
  return { name, bytes };
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
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await drop(page, files);
  writeFileSync(info.outputPath("import-status.txt"), await page.getByTestId("status").innerText());
  const dialog = page.getByRole("dialog", { name: "An object with multiple parts was detected", exact: true });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText("Load these files as a single object with multiple parts?", { exact: true })).toBeVisible();
  await expect(dialog.getByRole("checkbox", { name: "Auto-Drop", exact: true })).toBeChecked();
  await page.screenshot({ path: info.outputPath("question.png") });
  await dialog.getByRole("button", { name: "Yes", exact: true }).click();
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
  expect(gcode.toString()).toContain("G1");
  const objects = await list.getByRole("treeitem").allTextContents();
  writeFileSync(info.outputPath("result.json"), `${JSON.stringify({ inputs, objects, gcode: { bytes: gcode.length, sha256: sha256(gcode) }, errors }, null, 2)}\n`);
  await page.screenshot({ path: info.outputPath("sliced.png") });
  expect(errors).toEqual([]);
});

for (const autoDrop of [true, false]) {
  test(`picker loads separate objects with Auto-Drop ${autoDrop ? "on" : "off"}`, async ({ page }, info) => {
    const files = [part("base.stl", [0.8, 0.3, 0.2], [0, 0, 3]), part("upper.stl", [0.3, 0.8, 0.2], [12, 4, 7])];
    await page.goto("/");
    await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
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
    await download.saveAs(info.outputPath("separate.gcode"));
  });
}

test("batch replacement, cancellation and malformed input preserve the right project", async ({ page }, info) => {
  const files = [part("base.stl", [0.8, 0.3, 0.2], [0, 0, 0]), part("upper.stl", [0.3, 0.8, 0.2], [12, 4, 2])];
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles({ name: "old.stl", buffer: cube, mimeType: "model/stl" });
  await expect(page.getByTestId("object-count")).toHaveText("1");
  const picker = page.getByTestId("plater-toolbar").locator('input[type="file"]');
  await picker.setInputFiles(files.map(({ name, bytes: buffer }) => ({ name, buffer, mimeType: "model/stl" })));
  const dialog = page.getByRole("dialog", { name: "An object with multiple parts was detected", exact: true });
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
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
