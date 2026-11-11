import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { FIRST_RUN_CONFIG, test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;
const artifacts = fileURLToPath(new URL("../../target/web-e2e/wizard-entry-points/", import.meta.url));
const MODEL = "Bambu Lab A1 mini";
const FILAMENT = "Generic PLA Silk";

/** A1 mini's nozzle variants, in `nozzle_diameter` order (`VendorProfile::PrinterModel`). */
function a1miniVariants(): string[] {
  const index = JSON.parse(readFileSync(`${profiles}BBL.json`, "utf8")) as { machine_model_list: { name: string; sub_path: string }[] };
  const entry = index.machine_model_list.find(({ name }) => name === MODEL)!;
  return (JSON.parse(readFileSync(`${profiles}BBL/${entry.sub_path}`, "utf8")) as { nozzle_diameter: string }).nozzle_diameter.split(";");
}

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const storedConfig = (page: Page) =>
  page.evaluate(() => JSON.parse(localStorage.getItem("ares.config")!) as { installedPrinters: unknown; installedFilaments: string[] });

async function settled(page: Page) {
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

/** The models of the printer combo's "System presets" group; Escape closes the list. */
async function systemModels(page: Page) {
  await combo(page, "Printer preset").click();
  const names = await page.getByRole("listbox", { name: "Printer preset", exact: true })
    .getByRole("group", { name: "System presets", exact: true }).getByRole("option").allTextContents();
  await page.keyboard.press("Escape");
  return names.sort();
}

/** Confirm on the models page closes the guide once the installed printers are applied. */
async function confirm(page: Page, wizard: ReturnType<Page["getByRole"]>) {
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await settled(page);
}

test("the printer title, the printer combo and the filament title open the guide with the current selection checked", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "MyKlipper 0.4 nozzle", { timeout: 60_000 });
  await settled(page);
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });

  // The printer title's settings button (Plater.cpp:2851-2857) opens the models page with the
  // installed model checked; adding A1 mini installs exactly its presets and selects it.
  await page.getByRole("button", { name: "Select/Remove printers (system presets)", exact: true }).click();
  await expect(wizard.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true })).toBeChecked({ timeout: 30_000 });
  expect(await wizard.getByRole("checkbox", { checked: true }).evaluateAll((boxes) => boxes.map((box) => box.getAttribute("aria-label"))))
    .toEqual(["Generic Klipper Printer"]);
  await wizard.getByRole("searchbox", { name: "Search printers" }).fill("a1 mini");
  await wizard.getByRole("checkbox", { name: MODEL, exact: true }).check();
  await page.screenshot({ path: `${artifacts}title-button.png` });
  await confirm(page, wizard);
  const variants = a1miniVariants();
  expect((await storedConfig(page)).installedPrinters).toEqual({ ...FIRST_RUN_CONFIG.installedPrinters, BBL: { [MODEL]: [...variants].sort() } });
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", `${MODEL} ${variants[0]} nozzle`);
  expect(await systemModels(page)).toEqual([MODEL, "Generic Klipper Printer"]);

  // The combo's "Select/Remove printers (system presets)" item (PresetComboBoxes.cpp:1026-1037) opens
  // the same page with both models checked; unchecking A1 mini removes its presets, and the
  // selection falls to the first visible printer by name (Preset.cpp:4239-4248).
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  await expect(wizard.getByRole("checkbox", { name: MODEL, exact: true })).toBeChecked({ timeout: 30_000 });
  await expect(wizard.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true })).toBeChecked();
  await wizard.getByRole("checkbox", { name: MODEL, exact: true }).uncheck();
  await page.screenshot({ path: `${artifacts}combo-item.png` });
  await confirm(page, wizard);
  expect((await storedConfig(page)).installedPrinters).toEqual(FIRST_RUN_CONFIG.installedPrinters);
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "MyKlipper 0.2 nozzle");
  expect(await systemModels(page)).toEqual(["Generic Klipper Printer"]);

  // The filament title's "Set filaments to use" (Plater.cpp:3379-3387) opens the guide/23 page with
  // the installed filaments checked; checking one more installs its presets and lists it in the combo.
  // The installed set also holds A1 mini's default materials, which Confirm added (save_userguide_models).
  const before = (await storedConfig(page)).installedFilaments;
  const short = (name: string) => name.split("@")[0].trim();
  await page.getByRole("button", { name: "Set filaments to use", exact: true }).click();
  const filaments = page.getByRole("dialog", { name: "Filament selection" });
  const list = filaments.getByRole("group", { name: "Filaments" });
  await expect(list.getByRole("checkbox").first()).toBeVisible({ timeout: 60_000 });
  await filaments.getByRole("searchbox", { name: "Filter filaments" }).fill("::checked");
  const checked = await list.locator("label").allTextContents();
  expect(checked.filter((name) => !before.some((installed) => short(installed) === name))).toEqual([]);
  expect(checked).toEqual(expect.arrayContaining(FIRST_RUN_CONFIG.installedFilaments.map(short)));
  await filaments.getByRole("searchbox", { name: "Filter filaments" }).fill(FILAMENT.toLowerCase());
  const item = list.getByRole("checkbox", { name: FILAMENT, exact: true });
  await expect(item).not.toBeChecked();
  await item.check();
  await page.screenshot({ path: `${artifacts}filament-title-button.png` });
  await filaments.getByRole("button", { name: "Done" }).click();
  await expect(filaments).toHaveCount(0);
  const installed = (await storedConfig(page)).installedFilaments;
  const added = installed.filter((name) => !before.includes(name));
  expect(added.map(short)).toEqual(added.map(() => FILAMENT));
  expect(added.length).toBeGreaterThan(0);
  await combo(page, "Filament 1 preset").click();
  await page.getByRole("searchbox", { name: "Search Filament 1 preset" }).fill("silk");
  const listed = await page.getByRole("listbox", { name: "Filament 1 preset", exact: true })
    .getByRole("group", { name: "System presets", exact: true }).getByRole("option").allTextContents();
  expect(listed).toEqual(added.filter((name) => listed.includes(name)));
  expect(listed.length).toBeGreaterThan(0);
  await page.getByRole("option", { name: listed[0], exact: true }).click();
  await expect(combo(page, "Filament 1 preset")).toHaveText(listed[0]);
  await page.screenshot({ path: `${artifacts}filament-selected.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ variants, added, listed }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
