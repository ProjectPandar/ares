import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-nozzle/", import.meta.url));
// An X1 Carbon project on Cool Plate.
const PROJECT = `${repositoryRoot}tests/parity/adaptive-octree-spacing/baseline.3mf`;
const BED_TYPE = "Textured PEI Plate";

// PrintConfig.cpp:1086-1097: `curr_bed_type` values and labels, in BedType order (btPC = 1 ...).
const BED_TYPES = [
  ["Cool Plate", "Smooth Cool Plate"], ["Engineering Plate", "Engineering Plate"], ["High Temp Plate", "Smooth High Temp Plate"],
  ["Textured PEI Plate", "Textured PEI Plate"], ["Textured Cool Plate", "Textured Cool Plate"], ["Supertack Plate", "Cool Plate (SuperTack)"],
];

const readJson = (path: string) => JSON.parse(readFileSync(`${profiles}${path}`, "utf8"));
const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

/** A vendor's printers with resolved model, variant and default bed type, and its models' files. */
function vendorPrinters(vendor: string) {
  const index = readJson(`${vendor}.json`) as Record<string, { name: string; sub_path: string }[]>;
  const machines = new Map(index.machine_list.map(({ sub_path: path }) => {
    const machine = readJson(`${vendor}/${path}`) as Record<string, string>;
    return [machine.name, machine];
  }));
  const field = (name: string, key: string): string | undefined =>
    machines.get(name)![key] ?? (machines.get(name)!.inherits ? field(machines.get(name)!.inherits, key) : undefined);
  const models = index.machine_model_list.map(({ sub_path: path }) => readJson(`${vendor}/${path}`) as Record<string, string>);
  return { machines, field, models };
}

/** Sidebar::reset_bed_type_combox_choices (Plater.cpp:2793-2835) and Preset::get_default_bed_type (Preset.cpp:966-993). */
function plateOracle(vendor: string, printer: string) {
  const { field, models } = vendorPrinters(vendor);
  const model = models.find(({ name }) => name === field(printer, "printer_model"))!;
  const unsupported = (model.not_support_bed_type ?? "").split(";");
  const labels = BED_TYPES.filter(([, label]) => !unsupported.includes(label)).map(([, label]) => label);
  const defaultBed = field(printer, "default_bed_type") ?? "";
  const key = defaultBed !== "" ? (Number.parseInt(defaultBed, 10) > 0 ? Number.parseInt(defaultBed, 10) : 3)
    : ["BL-P001", "BL-P002", "C13"].includes(model.model_id) ? 1 : 3;
  return { labels, selected: BED_TYPES[key - 1][1] };
}

async function selectedLabel(page: Page, name: string) {
  return page.getByRole("combobox", { name, exact: true }).evaluate((select: HTMLSelectElement) => select.selectedOptions[0].textContent);
}

async function install(page: Page, vendor: string) {
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "System presets" });
  await wizard.getByRole("checkbox", { name: vendor, exact: true }).check();
  await wizard.getByRole("button", { name: "Done" }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

async function chooseModel(page: Page, model: string) {
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: model, exact: true }).click();
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", new RegExp(`^${model}`));
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

/** A copy of the project whose project config holds `edit` (slice.spec.ts `orcaEditedGcode`). */
function editedProject(edit: Record<string, string>) {
  const entries = unzipSync(readFileSync(PROJECT));
  const config = "Metadata/project_settings.config";
  entries[config] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[config])), ...edit }));
  const path = `${artifacts}edited.3mf`;
  writeFileSync(path, zipSync(entries));
  return path;
}

/** Official OrcaSlicer slicing a project file. */
function orcaGcode(project: string) {
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-plate-`)}/`;
  execFileSync(`${repositoryRoot}scripts/orca-parity.sh`, ["--slice", "0", "--outputdir", work, project], {
    cwd: work, stdio: ["ignore", "pipe", "pipe"],
  });
  const gcode = readFileSync(`${work}plate_1.gcode`, "utf8");
  writeFileSync(`${artifacts}native.gcode`, gcode);
  return normalize(gcode, "OrcaSlicer");
}

async function sliceAndExport(page: Page, name: string) {
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code" }).click()]);
  await download.saveAs(`${artifacts}${name}`);
  return readFileSync(`${artifacts}${name}`, "utf8");
}

/** Lines one G-code has more often than the other, counted by text. */
function lineDiff(a: string, b: string) {
  const count = (gcode: string) => gcode.split("\n").reduce((counts, line) => counts.set(line, (counts.get(line) ?? 0) + 1), new Map<string, number>());
  const [left, right] = [count(a), count(b)];
  const extra = (from: Map<string, number>, to: Map<string, number>) =>
    [...from].filter(([line, n]) => n > (to.get(line) ?? 0)).map(([line, n]) => `${line} (x${n - (to.get(line) ?? 0)})`);
  return { orcaOnly: extra(left, right), aresOnly: extra(right, left) };
}

test("plate type and nozzle offer the selected printer's options and reach the slice", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the project-edit oracle", "an extraction of the official OrcaSlicer 2.5.0-beta AppImage, see scripts/orca-parity.sh");
  test.setTimeout(400_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await install(page, "Bambulab");

  // A1 mini: two plates are not supported; the nozzle combo lists the model's printer variants.
  await chooseModel(page, "Bambu Lab A1 mini");
  const a1mini = (await combo(page, "Printer preset").getAttribute("title"))!;
  const plate = plateOracle("BBL", a1mini);
  const plateCombo = page.getByRole("combobox", { name: "Plate type", exact: true });
  expect(await plateCombo.locator("option").allTextContents()).toEqual(plate.labels);
  expect(await selectedLabel(page, "Plate type")).toBe(plate.selected);
  const { machines, field } = vendorPrinters("BBL");
  const variants = [...new Set([...machines.keys()].filter((name) => field(name, "printer_model") === "Bambu Lab A1 mini")
    .map((name) => field(name, "printer_variant")!))].sort();
  const nozzleCombo = page.getByRole("combobox", { name: "Nozzle diameter", exact: true });
  expect(await nozzleCombo.locator("option").allTextContents()).toEqual(variants.map((variant) => `${variant} mm`));
  expect(await selectedLabel(page, "Nozzle diameter")).toBe(`${field(a1mini, "printer_variant")} mm`);
  await page.screenshot({ path: `${artifacts}a1-mini.png` });

  // Another diameter selects the model's printer with that variant (get_similar_printer_preset).
  await nozzleCombo.selectOption("0.6");
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "Bambu Lab A1 mini 0.6 nozzle");
  await expect.poll(() => selectedLabel(page, "Nozzle diameter")).toBe("0.6 mm");
  // The chosen plate is stored for the printer and comes back after a reload.
  await plateCombo.selectOption({ label: "Textured PEI Plate" });
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "Bambu Lab A1 mini 0.6 nozzle", { timeout: 60_000 });
  await expect.poll(() => selectedLabel(page, "Plate type")).toBe("Textured PEI Plate");
  await page.screenshot({ path: `${artifacts}a1-mini-0.6.png` });

  // A printer without a BBL vendor or support_multi_bed_types has no plate combo (Plater.cpp:641-646).
  await install(page, "LH");
  await chooseModel(page, "LH Stinger");
  await expect(plateCombo).toHaveCount(0);
  await expect(nozzleCombo).toBeVisible();

  // Project mode: the X1C project's plate type is editable and slices as the edited project.
  await page.getByTestId("import-input").setInputFiles(PROJECT);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  const project = JSON.parse(strFromU8(unzipSync(readFileSync(PROJECT))["Metadata/project_settings.config"])) as Record<string, string>;
  await combo(page, "Printer preset").click();
  await page.getByRole("listbox", { name: "Printer preset" }).getByRole("group", { name: "Project presets" })
    .getByRole("option", { name: project.printer_settings_id, exact: true }).click();
  expect(await selectedLabel(page, "Plate type")).toBe(BED_TYPES.find(([value]) => value === project.curr_bed_type)![1]);
  await plateCombo.selectOption(BED_TYPE);
  const web = await sliceAndExport(page, "web.gcode");
  expect(web).toContain(`; curr_bed_type = ${BED_TYPE}`);
  await page.screenshot({ path: `${artifacts}project.png` });
  // The plate edit slices as the project file holding that plate type.
  const edited = editedProject({ curr_bed_type: BED_TYPE });
  await page.getByTestId("import-input").setInputFiles(edited);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  expect(normalize(web, "Ares"), "the plate edit equals the edited project").toBe(normalize(await sliceAndExport(page, "edited.gcode"), "Ares"));
  // Official OrcaSlicer on the edited project: the header and bed temperatures agree. Other lines
  // are recorded: ares-core does not yet emit the bed-temperature branch of the X1C
  // filament_start_gcode (`M106 P3 S150` at a 55 °C bed) on either path.
  let orcaDiff: ReturnType<typeof lineDiff> | null = null;
  if (orca) {
    const native = orcaGcode(edited);
    const pick = (gcode: string) => gcode.split("\n").filter((line) => /^; curr_bed_type = |^M1[49]0 /.test(line));
    expect(pick(normalize(web, "Ares"))).toEqual(pick(native));
    orcaDiff = lineDiff(native, normalize(web, "Ares"));
  }
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ a1mini, plate, variants, bedType: BED_TYPE, orcaDiff }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
