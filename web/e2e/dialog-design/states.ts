import { readFileSync } from "node:fs";
import { type BrowserContext, expect, type Locator, type Page } from "@playwright/test";
import {
  artifacts,
  config,
  CONTENT_MARGINS,
  crashEngine,
  CUBE,
  dialog,
  dualExtruder,
  fileItem,
  filamentSelection,
  firstRun,
  helpItem,
  importFile,
  mixedFilament,
  openSidebar,
  parity,
  printerSelection,
  PROJECT,
  real,
  ready,
  seed,
  SETTINGS_PARTS,
  settingsNotes,
  upstream,
  wizardFilaments,
  wizardPrinters,
  WIZARD_LIST,
} from "./app";
import { CLOUD_OPENINGS } from "./cloud";

export interface Opening {
  theme: "light" | "dark";
  /** What differs by content: the app's real data and upstream wording where the artboard shows placeholders. */
  notes: string[];
  /** The app's selectors for the parts it renders with the sidebar's classes (the settings pages). */
  appSelectors?: Record<string, string>;
  /** Properties the app sets on purpose, which the artboard states otherwise (DESIGN.md's decisions). */
  ignored?: { part: string; property: string }[];
  open: (env: { page: Page; context: BrowserContext; baseURL?: string }) => Promise<Locator>;
}

const SET_PRINTER = {
  notes: [...settingsNotes, "the subtitle names the selected preset"],
  appSelectors: SETTINGS_PARTS,
};

/** How the app reaches each dialog artboard, by `design.json` id. */
export const OPENINGS: Record<string, Opening> = {
  ...CLOUD_OPENINGS,
  "new-project": {
    theme: "light",
    notes: [],
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      await importFile(page, CUBE);
      await fileItem(page, "New Project");
      return dialog(page, "Discard the current project?");
    },
  },
  multipart: {
    theme: "light",
    notes: [],
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      await page.getByTestId("import-input").setInputFiles([
        { name: "base.stl", mimeType: "model/stl", buffer: readFileSync(CUBE) },
        { name: "upper.stl", mimeType: "model/stl", buffer: readFileSync(CUBE) },
      ]);
      return dialog(page, "An object with multiple parts was detected");
    },
  },
  copies: {
    theme: "light",
    notes: [upstream],
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      await importFile(page, CUBE);
      await page.getByTestId("object-list").getByRole("treeitem").nth(1).click({ button: "right" });
      await page
        .getByRole("menu", { name: "Object menu" })
        .getByRole("menuitem", { name: "Set number of instances..." })
        .click();
      return dialog(page, "Copies of the selected object");
    },
  },
  arrange: {
    theme: "light",
    notes: ["a printer without an extrusion calibration region hides that row"],
    // D4: the popup is anchored to the 3D view, not placed at the artboard's window coordinates.
    ignored: [{ part: "dialog", property: "position" }],
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      await importFile(page, CUBE);
      await page.getByRole("button", { name: "Arrange", exact: true }).click();
      const popup = dialog(page, "Arrange settings");
      await expect(popup).toBeVisible();
      return popup;
    },
  },
  "switch-diameter": {
    theme: "light",
    notes: [upstream],
    open: async ({ page }) => {
      seed(page, dualExtruder);
      await ready(page);
      await importFile(page, `${parity}master-extruder-id/stock-right-only.3mf`);
      const left = page.getByRole("combobox", { name: "Left Nozzle diameter", exact: true });
      await expect(left).toBeVisible({ timeout: 60_000 });
      await left.selectOption("0.6");
      return dialog(page, "Switch diameter");
    },
  },
  "merge-filament": {
    theme: "light",
    notes: [upstream],
    open: async ({ page }) => {
      // Slot menus need an editable printer: a BBL one (physical_slots.rs `can_remove`).
      seed(page, dualExtruder);
      await ready(page);
      await importFile(
        page,
        mixedFilament("master-extruder-id/stock-right-only.3mf", `${artifacts}merge-filament/mixed.3mf`),
      );
      const actions = page.getByRole("button", { name: "Filament 1 actions", exact: true });
      await expect(actions).toBeVisible({ timeout: 60_000 });
      await actions.click();
      await page.getByRole("menuitem", { name: "Merge with", exact: true }).click();
      await page.getByRole("menuitem", { name: /^Filament 3:/ }).click();
      return dialog(page, "Merge filament?");
    },
  },
  "printer-settings": {
    theme: "light",
    ...SET_PRINTER,
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      await importFile(page, PROJECT);
      await openSidebar(page);
      await page.getByRole("button", { name: "Edit printer settings" }).click();
      return dialog(page, "Printer settings");
    },
  },
  "printer-settings-mobile": {
    theme: "light",
    ...SET_PRINTER,
    open: async (env) => OPENINGS["printer-settings"]!.open(env),
  },
  "filament-settings": {
    theme: "dark",
    notes: settingsNotes,
    appSelectors: SETTINGS_PARTS,
    open: async ({ page }) => {
      seed(page, config("dark"));
      await ready(page);
      await importFile(page, PROJECT);
      await openSidebar(page);
      await page.getByRole("button", { name: "Edit filament 1 settings" }).click();
      return dialog(page, "Filament settings");
    },
  },
  "filament-library": {
    theme: "light",
    notes: [real, "the subtitle names the selected printer preset"],
    ignored: CONTENT_MARGINS,
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      return filamentSelection(page);
    },
  },
  "filament-library-mobile": {
    theme: "light",
    notes: [real, "the subtitle names the selected printer preset"],
    ignored: CONTENT_MARGINS,
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      return filamentSelection(page);
    },
  },
  "wizard-printers": {
    theme: "light",
    notes: [real],
    // D3: the vendor nav scrolls within the dialog on a long catalog.
    ignored: WIZARD_LIST,
    open: async ({ page }) => {
      seed(page, firstRun("light"));
      return wizardPrinters(page);
    },
  },
  "manage-printers": {
    theme: "light",
    notes: [real],
    ignored: WIZARD_LIST,
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      return printerSelection(page);
    },
  },
  "wizard-filaments": {
    theme: "light",
    notes: [real],
    ignored: CONTENT_MARGINS,
    open: async ({ page }) => {
      seed(page, firstRun("light"));
      return wizardFilaments(page);
    },
  },
  "wizard-filament-error": {
    theme: "light",
    notes: [real],
    ignored: CONTENT_MARGINS,
    open: async ({ page }) => {
      seed(page, firstRun("light"));
      const filaments = await wizardFilaments(page);
      const checkedBoxes = await filaments.getByRole("checkbox", { checked: true }).all();
      for (const box of checkedBoxes.reverse()) await box.uncheck();
      await filaments.getByRole("button", { name: "Finish", exact: true }).click();
      await expect(filaments.getByRole("alertdialog", { name: "Error" })).toBeVisible();
      return filaments;
    },
  },
  shortcuts: {
    theme: "dark",
    notes: [upstream],
    open: async ({ page }) => {
      seed(page, config("dark"));
      await ready(page);
      await helpItem(page, "Keyboard Shortcuts");
      return dialog(page, "Keyboard Shortcuts");
    },
  },
  about: {
    theme: "light",
    notes: ["the credits are the app's full attribution"],
    open: async ({ page }) => {
      seed(page, config("light"));
      await ready(page);
      await helpItem(page, "About Ares");
      return dialog(page, "About Ares");
    },
  },
  "engine-crash": {
    theme: "dark",
    notes: ["the message is the engine's HardCrash text, on its two lines, and the file is the imported model"],
    // BackgroundSlicingProcess.cpp's text carries a line break of its own.
    ignored: [{ part: "notice", property: "white-space" }],
    open: async ({ page }) => crashEngine(page),
  },
};
