import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { AxeBuilder } from "@axe-core/playwright";
import { expect, type Locator, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { FILAMENT_PAGES, PRINTER_PAGES } from "../app/src/sidebar/presetLayout";
import { PROCESS_PAGES } from "../app/src/sidebar/processLayout";
import type { LayoutPage } from "../app/src/sidebar/layoutTypes";
import { decodedSettings, type Decoded, expectedPage, MODE_NAMES, MODES, nativeConfigDef, pageVisible, projectConfig } from "./settingsOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/settings/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const slug = (title: string) => title.toLowerCase().replace(/[^a-z0-9]+/g, "-");

const defs = nativeConfigDef(repositoryRoot);
const config = projectConfig(fixture);
const decoded = decodedSettings(repositoryRoot);
/** Typed values per tab: the project settings under each preset's own (as the app merges them). */
const TYPED: Record<keyof typeof EXPERT_TITLES, Decoded> = {
  process: { ...decoded.project, ...decoded.process },
  printer: { ...decoded.project, ...decoded.printer },
  filament: { ...decoded.project, ...decoded.filament },
};

// Expert-mode page order of OrcaSlicer src/slic3r/GUI/Tab.cpp (pin 8500fcdc):
// TabPrint::build 2311-2800, TabFilament::build 3892-4243 (+ add_filament_overrides_page 3620),
// TabPrinter::build_fff 4457 + build_unregular_pages 4927 for a Marlin-flavour printer.
const EXPERT_TITLES = {
  process: ["Quality", "Strength", "Speed", "Support", "Multimaterial", "Others"],
  filament: ["Filament", "Cooling", "Setting Overrides", "Advanced", "Multimaterial", "Dependencies", "Notes"],
  printer: ["Basic information", "Machine G-code", "Multimaterial", "Extruder", "Motion ability", "Notes"],
};

/** Screenshots `target` with the viewport grown until `scroller` shows all of its content. */
async function fullShot(page: Page, target: Locator, scroller: Locator, path: string) {
  const { scrollHeight, clientHeight } = await scroller.evaluate((element) => ({ scrollHeight: element.scrollHeight, clientHeight: element.clientHeight }));
  const viewport = page.viewportSize()!;
  // The dialog is 84vh tall, so its scroller grows by 0.84 of the added height.
  const scale = (await scroller.evaluate((element) => Boolean(element.closest("dialog")))) ? 0.84 : 1;
  await page.setViewportSize({ width: viewport.width, height: Math.ceil(viewport.height + (scrollHeight - clientHeight) / scale) + 8 });
  await target.screenshot({ path });
  await page.setViewportSize(viewport);
}

/** Every rendered option of the active page: its control kind and the metadata it displays. */
const renderedFields = (scope: Locator) =>
  scope.locator("[data-option]").evaluateAll((elements) => elements.map((element) => {
    // PointCtrl (Field.cpp:2123): an x and a y TextInput.
    if (element.classList.contains("option-point")) {
      const inputs = [...element.querySelectorAll("input")];
      const label = element.closest(".option-subfield")?.querySelector(".option-sublabel") ?? element.closest(".option-row")!.querySelector(".option-label");
      return {
        key: element.getAttribute("data-option"), control: "point", label: label!.textContent,
        unit: element.closest(".option-value")!.querySelector(".option-unit")?.textContent ?? null,
        tooltip: element.getAttribute("title"), value: inputs.map((input) => input.value).join(","),
      };
    }
    const control = element as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;
    const kind = control instanceof HTMLSelectElement ? "choice"
      : control.type === "checkbox" ? "checkbox"
      : control instanceof HTMLTextAreaElement ? "code"
      : control.list ? "combo"
      : control.type === "number" ? "spin" : "text";
    const label = element.closest(".option-subfield")?.querySelector(".option-sublabel") ?? element.closest(".option-row")!.querySelector(".option-label");
    const field: Record<string, unknown> = {
      key: element.getAttribute("data-option"),
      control: kind,
      label: label!.textContent,
      unit: element.closest(".option-value")!.querySelector(".option-unit")?.textContent ?? null,
      tooltip: element.getAttribute("title"),
    };
    if (control instanceof HTMLSelectElement) field.options = [...control.options].filter((option) => !option.hidden).map(({ value, text }) => ({ value, text }));
    if (control instanceof HTMLInputElement && control.list) field.options = [...control.list.options].map(({ value, text }) => ({ value, text: text || value }));
    if (kind === "spin") Object.assign(field, { min: control.getAttribute("min"), max: control.getAttribute("max") });
    field.value = control.type === "checkbox" ? String((control as HTMLInputElement).checked) : control.value;
    return field;
  }));

/** Visits every page of a tab in every mode: listed pages, groups, controls and metadata per PrintConfigDef. */
async function checkTab(page: Page, scope: Locator, scroller: Locator, name: keyof typeof EXPERT_TITLES, layouts: LayoutPage[]) {
  expect(layouts.filter((layout) => pageVisible(defs, layout, "expert")).map((layout) => layout.title)).toEqual(EXPERT_TITLES[name]);
  const coverage: Record<string, Record<string, number>> = {};
  for (const mode of MODES) {
    // One widget; a click picks the mode under its x third (ModeSwitchButton::hit_test_selection).
    const switcher = scope.getByRole("slider", { name: "Settings mode" });
    const { width, height } = (await switcher.boundingBox())!;
    await switcher.click({ position: { x: (width * (2 * MODES.indexOf(mode) + 1)) / 6, y: height / 2 } });
    await expect(switcher).toHaveAttribute("aria-valuetext", MODE_NAMES[mode]);
    const pages = layouts.filter((layout) => pageVisible(defs, layout, mode));
    await expect(scope.getByRole("tab")).toHaveText(pages.map((layout) => layout.title));
    coverage[mode] = {};
    for (const [index, layout] of pages.entries()) {
      await scope.getByRole("tab", { name: layout.title, exact: true }).click();
      await expect(scope.getByRole("tab", { name: layout.title, exact: true })).toHaveAttribute("aria-selected", "true");
      const expected = expectedPage(defs, layout, mode, config, TYPED[name], name === "filament" ? 0 : undefined);
      await expect(scope.locator(".optgroup-title")).toHaveText(expected.groups);
      const fields = await renderedFields(scope);
      // Values are checked where the oracle knows them (all but untyped whole-vector texts).
      const comparable = fields.map((field, at) => (expected.fields[at]?.value === undefined ? { ...field, value: undefined } : field));
      expect(comparable, `${name} / ${mode} / ${layout.title}`).toEqual(expected.fields.map((field) => ({ value: undefined, ...field })));
      coverage[mode][layout.title] = fields.length;
      await fullShot(page, name === "process" ? scroller : scope, scroller, `${artifacts}${name}-${mode}-${index + 1}-${slug(layout.title)}.png`);
    }
  }
  return coverage;
}

/** Commits `text` into a field like typing then leaving it (`EVT_KILL_FOCUS`). */
async function commit(field: Locator, text: string) {
  await field.fill(text);
  await field.press("Enter");
}

test("OrcaSlicer settings tabs render PrintConfigDef metadata per mode; unedited slice matches native bytes", async ({ page }) => {
  test.setTimeout(240_000);
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  // OrcaSlicer starts in simple mode (MainFrame.cpp:325-326, GUI_App::get_saved_mode).
  const sidebar = page.getByTestId("sidebar");
  const modes = sidebar.getByRole("slider", { name: "Settings mode" });
  await expect(modes).toHaveAttribute("aria-valuetext", "Simple settings");
  // WCAG 2.5.8: every sidebar target, the mode switch included, is at least 24x24 CSS px.
  const { violations } = await new AxeBuilder({ page }).include("[data-testid=sidebar]").withRules(["target-size"]).analyze();
  expect(violations.flatMap((rule) => rule.nodes.map((node) => node.target.join(" "))), "undersized sidebar targets").toEqual([]);
  const process = page.getByTestId("process-settings");
  const processCoverage = await checkTab(page, sidebar, sidebar, "process", PROCESS_PAGES);

  // Tab.cpp:2528-2541: one "Overhang speed" line with four options, a separator, then the "Bridge" line.
  await process.getByRole("tab", { name: "Speed", exact: true }).click();
  const overhang = process.getByTestId("line-Overhang speed");
  await expect(overhang.locator("[data-option]")).toHaveCount(4);
  await expect(overhang.locator("[data-option]").first()).toHaveAttribute("data-option", "overhang_1_4_speed");
  await expect(process.locator(".optgroup-separator")).toHaveCount(1);
  await expect(process.getByTestId("line-Bridge").locator("[data-option]")).toHaveCount(2);

  // Field::get_value_by_opt_type (Field.cpp:313-450) when a value is committed: parse, clamp, show_error.
  await process.getByRole("tab", { name: "Quality", exact: true }).click();
  const layerHeight = process.getByRole("textbox", { name: "layer_height", exact: true });
  await commit(layerHeight, "abc");
  await expect(process.getByRole("alert")).toHaveText("Invalid numeric.");
  await expect(layerHeight).toHaveValue("0");
  await commit(layerHeight, "0.3");
  await expect(process.getByRole("alert")).toHaveCount(0);
  await commit(layerHeight, "0.3%");
  await expect(process.getByRole("alert")).toHaveText("Layer height can't be a percentage");
  await expect(layerHeight).toHaveValue("0");
  await commit(layerHeight, "-1");
  await expect(process.getByRole("alert")).toHaveText("Value is out of range.");
  await expect(layerHeight).toHaveValue("0");
  const generator = process.locator("select[data-option=wall_generator]");
  await generator.selectOption("classic");
  await expect(generator).toHaveValue("classic");
  await process.getByRole("tab", { name: "Strength", exact: true }).click();
  const walls = process.getByRole("spinbutton", { name: "wall_loops", exact: true });
  await commit(walls, "1001");
  await expect(process.getByRole("alert")).toHaveText("Value is out of range.");
  await expect(walls).toHaveValue("1000");
  const density = process.getByRole("textbox", { name: "sparse_infill_density", exact: true });
  await commit(density, "150%");
  await expect(process.getByRole("alert")).toHaveText("Value is out of range.");
  await expect(density).toHaveValue("100");
  const anchor = process.getByRole("combobox", { name: "infill_anchor", exact: true });
  await expect(anchor).toHaveValue("400%");
  await commit(anchor, "1000 (unlimited)");
  await expect(anchor).toHaveValue("1000 (unlimited)");
  await expect(process.getByRole("alert")).toHaveCount(0);
  // CheckBox (Field.cpp:1059-1088): toggling commits the bool.
  const thinWalls = process.getByRole("checkbox", { name: "detect_thin_wall", exact: true });
  await expect(thinWalls).toBeChecked();
  await thinWalls.click();
  await expect(thinWalls).not.toBeChecked();
  await expect(page.getByTestId("process-edited")).toContainText("6 modified options");
  await anchor.scrollIntoViewIfNeeded();
  await sidebar.screenshot({ path: `${artifacts}process-validation.png` });
  await page.getByTestId("process-edited").getByRole("button", { name: "Reset all" }).click();
  await expect(page.getByTestId("process-edited")).toHaveCount(0);

  await page.getByRole("button", { name: "Edit printer settings" }).click();
  const printer = page.getByRole("dialog", { name: "Printer settings" });
  const printerCoverage = await checkTab(page, printer, printer.locator(".settings-body"), "printer", PRINTER_PAGES);
  await expect(printer.locator("input:not([readonly]):not([disabled]), textarea:not([readonly]), select:not([disabled])")).toHaveCount(0);
  await printer.getByRole("tab", { name: "Machine G-code", exact: true }).click();
  await expect(printer.locator("textarea[data-option=machine_start_gcode]")).toHaveValue(/G28/);
  await printer.getByRole("button", { name: "Close" }).click();

  await page.getByRole("button", { name: "Edit filament 1 settings" }).click();
  const filament = page.getByRole("dialog", { name: "Filament settings" });
  const filamentCoverage = await checkTab(page, filament, filament.locator(".settings-body"), "filament", FILAMENT_PAGES);
  await expect(filament.locator("input:not([readonly]):not([disabled]), textarea:not([readonly]), select:not([disabled])")).toHaveCount(0);
  await filament.getByRole("tab", { name: "Filament", exact: true }).click();
  // Tab.cpp:3981-3982: "Print temperature" is one "Nozzle" line (initial layer, other layers).
  await expect(filament.getByTestId("line-Nozzle").locator("[data-option]")).toHaveCount(2);
  await filament.getByRole("button", { name: "Close" }).click();

  // The mode is one app setting (GUI_App::save_mode), shared by the sidebar and the dialogs.
  await expect(modes).toHaveAttribute("aria-valuetext", "Expert settings");
  // The keyboard steps the same widget (slider keys).
  await modes.press("ArrowLeft");
  await expect(modes).toHaveAttribute("aria-valuetext", "Advanced settings");
  await modes.press("End");
  await expect(modes).toHaveAttribute("aria-valuetext", "Expert settings");

  // Browsing settings must not change the slice: bytes equal native ares-cli.
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  const gcodePath = `${artifacts}${download.suggestedFilename()}`;
  await download.saveAs(gcodePath);
  const native = `${artifacts}native.gcode`;
  execFileSync(`${repositoryRoot}target/release/ares`, ["slice", "--plate", "1", "-o", native, fixture]);
  const browserHash = sha256(readFileSync(gcodePath));
  expect(browserHash, "browser G-code must match native ares-cli bytes").toBe(sha256(readFileSync(native)));

  writeFileSync(
    `${artifacts}result.json`,
    `${JSON.stringify({ fixture: "tests/parity/orca_cli_ender3.3mf", sha256: browserHash, fields: { process: processCoverage, printer: printerCoverage, filament: filamentCoverage } }, null, 2)}\n`,
  );
  expect(errors).toEqual([]);
});
