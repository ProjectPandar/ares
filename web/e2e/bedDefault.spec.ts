import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { ORCA_SLICER_COMMIT } from "@ares-slicer/profiles";
import { test } from "./failureDiagnostics";
import { bedCommands, firstBedCommands, nativeBedReference } from "./bedDefaultOracle";
import { oraclePath } from "./oracleEnv";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const model = `${repository}tests/parity/cube10.stl`;
const artifacts = `${repository}target/web-e2e/bed-default/`;

test("U1 initializes Textured PEI and exports Native first-layer bed temperatures without a bed selection", async ({ page }) => {
  const binary = oraclePath("ARES_ORCA_BIN", "the release bed-temperature oracle", "the pinned OrcaSlicer 2.5.0 launcher");
  if (!binary) test.skip();
  test.setTimeout(300_000);
  mkdirSync(artifacts, { recursive: true });
  expect(ORCA_SLICER_COMMIT).toBe("40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19");
  const combo = (name: string) => page.getByRole("button", { name, exact: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(model);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await combo("Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "System presets" });
  await wizard.getByRole("checkbox", { name: "Snapmaker", exact: true }).check();
  await wizard.getByRole("button", { name: "Done" }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await combo("Printer preset").click();
  await page.getByRole("option", { name: "Snapmaker U1", exact: true }).click();
  await expect(combo("Printer preset")).toHaveAttribute("title", "Snapmaker U1 (0.4 nozzle)");
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const plate = page.getByRole("combobox", { name: "Plate type", exact: true });
  await expect(plate).toBeVisible();
  const bed = await plate.inputValue();
  writeFileSync(`${artifacts}initial-selection.json`, JSON.stringify({ bed, printer: await combo("Printer preset").getAttribute("title"), explicit_bed_selection: false }, null, 2));
  await page.screenshot({ path: `${artifacts}implicit-selection.png` });
  expect(bed).toBe("Textured PEI Plate");
  await combo("Process preset").click();
  await page.getByRole("option", { name: "0.20 Standard @Snapmaker U1 (0.4 nozzle)", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await combo("Filament 1 preset").click();
  await page.getByRole("option", { name: "Add/Remove filaments", exact: true }).click();
  const filaments = page.getByRole("dialog", { name: "Filament selection" });
  await filaments.getByRole("group", { name: "Filaments", exact: true }).getByRole("checkbox", { name: "Generic PLA", exact: true }).check();
  await filaments.getByRole("button", { name: "Done", exact: true }).click();
  const slots = page.getByRole("button", { name: /^Filament \d+ preset$/ });
  for (let index = 0; index < await slots.count(); index++) {
    await slots.nth(index).click();
    await page.getByRole("option", { name: "Generic PLA @U1 0.4 nozzle", exact: true }).click();
  }
  const selection = {
    printer: (await combo("Printer preset").getAttribute("title"))!,
    process: (await combo("Process preset").getAttribute("title"))!,
    filaments: await page.getByRole("button", { name: /^Filament \d+ preset$/ }).evaluateAll((buttons) => buttons.map((button) => button.getAttribute("title")!)),
  };
  expect(selection.filaments.length).toBeGreaterThan(0);
  const native = await nativeBedReference(binary!, selection, model, `${artifacts}native`);
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code" }).click()]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const browser = readFileSync(`${artifacts}browser.gcode`, "utf8");
  writeFileSync(`${artifacts}result.json`, JSON.stringify({ selection, bed, explicit_bed_selection: false, native_first_bed_commands: firstBedCommands(native), browser_first_bed_commands: firstBedCommands(browser), native_bed_commands: bedCommands(native), browser_bed_commands: bedCommands(browser), complete_output_parity_claimed: false }, null, 2));
  expect(firstBedCommands(native)).toEqual(["M140 S55", "M190 S55"]);
  expect(firstBedCommands(browser)).toEqual(firstBedCommands(native));
  expect(browser).toContain("; curr_bed_type = Textured PEI Plate");
});
