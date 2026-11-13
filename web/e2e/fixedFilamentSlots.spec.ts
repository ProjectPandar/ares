import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

const root = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = process.env.ARES_FIXED_FILAMENT_ARTIFACT_ROOT ?? `${root}target/web-e2e/fixed-filament-witness`;

test("U1 has four fixed filament slots and edits them directly", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(`${root}tests/parity/cube10.stl`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  const printer = page.getByRole("button", { name: "Printer preset", exact: true });
  await printer.click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "System presets" });
  await wizard.getByRole("checkbox", { name: "Snapmaker", exact: true }).check();
  await wizard.getByRole("checkbox", { name: "Bambulab", exact: true }).check();
  await wizard.getByRole("button", { name: "Done" }).click();
  await printer.click();
  await page.getByRole("option", { name: "Bambu Lab X1 Carbon", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect(page.locator(".filament-row")).toHaveCount(1);
  await page.screenshot({ path: `${artifacts}/bbl-fresh-one-slot.png` });
  // A model item takes the current printer's variant (PresetBundle::get_similar_printer_preset,
  // PresetBundle.cpp:5322-5354). The U1 filaments of OrcaSlicer 2.4.2 fit only its 0.4 nozzle.
  await page.getByRole("combobox", { name: "Nozzle diameter", exact: true }).selectOption("0.4");
  await expect(printer).toHaveAttribute("title", "Bambu Lab X1 Carbon 0.4 nozzle", { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await printer.click();
  await page.getByRole("option", { name: "Snapmaker U1", exact: true }).click();
  await expect(printer).toHaveAttribute("title", "Snapmaker U1 (0.4 nozzle)", { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  // The 2.4.2 U1 model lists no default_materials, so no U1 filament is installed with the printer
  // (PresetBundle::load_installed_filaments, PresetBundle.cpp:3757-3812): install one, then select
  // the U1 again so its remembered slots take it.
  await page.getByRole("button", { name: "Filament 1 preset", exact: true }).click();
  await page.getByRole("option", { name: "Add/Remove filaments" }).click();
  const install = page.getByRole("dialog", { name: "Filament selection" });
  await install.getByRole("group", { name: "Filaments" }).getByRole("checkbox", { name: "Snapmaker PLA", exact: true }).check({ timeout: 60_000 });
  await install.getByRole("button", { name: "Done" }).click();
  await printer.click();
  await page.getByRole("option", { name: "Snapmaker U1", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect(page.getByRole("button", { name: "Filament 4 preset", exact: true })).toHaveText(/\S/);
  await expect(page.locator(".filament-row")).toHaveCount(4);
  await expect(page.getByRole("button", { name: /^(Add filament|Delete filament|Flushing volumes)$/ })).toHaveCount(0);
  const filaments = await page.getByRole("button", { name: /^Filament \d+ preset$/ }).allTextContents();
  await page.screenshot({ path: `${artifacts}/u1-four-slots.png` });
  await page.getByRole("button", { name: "Edit filament 4 settings", exact: true }).click();
  const editor = page.getByRole("dialog", { name: "Filament settings", exact: true });
  await expect(editor).toBeVisible();
  await editor.getByRole("slider", { name: "Settings mode", exact: true }).focus();
  await page.keyboard.press("End");
  const density = editor.getByLabel("filament_density", { exact: true });
  await expect(density).toBeVisible();
  await expect(density).toHaveAttribute("readonly", "");
  const originalDensity = Number(await density.inputValue());
  await page.screenshot({ path: `${artifacts}/u1-edit.png` });
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Process preset", exact: true }).click();
  await page.getByRole("listbox", { name: "Process preset", exact: true }).getByRole("option").first().click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 60_000 });
  await page.screenshot({ path: `${artifacts}/u1-slice.png` });
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export G-code file", exact: true }).click();
  await (await download).saveAs(`${artifacts}/u1.gcode`);
  const gcode = readFileSync(`${artifacts}/u1.gcode`, "utf8");
  const densities = gcode.match(/^; filament_density = (.+)$/m)![1].split(",").map(Number);
  expect(densities.every((value) => value === originalDensity)).toBe(true);
  const ids = gcode.match(/^; filament_settings_id = (.+)$/m)![1].split(";").map((value) => JSON.parse(value) as string);
  expect(ids).toEqual(filaments);
  for (const [key, length] of [["flush_multiplier", 4], ["flush_volumes_vector", 8], ["flush_volumes_matrix", 64]] as const) {
    expect(gcode.match(new RegExp(`^; ${key} = (.+)$`, "m"))![1].split(",")).toHaveLength(length);
  }
  await page.getByRole("link", { name: "Prepare", exact: true }).click();
  await printer.click();
  await page.getByRole("option", { name: "Bambu Lab X1 Carbon", exact: true }).click();
  // remember_printer_config, on by default (AppConfig.cpp:544-545, Tab.cpp:7740-7742): the X1 Carbon
  // gets back its own one-slot selection (PresetBundle::update_selections, PresetBundle.cpp:3914-3940),
  // which already meets its minimum; the U1's four slots are not carried over.
  await expect(printer).toHaveAttribute("title", "Bambu Lab X1 Carbon 0.4 nozzle", { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect(page.locator(".filament-row")).toHaveCount(1);
  await page.screenshot({ path: `${artifacts}/bbl-remembered-slots.png` });
  // The reload reads the stored selection, so it waits for the switch to store it.
  await expect.poll(() => page.evaluate(() => JSON.parse(localStorage.getItem("ares.config")!).printer)).toBe("Bambu Lab X1 Carbon 0.4 nozzle");
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await expect(printer).toHaveAttribute("title", "Bambu Lab X1 Carbon 0.4 nozzle", { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect(page.locator(".filament-row")).toHaveCount(1);
  expect(errors).toEqual([]);
  writeFileSync(`${artifacts}/browser-result.json`, JSON.stringify({ printer: "Snapmaker U1 (0.4 nozzle)", slots: 4, filaments, sliced: true, densities, bblFreshSlots: 1, bblRememberedSlots: 1, restoredSlots: 1, errors }, null, 2));
});
