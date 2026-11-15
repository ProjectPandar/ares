import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { ORCA_SLICER_COMMIT } from "@ares-slicer/profiles";
import { test } from "./failureDiagnostics";
import { bedCommands, firstBedCommands, nativeBedReference } from "./bedDefaultOracle";
import { oraclePath } from "./oracleEnv";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const model = `${repository}tests/parity/adaptive-octree-spacing/baseline.3mf`;
const artifacts = `${repository}target/web-e2e/bed-default/`;

// U1's four filament slots deterministically crash Native CLI; use a single-nozzle profile.
// Qidi/machine/Qidi X-Plus 5 0.4 nozzle.json sets Textured PEI Plate, not the Cool Plate fallback.
test("Qidi X-Plus 5 initializes Textured PEI and exports Native first-layer bed temperatures without a bed selection", async ({ page }) => {
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
  const wizard = page.getByRole("dialog");
  await wizard.getByRole("button", { name: "Qidi", exact: true }).click();
  await wizard.getByRole("checkbox", { name: "Qidi X-Plus 5", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await combo("Printer preset").click();
  await page.getByRole("option", { name: "Qidi X-Plus 5", exact: true }).click();
  await page.getByRole("combobox", { name: "Nozzle diameter", exact: true }).selectOption("0.4");
  await expect(combo("Printer preset")).toHaveAttribute("title", "Qidi X-Plus 5 0.4 nozzle");
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const plate = page.getByRole("combobox", { name: "Plate type", exact: true });
  await expect(plate).toBeVisible();
  const bed = await plate.inputValue();
  writeFileSync(`${artifacts}initial-selection.json`, JSON.stringify({ bed, printer: await combo("Printer preset").getAttribute("title"), explicit_bed_selection: false }, null, 2));
  await page.screenshot({ path: `${artifacts}implicit-selection.png` });
  expect(bed).toBe("Textured PEI Plate");
  await combo("Process preset").click();
  await page.getByRole("option", { name: "0.20mm Standard @X-Plus 5", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await combo("Filament 1 preset").click();
  await page.getByRole("option", { name: "Add/Remove filaments", exact: true }).click();
  const filaments = page.getByRole("dialog", { name: "Filament selection" });
  await filaments.getByRole("group", { name: "Filaments", exact: true }).getByRole("checkbox", { name: "QIDI PLA Rapido", exact: true }).check();
  await filaments.getByRole("button", { name: "Done", exact: true }).click();
  const slots = page.getByRole("button", { name: /^Filament \d+ preset$/ });
  for (let index = 0; index < await slots.count(); index++) {
    await slots.nth(index).click();
    await page.getByRole("option", { name: "QIDI PLA Rapido @Qidi X-Plus 5 0.4 nozzle", exact: true }).click();
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
  expect(firstBedCommands(native)).toEqual(["M140 S60", "M190 S60"]);
  expect(firstBedCommands(browser)).toEqual(firstBedCommands(native));
  expect(browser).toContain("; curr_bed_type = Textured PEI Plate");
});

test("the actual WASM module resolves bed hints through its message protocol", async ({ page }) => {
  const workerReady = page.waitForEvent("worker");
  const wasmRequested = page.waitForRequest((request) => new URL(request.url()).pathname.endsWith(".wasm"));
  await page.goto("/");
  const workerUrl = (await workerReady).url();
  const wasmUrl = (await wasmRequested).url();
  const cases = [
    ["Cool Plate", "", "Cool Plate"], ["Engineering Plate", "", "Engineering Plate"],
    ["High Temp Plate", "", "High Temp Plate"], ["Textured PEI Plate", "", "Textured PEI Plate"],
    ["Textured Cool Plate", "", "Textured Cool Plate"], ["Supertack Plate", "", "Supertack Plate"],
    ["4suffix", "", "Textured PEI Plate"], [" +4tail", "", "Textured PEI Plate"],
    ["invalid", "BL-P001", "High Temp Plate"], ["", "BL-P001", "Cool Plate"],
    ["", "BL-P002", "Cool Plate"], ["", "C13", "Cool Plate"], ["", "C11", "High Temp Plate"],
    ["", "", "High Temp Plate"], ["", "Bambu Lab X1 Carbon", "High Temp Plate"],
  ];
  const actual = await page.evaluate(async ({ workerUrl, wasmUrl, cases }) => {
    const worker = new Worker(workerUrl, { type: "module" });
    let nextId = 0;
    const call = (request: Record<string, unknown>): Promise<unknown> => new Promise((resolve, reject) => {
      const id = nextId++;
      const receive = ({ data }: MessageEvent) => {
        if (data.id !== id || "status" in data) return;
        worker.removeEventListener("message", receive);
        if (data.ok) resolve(data.value);
        else reject(new Error(data.error));
      };
      worker.addEventListener("message", receive);
      worker.postMessage({ id, ...request });
    });
    try {
      await call({ kind: "init", wasmUrl });
      return await Promise.all(cases.map(([hint, modelId]) => call({ kind: "resolveDefaultBedType", hint, modelId })));
    } finally {
      worker.terminate();
    }
  }, { workerUrl, wasmUrl, cases });
  mkdirSync(artifacts, { recursive: true });
  writeFileSync(`${artifacts}resolver.json`, JSON.stringify({ workerUrl, wasmUrl, cases, actual }, null, 2));
  expect(actual).toEqual(cases.map(([, , expected]) => expected));
});

test("a hidden plate combo still initializes a concrete printer bed", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)", exact: true }).click();
  const wizard = page.getByRole("dialog");
  await wizard.getByRole("button", { name: "LH", exact: true }).click();
  await wizard.getByRole("checkbox", { name: "LH Stinger", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm", exact: true }).click();
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "LH Stinger", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect(page.getByRole("combobox", { name: "Plate type", exact: true })).toHaveCount(0);
  const state = () => page.evaluate(() => {
    const config = JSON.parse(localStorage.getItem("ares.config")!);
    return { printer: config.printer, bedType: config.printers[config.printer]?.bedType };
  });
  mkdirSync(artifacts, { recursive: true });
  writeFileSync(`${artifacts}hidden-combo.json`, JSON.stringify(await state(), null, 2));
  await expect.poll(async () => (await state()).bedType).toBe("High Temp Plate");
  writeFileSync(`${artifacts}hidden-combo.json`, JSON.stringify(await state(), null, 2));
  await page.evaluate(() => {
    const config = JSON.parse(localStorage.getItem("ares.config")!);
    config.printers[config.printer].bedType = "Cool Plate";
    localStorage.setItem("ares.config", JSON.stringify(config));
  });
  const { printer } = await state();
  await page.reload();
  // GUI_App startup loads the preset bundle (GUI_App.cpp:3723) before load_selections selects the stored
  // printer (PresetBundle.cpp:2723); the printer's bed is set when that selection ends. The combo shows the
  // printer as the selection starts, and the filtering state until it ends.
  await expect(page.getByRole("button", { name: "Printer preset", exact: true })).toHaveAttribute("title", printer, { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect.poll(async () => (await state()).bedType).toBe("High Temp Plate");
  writeFileSync(`${artifacts}hidden-combo-stored.json`, JSON.stringify(await state(), null, 2));
});

test("supported remembered beds win and unsupported remembered beds use the first listed choice", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  const printer = page.getByRole("button", { name: "Printer preset", exact: true });
  await printer.click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)", exact: true }).click();
  const wizard = page.getByRole("dialog");
  await wizard.getByRole("button", { name: "Snapmaker", exact: true }).click();
  await wizard.getByRole("checkbox", { name: "Snapmaker U1", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm", exact: true }).click();
  await printer.click();
  await page.getByRole("option", { name: "Snapmaker U1", exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const plate = page.getByRole("combobox", { name: "Plate type", exact: true });
  await plate.selectOption("High Temp Plate");
  await page.reload();
  await expect(plate).toHaveValue("High Temp Plate", { timeout: 60_000 });
  const firstListed = await plate.locator("option").first().getAttribute("value");
  await page.evaluate(() => {
    const config = JSON.parse(localStorage.getItem("ares.config")!);
    config.printers[config.printer].bedType = "Unsupported Plate";
    localStorage.setItem("ares.config", JSON.stringify(config));
  });
  await page.reload();
  await expect(plate).toHaveValue(firstListed!, { timeout: 60_000 });
  mkdirSync(artifacts, { recursive: true });
  writeFileSync(`${artifacts}remembered.json`, JSON.stringify({ selected: await plate.inputValue(), options: await plate.locator("option").allTextContents() }, null, 2));
});
