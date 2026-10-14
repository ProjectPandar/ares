import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { canvasDigest } from "./toolpathPreview";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/shortcuts/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;

const blur = (page: Page) => page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
const pressed = (page: Page, view: string) =>
  expect(page.getByTestId("view-toolbar").getByRole("button", { name: view, exact: true }))
    .toHaveAttribute("aria-pressed", "true");

/** Preview shortcuts (KBShortcutsDialog "Preview"): layer slider keys, wheel, one-layer mode. */
async function checkSliderShortcuts(page: Page) {
  const top = page.getByRole("slider", { name: "Top layer" });
  const bottom = page.getByRole("slider", { name: "Bottom layer" });
  await expect(top).toHaveAttribute("aria-valuenow", "50");
  await blur(page);

  // C: G-code window on/off (shown once there is a current move).
  await expect(page.getByTestId("gcode-panel")).toBeVisible();
  await page.keyboard.press("c");
  await expect(page.getByTestId("gcode-panel")).toHaveCount(0);
  await page.keyboard.press("c");
  await expect(page.getByTestId("gcode-panel")).toBeVisible();

  // GLCanvas3D::on_key: Up/Down move the active (upper) thumb, x5 with Shift or Ctrl.
  await page.keyboard.press("ArrowDown");
  await expect(top).toHaveAttribute("aria-valuenow", "49");
  await page.keyboard.press("Shift+ArrowDown");
  await expect(top).toHaveAttribute("aria-valuenow", "44");
  await page.keyboard.press("Control+ArrowUp");
  await expect(top).toHaveAttribute("aria-valuenow", "49");

  // IMSlider::on_mouse_wheel over the slider: one layer per notch, x5 with Shift.
  const box = (await page.getByTestId("layer-slider").boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, 100);
  await expect(top).toHaveAttribute("aria-valuenow", "48");
  await page.keyboard.down("Shift");
  await page.mouse.wheel(0, 100);
  await page.keyboard.up("Shift");
  await expect(top).toHaveAttribute("aria-valuenow", "43");

  // IMSlider::switch_one_layer_mode: keeps a moved upper thumb, restores the backup at the top.
  await page.keyboard.press("l");
  await expect(top).toHaveAttribute("aria-valuenow", "43");
  await expect(bottom).toHaveCount(0);
  const oneLayer = await canvasDigest(page);
  await page.screenshot({ path: `${artifacts}one-layer.png` });
  await page.keyboard.press("ArrowUp");
  await expect(top).toHaveAttribute("aria-valuenow", "44");
  expect(await canvasDigest(page), "one-layer mode must follow the thumb").not.toBe(oneLayer);
  await page.keyboard.press("l");
  await expect(top).toHaveAttribute("aria-valuenow", "50");
  await expect(bottom).toHaveAttribute("aria-valuenow", "1");
  await page.keyboard.press("l");
  await expect(top).toHaveAttribute("aria-valuenow", "44");
  await page.keyboard.press("l");

  // The focused lower thumb becomes the active one for canvas arrows.
  await bottom.focus();
  await page.keyboard.press("ArrowUp");
  await expect(bottom).toHaveAttribute("aria-valuenow", "2");
  await blur(page);
  await page.keyboard.press("ArrowUp");
  await expect(bottom).toHaveAttribute("aria-valuenow", "3");
  await expect(top).toHaveAttribute("aria-valuenow", "50");
}

test("OrcaSlicer keyboard shortcuts drive import, camera, slicing, export and preview", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });

  // "?" opens the shortcut list (Plater::keyboard_shortcuts).
  await page.keyboard.press("?");
  const dialog = page.getByRole("dialog", { name: "Keyboard Shortcuts" });
  await expect(dialog).toBeVisible();
  for (const group of ["Global shortcuts", "Plater", "Preview"]) {
    await expect(dialog.getByRole("heading", { name: group })).toBeVisible();
  }
  await expect(dialog).toContainText("Slice plate");
  await page.screenshot({ path: `${artifacts}dialog.png` });
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();

  // Ctrl+I: import (MainFrame import menu accelerator).
  const [chooser] = await Promise.all([page.waitForEvent("filechooser"), page.keyboard.press("ControlOrMeta+i")]);
  await chooser.setFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  // GLCanvas3D::on_char: Ctrl+0-6 select camera views; bare digits are filament keys, not views.
  await page.getByRole("button", { name: "Iso", exact: true }).click();
  await blur(page);
  await page.keyboard.press("1");
  await pressed(page, "Iso");
  await page.keyboard.press("ControlOrMeta+1");
  await pressed(page, "Top");
  await page.keyboard.press("ControlOrMeta+6");
  await pressed(page, "Right");
  await page.keyboard.press("ControlOrMeta+0");
  await expect(page.getByTestId("view-toolbar").locator("[aria-pressed=true]")).toHaveCount(0);

  // I / O: camera zoom in / out.
  const framed = await canvasDigest(page);
  await page.keyboard.press("i");
  const zoomed = await canvasDigest(page);
  expect(zoomed, "I must zoom in").not.toBe(framed);
  await page.keyboard.press("o");
  expect(await canvasDigest(page), "O must zoom out").not.toBe(zoomed);

  // Esc: deselect all.
  const object = page.getByTestId("object-list").getByRole("treeitem").first();
  await object.click();
  await expect(object).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("Escape");
  await expect(object).toHaveAttribute("aria-selected", "false");

  // Tab: switch between Prepare and Preview.
  await blur(page);
  await page.keyboard.press("Tab");
  await expect(page).toHaveURL(/#\/preview$/);
  await page.keyboard.press("Tab");
  await expect(page).toHaveURL(/#\/prepare$/);

  // Ctrl+R: slice plate and show Preview (MainFrame key handler).
  await page.keyboard.press("ControlOrMeta+r");
  await expect(page).toHaveURL(/#\/preview$/);
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });

  // Ctrl+G: export plate sliced file.
  const [download] = await Promise.all([page.waitForEvent("download"), page.keyboard.press("ControlOrMeta+g")]);
  expect(download.suggestedFilename()).toBe("orca_cli_ender3_plate_1.gcode");
  const gcodePath = `${artifacts}${download.suggestedFilename()}`;
  await download.saveAs(gcodePath);

  await checkSliderShortcuts(page);

  await page.screenshot({ path: `${artifacts}preview.png` });
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ gcode: download.suggestedFilename(), slider: true }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
