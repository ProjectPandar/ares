import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { type Route, expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { FIRST_RUN_CONFIG } from "../failureDiagnostics";

/**
 * How the app is driven into the state of a dialog artboard. The app config is seeded, never
 * clicked through: the theme, the printers and the Cloud session of the artboard.
 */
export const repositoryRoot = fileURLToPath(new URL("../../../", import.meta.url));
export const parity = `${repositoryRoot}tests/parity/`;
export const artifacts = `${repositoryRoot}target/web-e2e/dialog-design/`;
/** `bun scripts/build-panic-probe.mjs` builds the engine that panics while slicing. */
const probe = `${repositoryRoot}target/web-e2e/panic-probe/`;

export type Config = Record<string, unknown>;

export const seed = (page: Page, config: Config) =>
  page.addInitScript((record) => {
    if (localStorage.getItem("ares.config") === null) localStorage.setItem("ares.config", record);
  }, JSON.stringify(config));

export const config = (theme: "light" | "dark", extra: Config = {}): Config => ({
  ...FIRST_RUN_CONFIG,
  theme,
  ...extra,
});

export const status = (page: Page) => page.getByTestId("status");
/** The open `<dialog class="dialog">` whose title is `name`, whatever its role, or the arrange `.popup`. */
export const dialog = (page: Page, name: string) =>
  page.locator("dialog.dialog[open], .popup").filter({ has: page.getByRole("heading", { name, exact: true }) });

/** On a phone the sidebar is a drawer: its buttons exist only once it is opened. */
export async function openSidebar(page: Page) {
  const toggle = page.locator(".panel-toggle");
  if (await toggle.isVisible()) await toggle.click();
}

export async function ready(page: Page) {
  await page.goto("/#/prepare");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

export async function importFile(page: Page, file: string) {
  await page.getByTestId("import-input").setInputFiles(file);
  await expect(status(page)).toHaveText("Ready to slice", { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

export async function slice(page: Page) {
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(status(page)).toContainText("Slice complete", { timeout: 180_000 });
}

export const fileItem = async (page: Page, name: string) => {
  await page.getByRole("button", { name: "File", exact: true }).click();
  await page.getByRole("menu", { name: "File", exact: true }).getByRole("menuitem", { name, exact: true }).click();
};

export const helpItem = async (page: Page, name: string) => {
  await page.getByRole("button", { name: "Help", exact: true }).click();
  await page.getByRole("menu", { name: "Help", exact: true }).getByRole("menuitem", { name, exact: true }).click();
};

/** The printer model page the printer combo opens (PresetComboBoxes.cpp:1026-1037). */
export async function printerSelection(page: Page) {
  await openSidebar(page);
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = dialog(page, "Printer Selection");
  await expect(wizard.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true })).toBeChecked({
    timeout: 60_000,
  });
  return wizard;
}

export async function filamentSelection(page: Page) {
  await openSidebar(page);
  await page.getByRole("button", { name: "Set filaments to use", exact: true }).click();
  const library = dialog(page, "Filament selection");
  await expect(library.getByRole("group", { name: "Filaments" }).getByRole("checkbox").first()).toBeVisible({
    timeout: 60_000,
  });
  return library;
}

/** A first run in the artboard's theme: a stored config with no printer opens the wizard (presets.ts). */
export const firstRun = (theme: "light" | "dark") =>
  config(theme, { installedPrinters: {}, printer: null, printers: {}, installedFilaments: [] });

export async function wizardPrinters(page: Page) {
  await page.goto("/#/prepare");
  const wizard = dialog(page, "Printer Selection");
  await wizard.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true }).check({ timeout: 60_000 });
  await expect(wizard.getByRole("button", { name: "Next", exact: true })).toBeEnabled({ timeout: 60_000 });
  return wizard;
}

export async function wizardFilaments(page: Page) {
  await wizardPrinters(page);
  await dialog(page, "Printer Selection").getByRole("button", { name: "Next", exact: true }).click();
  const filaments = dialog(page, "Filament Selection");
  await expect(filaments.getByRole("group", { name: "Filaments" }).getByRole("checkbox").first()).toBeVisible({
    timeout: 60_000,
  });
  return filaments;
}

/** `source` with a third, mixed filament that lists slot 1 as a component, after two physical ones. */
export function mixedFilament(source: string, target: string) {
  const entries = unzipSync(readFileSync(`${parity}${source}`));
  const key = "Metadata/project_settings.config";
  const settings = JSON.parse(strFromU8(entries[key]!)) as Config;
  const colours = settings.filament_colour as string[];
  const presets = settings.filament_settings_id as string[];
  entries[key] = strToU8(
    JSON.stringify({
      ...settings,
      filament_colour: [...colours, colours.at(-1)],
      filament_settings_id: [...presets, presets.at(-1)],
      filament_is_mixed: [...colours.map(() => "0"), "1"],
      filament_mixed_components: [...colours.map(() => ""), "1"],
    }),
  );
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, zipSync(entries));
  return target;
}

/** The first page load runs the probe engine, whose slice panics (wasmPanic.spec.ts). */
export async function crashEngine(page: Page) {
  const first = (file: string, contentType: string) => {
    let served = false;
    return async (route: Route) => {
      if (served) return route.fallback();
      served = true;
      await route.fulfill({ body: readFileSync(`${probe}${file}`), contentType });
    };
  };
  await page.route("**/assets/worker-*.js", first("worker.js", "text/javascript"));
  await page.route("**/assets/ares_wasm_bg-*.wasm", first("ares_wasm_bg.wasm", "application/wasm"));
  seed(page, config("dark"));
  await ready(page);
  await importFile(page, CUBE);
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  const crash = dialog(page, "Slicing engine failed");
  await expect(crash).toBeVisible({ timeout: 60_000 });
  return crash;
}

export const CUBE = `${parity}cube10.stl`;
/** A project with settings: the printer and filament editors need its ProjectSettings. */
export const PROJECT = `${parity}orca_cli_ender3.3mf`;
/** The H2D, a dual-nozzle BBL printer: its Left and Right Nozzle groups and editable filament slots. */
export const dualExtruder = config("light", {
  installedPrinters: { BBL: { "Bambu Lab H2D": ["0.4"] }, ...FIRST_RUN_CONFIG.installedPrinters },
  printer: "Bambu Lab H2D 0.4 nozzle",
});

export const real = "lists the real catalog or preset, not the artboard's placeholder rows";
export const upstream = "keeps the upstream OrcaSlicer wording the app cites, not the artboard's paraphrase";
export const settingsNotes: string[] = [real, "the pages list the selected preset's real options"];
/** The settings pages reuse the sidebar's option controls; their rows answer to the design's field names. */
export const SETTINGS_PARTS = {
  tabs: ".process-tabs",
  tab: ".process-tab:not(.active)",
  "tab-active": ".process-tab.active",
  field: ".optgroup .option-row:not(.option-line)",
  "field-label": ".option-label",
  input: ".optgroup .option-row :is(.option-field, .option-select)",
  "setting-field": ".optgroup .option-row:not(.option-line)",
  "setting-input": ".optgroup .option-row :is(.option-field, .option-select)",
};
/** A summary or detail pushed aside by `margin: auto` ends where its own text ends. */
export const CONTENT_MARGINS = [
  { part: "summary", property: "margin-right" },
  { part: "check-detail", property: "margin-left" },
];
/** The wizard's vendor nav scrolls in place on a long catalog (DESIGN.md D3). */
export const WIZARD_LIST = [
  { part: "summary", property: "margin-right" },
  { part: "vendor-nav", property: "overflow-x" },
  { part: "vendor-nav", property: "overflow-y" },
];
