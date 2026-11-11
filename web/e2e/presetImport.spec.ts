import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { vendorProfiles } from "./filamentOracle";
import { projectConfig } from "./settingsOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const orca = process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`;
const artifacts = fileURLToPath(new URL("../../target/web-e2e/preset-import/", import.meta.url));
/** A BBL machine preset's `default_print_profile`, as its vendor file states it. */
const defaultPrint = (machine: string) => {
  const file = readFileSync(`${orca}/resources/profiles/BBL/machine/${machine}.json`, "utf8");
  return (JSON.parse(file) as { default_print_profile: string }).default_print_profile;
};

/** A BBL machine's first `default_filament_profile`, inherited from its parents. */
const defaultFilament = (machine: string): string => {
  const file = readFileSync(`${orca}/resources/profiles/BBL/machine/${machine}.json`, "utf8");
  const preset = JSON.parse(file) as { inherits: string; default_filament_profile?: string[] };
  return preset.default_filament_profile?.[0] ?? defaultFilament(preset.inherits);
};

interface Compatible {
  processes: string[];
  filaments: string[];
  diagnostics: { kind: string; name: string; message: string }[];
  missing_parents: { kind: string; name: string; parent: string }[];
}

/** Native `PresetCatalog` filtering of the same imports and OrcaSlicer vendor files (ares-wasm `compatible_presets` test). */
interface Expected {
  bundle: string;
  filament: string;
  printer: string;
  process: string;
  missingVendorError: string;
  withoutProcess: Compatible;
  withProcess: Compatible;
}

const expected = JSON.parse(
  readFileSync(`${repositoryRoot}target/web-e2e/expected-compatible.json`, "utf8"),
) as Expected;

/** Names of a combo's group, in list order; Escape closes the list. */
async function groupOptions(page: Page, combo: string, group: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  const options = page.getByRole("listbox", { name: combo, exact: true }).getByRole("group", { name: group }).getByRole("option");
  const names = await options.allTextContents();
  await page.keyboard.press("Escape");
  return names;
}

async function choose(page: Page, combo: string, name: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  await page.getByRole("listbox", { name: combo, exact: true }).getByRole("option", { name, exact: true }).click();
  await expect(page.getByRole("button", { name: combo, exact: true })).toHaveText(name);
}

/**
 * The plater printer combo's "System presets" items for one installed vendor, from the raw
 * OrcaSlicer vendor files: PlaterPresetComboBox::update keys each visible system printer by its
 * `printer_model` once (PresetComboBoxes.cpp:1238-1251) in a `std::map` (:1150), appended in key
 * order without a re-sort for the "System" group (:1317-1324, 1427). The wizard installs every
 * model and variant of the vendor (Preset::set_visible_from_appconfig, Preset.cpp:858-864); abstract
 * machine presets (`instantiation` "false") are not listed.
 */
function systemPrinterModels(vendor: string) {
  const models = systemPrinters(vendor).map((printer) => printer.printer_model);
  return [...new Set(models)].sort(ordinal);
}

const ordinal = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

/** The vendor's instantiable machine presets with their (inherited) model and variant, from the raw files. */
function systemPrinters(vendor: string) {
  const readJson = (path: string) => JSON.parse(readFileSync(`${orca}resources/profiles/${path}`, "utf8"));
  const index = readJson(`${vendor}.json`) as { machine_list: { sub_path: string }[] };
  const machines = new Map(index.machine_list.map(({ sub_path }) => {
    const machine = readJson(`${vendor}/${sub_path}`) as Record<string, string>;
    return [machine.name, machine];
  }));
  const field = (name: string, key: string): string => machines.get(name)![key] ?? field(machines.get(name)!.inherits, key);
  return [...machines.values()].filter((machine) => machine.instantiation === "true").map(({ name }) => ({
    name, printer_model: field(name, "printer_model"), printer_variant: field(name, "printer_variant"),
  }));
}

/**
 * PresetBundle::get_similar_printer_preset(model, {}) (PresetBundle.cpp:3750-3782) as the plater calls it
 * (Plater.cpp:9565-9566) with a project-inside printer selected, whose variant is `variantOld`: the
 * system candidates of the model keyed by name (:3757-3762). The alias step (:3765-3771) picks the same
 * preset as the variant step here: either alias ("" or the project preset name) yields no other name of
 * the model. Then the first candidate with the old variant (:3773-3779), else the first by name (:3781).
 */
function similarSystemPrinter(vendor: string, model: string, variantOld: string) {
  const candidates = systemPrinters(vendor).filter((printer) => printer.printer_model === model)
    .sort((a, b) => ordinal(a.name, b.name));
  return (candidates.find((printer) => printer.printer_variant === variantOld) ?? candidates[0]).name;
}

const timings = (page: Page) =>
  page.evaluate(() =>
    performance.getEntriesByType("measure")
      .filter((entry) => entry.name.startsWith("ares:"))
      .map(({ name, duration }) => ({ name, duration })),
  );

test("imported printer presets list the compatible processes and filaments", async ({ page }) => {
  test.setTimeout(240_000);
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/orca_cli_ender3.3mf`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  const projectPrinter = await page.getByRole("button", { name: "Printer preset", exact: true }).textContent();
  const sliceButton = page.getByRole("button", { name: "Slice plate" });
  await expect(sliceButton).toBeEnabled();

  // MainFrame::load_config_file: File > Import > Import Configs.
  await page.getByTestId("preset-import-input").setInputFiles(
    [expected.bundle, expected.filament].map((name) => `${repositoryRoot}tests/presets/${name}`),
  );
  expect(await groupOptions(page, "Printer preset", "User presets")).toEqual([expected.printer]);
  expect(await groupOptions(page, "Printer preset", "Project presets")).toEqual([projectPrinter]);

  // No system vendor installed: the engine's missing-parent error, verbatim.
  await choose(page, "Printer preset", expected.printer);
  await expect(page.getByTestId("preset-error")).toHaveText(expected.missingVendorError);
  await expect(sliceButton).toBeDisabled();
  await page.screenshot({ path: `${artifacts}missing-vendor.png` });

  // ConfigWizard: install the BBL vendor, as upstream requires before importing user presets.
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  // The first-run record's model is unchecked, so only BBL models stay installed.
  await wizard.getByRole("checkbox", { name: "Generic Klipper Printer", exact: true }).uncheck();
  await page.screenshot({ path: `${artifacts}vendors.png` });
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  // apply_config selects the first newly installed model's printer (PresetBundle.cpp:2050-2095);
  // the imported printer is chosen again.
  await choose(page, "Printer preset", expected.printer);
  await expect(page.getByTestId("preset-error")).toHaveCount(0);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  // One item per printer model, not per preset (48 instantiable BBL printers, 12 models).
  expect(await groupOptions(page, "Printer preset", "System presets")).toEqual(systemPrinterModels("BBL"));

  const processes = [
    ...await groupOptions(page, "Process preset", "User presets"),
    ...await groupOptions(page, "Process preset", "System presets"),
  ];
  expect(processes.sort()).toEqual([...expected.withoutProcess.processes].sort());
  await expect(page.getByTestId("preset-diagnostics")).toHaveCount(expected.withoutProcess.diagnostics.length ? 1 : 0);
  // Preset.cpp:1685-1691: presets whose parent is not loaded are skipped; the engine lists them.
  const missingParents = page.getByTestId("preset-missing-parents");
  await missingParents.locator("summary").click();
  await expect(missingParents.getByRole("listitem")).toHaveText(
    expected.withoutProcess.missing_parents.map(({ kind, name, parent }) => `${name} (${kind}): parent preset "${parent}" is not loaded`),
  );
  await page.screenshot({ path: `${artifacts}missing-parents.png` });
  // update_compatible(Always) selects the print (PresetBundle.cpp:8635-8639): with no previous print,
  // the inherited default_print_profile of the imported printer's parent.
  await expect(page.getByRole("button", { name: "Process preset", exact: true }))
    .toHaveText(defaultPrint("Bambu Lab X1 Carbon 0.4 nozzle"));
  // The filament pass of the same call filled the slot (Tab.cpp:7702-7713, PresetBundle.cpp:8637, 8656-8664),
  // so nothing blocks the slice.
  await expect(page.getByRole("button", { name: "Filament 1 preset", exact: true }))
    .toHaveText(defaultFilament("Bambu Lab X1 Carbon 0.4 nozzle"));
  await expect(sliceButton).toBeEnabled();
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  await page.getByRole("button", { name: "Process preset", exact: true }).click();
  await page.screenshot({ path: `${artifacts}processes.png` });
  await page.keyboard.press("Escape");

  await choose(page, "Process preset", expected.process);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const filaments = [
    ...await groupOptions(page, "Filament 1 preset", "User presets"),
    ...await groupOptions(page, "Filament 1 preset", "System presets"),
  ];
  // The combo lists the compatible filaments that are visible (PresetComboBoxes.cpp:1179-1187): user
  // presets and the installed system ones, seeded from the printer models' default materials.
  const bbl = vendorProfiles(["BBL"]);
  const visible = expected.withProcess.filaments.filter((name) => !bbl.filaments.has(name) || bbl.seeded.has(name));
  expect(filaments.sort()).toEqual([...visible].sort());
  // update_compatible(Always) on the printer change (Tab.cpp:7702-7713) filled the empty slot with the
  // printer's preferred default_filament_profile (PresetBundle.cpp:8637, 8656-8664).
  await expect(page.getByRole("button", { name: "Filament 1 preset", exact: true }))
    .toHaveText(defaultFilament("Bambu Lab X1 Carbon 0.4 nozzle"));
  await expect(sliceButton).toBeEnabled();
  // The imported PLA: Cool Plate, the 3MF plate, supports it.
  await choose(page, "Filament 1 preset", visible.find((name) => !bbl.filaments.has(name))!);
  // A 3MF with every preset selected slices through sliceWithProfiles.
  await expect(sliceButton).toBeEnabled();
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  // The BBL process stores the scalar bridge_speed as an array; upstream loads it
  // (Config.cpp:950-1000 joins it, ConfigOptionFloat::deserialize Config.hpp:784-790 reads the
  // leading value), so the imported printer over the 3MF slices.
  await sliceButton.click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.getByRole("button", { name: "Filament 1 preset", exact: true }).click();
  await page.screenshot({ path: `${artifacts}filaments.png` });
  await page.keyboard.press("Escape");

  // Back to the project preset: slicing is enabled again.
  await choose(page, "Printer preset", projectPrinter!);
  await expect(sliceButton).toBeEnabled();

  const measures = await timings(page);
  const build = measures.filter((entry) => entry.name === "ares:catalog-build");
  const filter = measures.filter((entry) => entry.name === "ares:filter-compatible-presets");
  // One catalog handle per vendor change (without and with BBL). Filtered for the first-run printer, the
  // imported printer without BBL, the printer Confirm selects, the imported printer with its print, and
  // the chosen process.
  // Filtering never serializes the whole catalog: that is only for slicing.
  expect(build).toHaveLength(2);
  expect(filter).toHaveLength(5);
  expect(measures.filter((entry) => entry.name === "ares:catalog-stringify")).toHaveLength(0);

  // A printer model selected over the project's printer, which no installed vendor holds: the combo
  // shows the model and its tooltip the resolved preset (PresetComboBoxes.cpp:1241-1244, 1265, 1486).
  const model = "Bambu Lab X1 Carbon";
  const project = projectConfig(`${repositoryRoot}tests/parity/orca_cli_ender3.3mf`);
  const resolved = similarSystemPrinter("BBL", model, String(project.printer_variant));
  await choose(page, "Printer preset", model);
  await expect(page.getByRole("button", { name: "Printer preset", exact: true })).toHaveAttribute("title", resolved);
  await expect(page.getByTestId("preset-error")).toHaveCount(0);
  await page.screenshot({ path: `${artifacts}printer-model.png` });
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    vendors: ["BBL", "OrcaFilamentLibrary"],
    printer: expected.printer,
    printerModel: { model, resolved },
    processes: processes.length,
    filaments: filaments.length,
    missingParents: expected.withoutProcess.missing_parents,
    // Without BBL, with BBL; and the filters without BBL, with BBL, with BBL and a process.
    catalogBuildMs: build.map((entry) => entry.duration),
    filterMs: filter.map((entry) => entry.duration),
  }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
