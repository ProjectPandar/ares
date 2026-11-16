import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/persistence/", import.meta.url));
// A two-filament project.
const PROJECT = `${repositoryRoot}tests/parity/master-extruder-id/stock-right-only.3mf`;
const IMPORT = `${repositoryRoot}tests/presets/My X1C 0.4 nozzle.orca_printer`;
const A1_MINI = "Bambu Lab A1 mini 0.4 nozzle";
const orca = process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`;
// The never-used single-extruder printer has slot 1 and no stored filament_NN (PresetBundle::update_selections,
// PresetBundle.cpp:3929-3937), and min_filament_count() is 1 (5419-5424): one slot, whatever the project has.
const SELECTION = {
  printer: "My X1C 0.4 nozzle",
  process: "0.20mm Standard @BBL X1C - Mine",
  filaments: ["Bambu PLA Basic @BBL X1C - Mine"],
};

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

async function settled(page: Page) {
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

async function choose(page: Page, name: string, option: string) {
  await combo(page, name).click();
  await page.getByRole("listbox", { name, exact: true }).getByRole("option", { name: option, exact: true }).click();
  await settled(page);
}

async function open(page: Page) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
}

async function loadProject(page: Page) {
  await page.getByTestId("import-input").setInputFiles(PROJECT);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
}

async function expectSelection(page: Page) {
  await expect(combo(page, "Printer preset")).toHaveText(SELECTION.printer, { timeout: 60_000 });
  await settled(page);
  await expect(combo(page, "Process preset")).toHaveText(SELECTION.process);
  for (const [slot, filament] of SELECTION.filaments.entries()) await expect(combo(page, `Filament ${slot + 1} preset`)).toHaveText(filament);
  await expect(combo(page, `Filament ${SELECTION.filaments.length + 1} preset`)).toHaveCount(0);
}

/** Names in a combo's group; Escape closes the list. */
async function groupOptions(page: Page, name: string, group: string) {
  await combo(page, name).click();
  const names = await page.getByRole("listbox", { name, exact: true }).getByRole("group", { name: group, exact: true })
    .getByRole("option").allTextContents();
  await page.keyboard.press("Escape");
  return names;
}

const importCount = (page: Page) => page.evaluate(async () => {
  const db = await new Promise<IDBDatabase>((resolve) => {
    const request = indexedDB.open("ares");
    request.onsuccess = () => resolve(request.result);
  });
  const count = await new Promise<number>((resolve) => {
    const request = db.transaction("presetImports").objectStore("presetImports").count();
    request.onsuccess = () => resolve(request.result);
  });
  db.close();
  return count;
});

test("selections, imported presets and UI preferences survive a reload; corrupt entries are dropped", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });

  await open(page);
  await loadProject(page);
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await page.getByTestId("preset-import-input").setInputFiles(IMPORT);
  await choose(page, "Printer preset", SELECTION.printer);
  await choose(page, "Process preset", SELECTION.process);
  for (const [slot, filament] of SELECTION.filaments.entries()) await choose(page, `Filament ${slot + 1} preset`, filament);
  await page.getByRole("button", { name: "Light theme" }).click();
  await page.getByRole("slider", { name: "Settings mode" }).press("End");
  await expectSelection(page);
  await page.screenshot({ path: `${artifacts}before-reload.png` });

  // The model is not persisted; it comes back with the project. Opening it selects
  // its presets (PresetBundle::load_config_model, Plater.cpp:9514-9515, :9623); choosing the printer
  // again brings back its process and filaments (remember_printer_config).
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await loadProject(page);
  expect(await page.evaluate(() => (JSON.parse(localStorage.getItem("ares.config")!) as { printer: string | null }).printer)).toBeNull();
  await choose(page, "Printer preset", SELECTION.printer);
  await expectSelection(page);
  expect(await page.evaluate(() => document.documentElement.dataset.theme)).toBe("light");
  await expect(page.getByRole("slider", { name: "Settings mode" })).toHaveAttribute("aria-valuetext", "Expert settings");
  expect(await groupOptions(page, "Printer preset", "User presets")).toEqual([SELECTION.printer]);
  await page.screenshot({ path: `${artifacts}after-reload.png` });

  // remember_printer_config: another printer does not take this printer's process. update_compatible
  // (PresetBundle.cpp:8635-8639) gives it the print matching the user print's alias, its whole name
  // (Preset.cpp:4414-4437), else its layer height: A1 mini's default_print_profile. Switching back
  // brings the process and filaments back.
  await choose(page, "Printer preset", "Bambu Lab A1 mini");
  const a1mini = JSON.parse(readFileSync(`${orca}/resources/profiles/BBL/machine/${A1_MINI}.json`, "utf8")) as
    { default_print_profile: string };
  await expect(combo(page, "Process preset")).toHaveText(a1mini.default_print_profile);
  await choose(page, "Printer preset", SELECTION.printer);
  await expectSelection(page);
  await page.screenshot({ path: `${artifacts}printer-switch.png` });

  // Corrupt or old app config records: each is dropped and the app starts with the defaults.
  const records = {
    old: JSON.stringify({ version: 0, theme: "light" }),
    invalid: "{",
    badField: JSON.stringify({ version: 1, theme: 5, mode: "simple", gcodeWindow: true, vendors: [], printer: null, printers: {} }),
  };
  const dropped: Record<string, boolean> = {};
  for (const [name, record] of Object.entries(records)) {
    await page.evaluate((text) => localStorage.setItem("ares.config", text), record);
    await page.reload();
    await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
    await expect(combo(page, "Printer preset")).toHaveText("No printer preset");
    expect(await page.evaluate(() => document.documentElement.dataset.theme)).toBe("dark");
    // The corrupt record is gone; the app may already have written a fresh default record.
    dropped[name] = await page.evaluate(() => {
      const text = localStorage.getItem("ares.config");
      if (text === null) return true;
      const stored = JSON.parse(text) as { version: number; theme: string; printer: string | null; installedPrinters: object };
      return stored.version === 1 && stored.theme === "dark" && stored.printer === null
        && Object.keys(stored.installedPrinters).length === 0;
    });
    expect(dropped[name], name).toBe(true);
  }
  await page.screenshot({ path: `${artifacts}corrupt-config.png` });

  // Corrupt import entries: a file that does not parse and an entry of the wrong shape are deleted;
  // the valid import still loads.
  await page.evaluate(async () => {
    const db = await new Promise<IDBDatabase>((resolve) => {
      const request = indexedDB.open("ares");
      request.onsuccess = () => resolve(request.result);
    });
    const transaction = db.transaction("presetImports", "readwrite");
    const store = transaction.objectStore("presetImports");
    store.add({ version: 1, filename: "broken.json", bytes: new TextEncoder().encode("{ not json") });
    store.add({ version: 1, filename: "broken.json", bytes: "not bytes" });
    store.add({ unrelated: true });
    await new Promise((resolve) => (transaction.oncomplete = resolve));
    db.close();
  });
  expect(await importCount(page)).toBe(4);
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await expect.poll(() => importCount(page)).toBe(1);
  expect(await groupOptions(page, "Printer preset", "User presets")).toEqual([SELECTION.printer]);
  await page.screenshot({ path: `${artifacts}corrupt-imports.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ selection: SELECTION, dropped, importsAfterCleanup: 1 }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
