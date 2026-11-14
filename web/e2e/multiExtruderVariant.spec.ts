import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";

const root = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${root}OrcaSlicer`}/resources/profiles`;
const artifacts = process.env.ARES_VARIANT_ARTIFACT_ROOT ?? `${root}target/web-e2e/multi-extruder-variant`;
const cube = `${root}tests/parity/cube10.stl`;
const readJson = (path: string) => JSON.parse(readFileSync(path, "utf8").replace(/^\uFEFF/, ""));
const printer = (page: Page) => page.getByRole("button", { name: "Printer preset", exact: true });
const selectionConfig = (gcode: string) => gcode.split("\n").filter((line) =>
  /^; (nozzle_diameter|printer_settings_id|curr_bed_type) = /.test(line));

type Selection = { printer: string; process: string; filaments: string[]; bedType: string };

function nativePresets() {
  const datadir = `${artifacts}/preset-data`;
  const leaves = new Map<string, string>();
  for (const vendor of ["OrcaFilamentLibrary", "Snapmaker"]) {
    const destination = `${datadir}/system`;
    mkdirSync(destination, { recursive: true });
    cpSync(`${profiles}/${vendor}.json`, `${destination}/${vendor}.json`);
    cpSync(`${profiles}/${vendor}`, `${destination}/${vendor}`, { recursive: true });
    const index = readJson(`${profiles}/${vendor}.json`) as Record<string, { name: string; sub_path: string }[]>;
    for (const kind of ["machine", "process", "filament"]) {
      for (const { name, sub_path: path } of index[`${kind}_list`]) {
        leaves.set(`${kind}:${name}`, `${destination}/${vendor}/${path}`);
      }
    }
  }
  // fdm_U1.json printable_area: 0.5x1 .. 270.5x271; the browser centers the STL here.
  const centered = readFileSync(cube);
  for (let facet = 84; facet < centered.length; facet += 50) {
    for (let vertex = facet + 12; vertex < facet + 48; vertex += 12) {
      centered.writeFloatLE(centered.readFloatLE(vertex) + 135.5, vertex);
      centered.writeFloatLE(centered.readFloatLE(vertex + 4) + 136, vertex + 4);
    }
  }
  const input = `${artifacts}/cube10.stl`;
  writeFileSync(input, centered);
  return { datadir, leaves, input };
}

function nativeSlice(selection: Selection, variant: string, presets: ReturnType<typeof nativePresets>, bridgeLineWidth: string) {
  const output = `${artifacts}/native-${variant}`;
  mkdirSync(output, { recursive: true });
  const leaf = (kind: string, name: string) => presets.leaves.get(`${kind}:${name}`)!;
  const launcher = process.env.ARES_PARITY_SEEDED_REFERENCE === "1" ? process.env.ARES_ORCA_SEEDED_BIN : `${root}scripts/orca-parity.sh`;
  if (!launcher) throw new Error("The selected native mode needs its launcher");
  execFileSync(launcher, [
    "--datadir", presets.datadir,
    "--load-settings", [leaf("machine", selection.printer), leaf("process", selection.process)].join(";"),
    "--load-filaments", selection.filaments.map((name) => leaf("filament", name)).join(";"),
    "--curr-bed-type", selection.bedType, "--bridge-line-width", bridgeLineWidth,
    "--arrange", "0", "--slice", "0", "--outputdir", output, presets.input,
  ], { cwd: artifacts, stdio: ["ignore", "pipe", "pipe"] });
  return readFileSync(`${output}/plate_1.gcode`, "utf8");
}

async function selectedPresets(page: Page): Promise<Selection> {
  return page.evaluate(() => {
    const config = JSON.parse(localStorage.getItem("ares.config")!);
    return { printer: config.printer, ...config.printers[config.printer] };
  });
}

test("U1 machine variants and plate type reach the selected preset slice", async ({ page }) => {
  oraclePath("ORCA_APPDIR", "the machine-preset slicing oracle", "OrcaSlicer 2.5.0");
  test.setTimeout(600_000);
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(cube);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await printer(page).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Snapmaker models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await printer(page).click();
  await page.getByRole("option", { name: "Snapmaker U1", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });

  const variant = page.getByRole("combobox", { name: "Variant", exact: true });
  await expect(variant).toBeVisible();
  await expect(variant.locator("option")).toHaveText(["0.2 mm", "0.4 mm", "0.4+0.6 mm", "0.6 mm", "0.8 mm"]);
  await variant.selectOption("0.4");
  await expect(printer(page)).toHaveAttribute("title", "Snapmaker U1 (0.4 nozzle)", { timeout: 60_000 });
  await expect(variant).toHaveValue("0.4");
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });

  const presets = nativePresets();
  const records = [];
  for (const [value, diameters, bedType] of [
    ["0.2", "0.2,0.2,0.2,0.2", "High Temp Plate"],
    ["0.4+0.6", "0.4,0.4,0.6,0.6", "Textured PEI Plate"],
    ["0.8", "0.8,0.8,0.8,0.8", "High Temp Plate"],
  ]) {
    await variant.selectOption(value);
    await expect(printer(page)).toHaveAttribute("title", `Snapmaker U1 (${value} nozzle)`, { timeout: 60_000 });
    await expect(variant).toHaveValue(value);
    await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
    const plate = page.getByRole("combobox", { name: "Plate type", exact: true });
    await expect(plate).toBeVisible();
    await expect(plate.locator("option")).not.toHaveCount(0);
    await plate.selectOption(bedType);
    await expect(plate).toHaveValue(bedType);
    await expect.poll(async () => (await selectedPresets(page)).bedType).toBe(bedType);
    const selection = await selectedPresets(page);
    // The mixed preset's automatic bridge width exceeds its smaller nozzle in the native CLI.
    const bridgeLineWidth = value === "0.4+0.6" ? "0.4" : "100%";
    await page.getByRole("slider", { name: "Settings mode", exact: true }).focus();
    await page.keyboard.press("End");
    await page.getByRole("tab", { name: "Quality", exact: true }).click();
    const width = page.getByRole("textbox", { name: "bridge_line_width", exact: true });
    await width.fill(bridgeLineWidth);
    await width.press("Enter");
    await page.screenshot({ path: `${artifacts}/variant-${value}.png` });
    await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
    await page.getByRole("button", { name: "Slice plate", exact: true }).click();
    await expect(page.getByTestId("status")).toHaveText(/^(Slice complete|Slicing failed)/, { timeout: 240_000 });
    await expect(page.getByTestId("status")).toContainText("Slice complete");
    const download = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export G-code file", exact: true }).click();
    const path = `${artifacts}/variant-${value}.gcode`;
    await (await download).saveAs(path);
    const gcode = readFileSync(path, "utf8");
    expect(gcode).toContain(`; nozzle_diameter = ${diameters}\n`);
    expect(gcode).toContain(`; curr_bed_type = ${bedType}\n`);
    expect(gcode).toContain(`; printer_settings_id = ${selection.printer}\n`);
    const native = nativeSlice(selection, value, presets, bridgeLineWidth);
    expect(selectionConfig(gcode)).toEqual(selectionConfig(native));
    records.push({ variant: value, diameters, selection, bridgeLineWidth, config: selectionConfig(gcode) });
    await page.getByRole("link", { name: "Prepare", exact: true }).click();
  }
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await expect(variant).toHaveValue("0.8", { timeout: 60_000 });
  await expect(page.getByRole("combobox", { name: "Plate type", exact: true })).toHaveValue("High Temp Plate");
  expect(errors).toEqual([]);
  writeFileSync(`${artifacts}/result.json`, `${JSON.stringify({ records, errors }, null, 2)}\n`);
});

test("H2D keeps its paired nozzle controls when switching the printer diameter", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await printer(page).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await printer(page).click();
  await page.getByRole("option", { name: "Bambu Lab H2D", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const left = page.getByRole("combobox", { name: "Left Nozzle diameter", exact: true });
  const right = page.getByRole("combobox", { name: "Right Nozzle diameter", exact: true });
  await expect(left).toBeVisible();
  await expect(right).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Variant", exact: true })).toHaveCount(0);
  await expect(page.getByRole("combobox", { name: "Nozzle diameter", exact: true })).toHaveCount(0);
  const diameter = await left.inputValue() === "0.6" ? "0.4" : "0.6";
  await left.selectOption(diameter);
  const question = page.getByRole("dialog", { name: "Switch diameter" });
  await question.getByRole("button", { name: `Left nozzle: ${diameter}mm`, exact: true }).click();
  await expect(printer(page)).toHaveAttribute("title", `Bambu Lab H2D ${diameter} nozzle`, { timeout: 60_000 });
  await expect(left).toHaveValue(diameter);
  await expect(right).toHaveValue(diameter);
  await expect(page.getByRole("combobox", { name: "Plate type", exact: true })).toBeVisible();
  await page.screenshot({ path: `${artifacts}/h2d-paired-nozzles.png` });
});
