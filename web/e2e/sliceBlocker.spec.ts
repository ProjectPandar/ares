import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { FIRST_RUN_CONFIG, test } from "./failureDiagnostics";
import { vendorProfiles } from "./filamentOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/slice-blocker/", import.meta.url));
// Two filaments; the tests rewrite its project config.
const FIXTURE = `${repositoryRoot}tests/parity/bbl-jerk-tail-precision/stock-control.3mf`;
const A1M_FILAMENTS = ["Bambu PLA Basic @BBL A1M", "Bambu PLA Matte @BBL A1M"];
const H2D = "Bambu Lab H2D 0.4 nozzle";
const A1M = "Bambu Lab A1 mini 0.4 nozzle";

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

/** A copy of the fixture whose project config holds `edit`. */
function project(name: string, edit: Record<string, unknown>) {
  const entries = unzipSync(readFileSync(FIXTURE));
  const config = "Metadata/project_settings.config";
  entries[config] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[config])), ...edit }));
  writeFileSync(`${artifacts}${name}`, zipSync(entries));
  return `${artifacts}${name}`;
}

async function settled(page: Page) {
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

async function choose(page: Page, name: string, option: string) {
  await combo(page, name).click();
  await page.getByRole("listbox", { name, exact: true }).getByRole("option", { name: option, exact: true }).click();
  await settled(page);
}

/** The preset options of an open-and-closed filament combo, sorted. */
async function filamentOptions(page: Page, name: string) {
  await combo(page, name).click();
  const items = page.getByRole("listbox", { name, exact: true }).locator("[role=group] [role=option]");
  await expect(items.first()).toBeVisible();
  const options = await items.evaluateAll((nodes) => nodes.map((item) => item.textContent!.trim()));
  await page.keyboard.press("Escape");
  return [...new Set(options)].sort();
}

async function installBambuLab(page: Page) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
}

test.beforeAll(() => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
});

test("an A1M project switched to the H2D lists and selects only filaments compatible with the H2D, like update_compatible", async ({ page }) => {
  test.setTimeout(300_000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  // Oracle from the raw vendor files: the installed (seeded) BBL filaments that list the printer, and the
  // installed library filaments (the first-run ones) that no preset of the same alias lists the printer for
  // (update_library_profile_excluded_from, Preset.cpp:4365-4395; is_compatible_with_printer, :916-923).
  const bbl = vendorProfiles(["BBL"]);
  const alias = (name: string) => name.replace(/ @.*$/, "");
  const lists = (name: string, printer: string) => bbl.filament(name).compatiblePrinters.includes(printer);
  const superseded = (name: string, printer: string) =>
    [...bbl.filaments.keys()].some((other) => alias(other) === alias(name) && lists(other, printer));
  const library = FIRST_RUN_CONFIG.installedFilaments.filter((name) => name.endsWith(" @System"));
  const compatible = (printer: string) => [
    ...[...bbl.seeded].filter((name) => lists(name, printer)),
    ...library.filter((name) => !superseded(name, printer)),
  ].sort();
  const expected = ["Bambu PLA Basic @BBL H2D", "Bambu PLA Matte @BBL H2D"];
  for (const name of expected) expect(compatible(H2D)).toContain(name);

  await installBambuLab(page);
  // Both printers remember the A1M presets (AppConfig printer settings, PresetBundle::update_selections,
  // PresetBundle.cpp:3914-3937): the project was used on the A1 mini.
  await page.evaluate(({ printers, filaments }) => {
    const config = JSON.parse(localStorage.getItem("ares.config")!);
    for (const printer of printers) config.printers = { ...config.printers, [printer]: { filaments } };
    localStorage.setItem("ares.config", JSON.stringify(config));
  }, { printers: [H2D, A1M], filaments: A1M_FILAMENTS });
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project("a1m-filaments.3mf", { filament_settings_id: A1M_FILAMENTS }));
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  // update_compatible(Always) replaces each incompatible filament with the compatible preset of the
  // same alias (PresetBundle.cpp:8574-8598, 8664-8672).
  await choose(page, "Printer preset", "Bambu Lab H2D");
  await expect(combo(page, "Printer preset")).toHaveText("Bambu Lab H2D");
  for (const [slot, name] of expected.entries()) await expect(combo(page, `Filament ${slot + 1} preset`)).toHaveText(name);
  // PlaterPresetComboBox::update lists no incompatible preset (PresetComboBoxes.cpp:1300, 1501-1503).
  const h2dOptions = await filamentOptions(page, "Filament 2 preset");
  expect(h2dOptions).toEqual(compatible(H2D));
  expect(h2dOptions.filter((name) => name.includes("@BBL A1M"))).toEqual([]);

  // The same call selects the print: the H2D's default_print_profile (PreferedPrintProfileMatch,
  // PresetBundle.cpp:8638-8639; BBL/machine/fdm_bbl_3dp_002_common.json:23), so nothing blocks the slice.
  await expect(combo(page, "Process preset")).toHaveText("0.20mm Standard @BBL H2D");
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect(page.getByTestId("status")).not.toHaveClass(/warning/);
  await expect(page.getByRole("button", { name: "Slice plate" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Slice plate" })).not.toHaveAttribute("title");
  await page.screenshot({ path: `${artifacts}h2d-ready.png` });

  // The list follows the printer, and the A1 mini's remembered filaments come back.
  await choose(page, "Printer preset", "Bambu Lab A1 mini");
  await expect(combo(page, "Printer preset")).toHaveText("Bambu Lab A1 mini");
  for (const [slot, name] of A1M_FILAMENTS.entries()) await expect(combo(page, `Filament ${slot + 1} preset`)).toHaveText(name);
  const a1mOptions = await filamentOptions(page, "Filament 2 preset");
  expect(a1mOptions).toEqual(compatible(A1M));
  expect(a1mOptions).not.toEqual(h2dOptions);
  await choose(page, "Process preset", "0.20mm Standard @BBL A1M");
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  await page.screenshot({ path: `${artifacts}a1m-sliced.png` });
  expect(errors).toEqual([]);
});

test("a project with an out-of-range value warns at load like Plater::load_files and slices like the GUI", async ({ page }) => {
  test.setTimeout(300_000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project("raft-expansion.3mf", { raft_first_layer_expansion: "-1" }));
  // Plater.cpp:9466-9481: one "-key: message" line per invalid value, and the project loads.
  const warning = page.getByTestId("load-warning");
  await expect(warning).toContainText("Invalid values found in the 3MF:", { timeout: 60_000 });
  await expect(warning).toContainText("-raft_first_layer_expansion: -1 not in range");
  await expect(warning).toContainText("Please correct them in the Param tabs");
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.screenshot({ path: `${artifacts}raft-warning.png` });

  // BackgroundSlicingProcess::validate runs Print::validate only (BackgroundSlicingProcess.cpp:726-734).
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  await page.screenshot({ path: `${artifacts}raft-sliced.png` });
  await warning.getByRole("button", { name: "Close" }).click();
  await expect(warning).toHaveCount(0);
  expect(errors).toEqual([]);
});
