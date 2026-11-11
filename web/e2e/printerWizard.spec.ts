import { mkdirSync, readFileSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { FIRST_RUN_CONFIG, test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;
const artifacts = fileURLToPath(new URL("../../target/web-e2e/printer-wizard/", import.meta.url));
const MODEL = `${repositoryRoot}tests/parity/cube10.stl`;

test.use({ firstRunConfig: false });

/** A vendor model's nozzle variants, in `nozzle_diameter` order (`VendorProfile::PrinterModel`). */
function modelVariants(vendor: string, model: string): string[] {
  const index = JSON.parse(readFileSync(`${profiles}${vendor}.json`, "utf8")) as { machine_model_list: { name: string; sub_path: string }[] };
  const entry = index.machine_model_list.find(({ name }) => name === model)!;
  const file = JSON.parse(readFileSync(`${profiles}${vendor}/${entry.sub_path}`, "utf8")) as { nozzle_diameter: string };
  return file.nozzle_diameter.split(";");
}

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const storedConfig = (page: Page) => page.evaluate(() => JSON.parse(localStorage.getItem("ares.config") ?? "null") as unknown);

async function settled(page: Page) {
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

/** The models of the printer combo's "System presets" group; Escape closes the list. */
async function systemModels(page: Page) {
  await combo(page, "Printer preset").click();
  const names = await page.getByRole("listbox", { name: "Printer preset", exact: true })
    .getByRole("group", { name: "System presets", exact: true }).getByRole("option").allTextContents();
  await page.keyboard.press("Escape");
  return names;
}

async function openModelsPage(page: Page) {
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await expect(wizard.getByRole("checkbox").first()).toBeVisible({ timeout: 30_000 });
  return wizard;
}

test("the setup wizard installs printer models with every nozzle and their default filaments", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  // 1. A profile without an app config opens the wizard, which cannot be closed.
  await page.goto("/");
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  const klipper = wizard.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true });
  await expect(klipper).toBeVisible({ timeout: 30_000 });
  await expect(wizard.getByRole("button", { name: "Close" })).toHaveCount(0);
  await page.keyboard.press("Escape");
  await page.keyboard.press("Escape");
  await expect(wizard).toBeVisible();
  await expect(wizard.getByRole("region", { name: "Custom Printer" })).toBeVisible();
  await page.screenshot({ path: `${artifacts}first-run.png` });

  // 4a. Next with no model checked stays on the printer page.
  await wizard.getByRole("button", { name: "Next" }).click();
  await expect(wizard.getByRole("heading", { name: "Printer Selection" })).toBeVisible();

  // 2. Generic Klipper Printer, then the filament page with its default materials checked.
  await klipper.check();
  await expect(wizard.getByRole("region", { name: "Custom Printer" })).toContainText(/1 \/ \d+/);
  await wizard.getByRole("button", { name: "Next" }).click();
  const filaments = page.getByRole("dialog", { name: "Filament Selection" });
  await expect(filaments.getByRole("group", { name: "Filaments" })).toBeVisible({ timeout: 60_000 });
  await page.screenshot({ path: `${artifacts}filaments.png` });

  // 4b. Finish with no filament checked shows Orca's notice and stays open.
  // From the last: each locator is an index into the checked boxes.
  for (const box of (await filaments.getByRole("checkbox", { checked: true }).all()).reverse()) await box.uncheck();
  await filaments.getByRole("button", { name: "Finish" }).click();
  const notice = filaments.getByRole("alertdialog", { name: "Error" });
  await expect(notice).toContainText("At least one filament must be selected.");
  await page.screenshot({ path: `${artifacts}no-filament.png` });
  await notice.getByRole("button", { name: "OK" }).click();
  await filaments.getByRole("button", { name: "Back" }).click();
  await wizard.getByRole("button", { name: "Next" }).click();
  await filaments.getByRole("button", { name: "Finish" }).click();
  await expect(filaments).toHaveCount(0, { timeout: 60_000 });
  await settled(page);

  // The stored result is the first-run app config every other spec starts from.
  expect(await storedConfig(page)).toEqual(FIRST_RUN_CONFIG);
  const klipperVariants = modelVariants("Custom", "Generic Klipper Printer");
  expect(await systemModels(page)).toEqual(["Generic Klipper Printer"]);
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "MyKlipper 0.4 nozzle");
  const nozzles = page.getByRole("combobox", { name: "Nozzle diameter", exact: true });
  expect((await nozzles.locator("option").allTextContents()).sort()).toEqual(klipperVariants.map((variant) => `${variant} mm`).sort());
  await page.screenshot({ path: `${artifacts}finished.png` });

  // 6. A plate slices with the chosen printer.
  await page.getByTestId("import-input").setInputFiles(MODEL);
  await combo(page, "Process preset").click();
  await page.getByRole("option", { name: "0.20mm Standard @MyKlipper", exact: true }).click();
  await settled(page);
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.screenshot({ path: `${artifacts}sliced.png` });

  // 3. Re-opening shows the model checked; Confirm with none checked changes nothing.
  let models = await openModelsPage(page);
  await expect(models.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true })).toBeChecked();
  await models.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true }).uncheck();
  await models.getByRole("button", { name: "Confirm" }).click();
  await expect(models).toBeVisible();
  expect(await storedConfig(page)).toMatchObject({ installedPrinters: FIRST_RUN_CONFIG.installedPrinters });
  await page.keyboard.press("Escape");
  await expect(models).toHaveCount(0);

  // Adding a second model installs exactly that model and its variants.
  models = await openModelsPage(page);
  await models.getByRole("searchbox", { name: "Search printers" }).fill("a1 mini");
  await models.getByRole("checkbox", { name: "Bambu Lab A1 mini", exact: true }).check();
  await page.screenshot({ path: `${artifacts}second-model.png` });
  await models.getByRole("button", { name: "Confirm" }).click();
  await expect(models).toHaveCount(0, { timeout: 60_000 });
  await settled(page);
  const installed = {
    ...FIRST_RUN_CONFIG.installedPrinters,
    BBL: { "Bambu Lab A1 mini": modelVariants("BBL", "Bambu Lab A1 mini").sort() },
  };
  expect(await storedConfig(page)).toMatchObject({ installedPrinters: installed });
  // apply_config's preferred printer: the new model, at its first nozzle in nozzle_diameter order.
  const a1mini = `Bambu Lab A1 mini ${modelVariants("BBL", "Bambu Lab A1 mini")[0]} nozzle`;
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", a1mini);
  expect((await systemModels(page)).sort()).toEqual(["Bambu Lab A1 mini", "Generic Klipper Printer"]);

  // 5. A reload keeps the installed printers and does not open the wizard.
  await page.reload();
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", a1mini, { timeout: 60_000 });
  await settled(page);
  expect((await systemModels(page)).sort()).toEqual(["Bambu Lab A1 mini", "Generic Klipper Printer"]);
  await expect(page.getByRole("dialog")).toHaveCount(0);

  // A record of the vendor-level installer keeps its other fields, installs nothing and opens the wizard.
  await page.evaluate(() => {
    const record = JSON.parse(localStorage.getItem("ares.config")!) as Record<string, unknown>;
    delete record.installedPrinters;
    localStorage.setItem("ares.config", JSON.stringify({ ...record, theme: "light", vendors: ["BBL", "Custom"] }));
  });
  await page.reload();
  await expect(page.getByRole("dialog", { name: "Printer Selection" })).toBeVisible({ timeout: 30_000 });
  expect(await page.evaluate(() => document.documentElement.dataset.theme)).toBe("light");
  await page.screenshot({ path: `${artifacts}vendors-record.png` });
  expect(errors).toEqual([]);
});
