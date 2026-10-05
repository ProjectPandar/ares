import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Locator, type Page, test } from "@playwright/test";
import { FILAMENT_PAGES, PRINTER_PAGES } from "../app/src/sidebar/presetLayout";
import { PROCESS_PAGES } from "../app/src/sidebar/processLayout";
import type { LayoutPage } from "../app/src/sidebar/layoutTypes";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/settings/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const slug = (title: string) => title.toLowerCase().replace(/[^a-z0-9]+/g, "-");

// Page order of OrcaSlicer src/slic3r/GUI/Tab.cpp (pin 8500fcdc):
// TabPrint::build 2311-2800, TabFilament::build 3892-4243 (+ add_filament_overrides_page 3620),
// TabPrinter::build_fff 4457 + build_unregular_pages 4927 for a Marlin-flavour printer.
const PROCESS_TITLES = ["Quality", "Strength", "Speed", "Support", "Multimaterial", "Others"];
const FILAMENT_TITLES = ["Filament", "Cooling", "Setting Overrides", "Advanced", "Multimaterial", "Dependencies", "Notes"];
const PRINTER_TITLES = ["Basic information", "Machine G-code", "Multimaterial", "Extruder", "Motion ability", "Notes"];

const fieldsOf = (layout: LayoutPage) =>
  layout.groups.flatMap((group) => group.rows).reduce((count, row) => count + (row.kind === "line" ? row.options.length : row.kind === "separator" ? 0 : 1), 0);

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

/** Clicks through every page of a settings tab, screenshots it and returns the field coverage. */
async function visitPages(page: Page, scope: Locator, scroller: Locator, name: string, titles: string[], layouts: LayoutPage[]) {
  await expect(scope.getByRole("tab")).toHaveText(titles);
  const coverage: Record<string, { fields: number; layout: number }> = {};
  for (const [index, title] of titles.entries()) {
    await scope.getByRole("tab", { name: title, exact: true }).click();
    await expect(scope.getByRole("tab", { name: title, exact: true })).toHaveAttribute("aria-selected", "true");
    const groups = await scope.locator(".optgroup-title").allTextContents();
    const layoutGroups = layouts[index].groups.map((group) => group.title);
    // Groups without any option of this config are skipped, the rest keep Tab.cpp order.
    expect(groups.every((group, at) => layoutGroups.indexOf(group) > layoutGroups.indexOf(groups[at - 1] ?? ""))).toBe(true);
    coverage[title] = { fields: await scope.locator("[data-option]").count(), layout: fieldsOf(layouts[index]) };
    await fullShot(page, name === "process" ? scroller : scope, scroller, `${artifacts}${name}-${index + 1}-${slug(title)}.png`);
  }
  return coverage;
}

test("OrcaSlicer settings tabs: every page laid out per Tab.cpp, unedited slice matches native bytes", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  const process = page.getByTestId("process-settings");
  const processCoverage = await visitPages(page, process, page.getByTestId("sidebar"), "process", PROCESS_TITLES, PROCESS_PAGES);
  // Every TabPrint::build option exists in this project config.
  for (const { fields, layout } of Object.values(processCoverage)) expect(fields).toBe(layout);

  // Tab.cpp:2528-2541: one "Overhang speed" line with four options, a separator, then the "Bridge" line.
  await process.getByRole("tab", { name: "Speed", exact: true }).click();
  const overhang = process.getByTestId("line-Overhang speed");
  await expect(overhang.locator("[data-option]")).toHaveCount(4);
  await expect(overhang.locator("[data-option]").first()).toHaveAttribute("data-option", "overhang_1_4_speed");
  await expect(process.locator(".optgroup-separator")).toHaveCount(1);
  await expect(process.getByTestId("line-Bridge").locator("[data-option]")).toHaveCount(2);
  // Tab.cpp:2775-2779: code fields (`is_code`) are multi-line editors.
  await process.getByRole("tab", { name: "Others", exact: true }).click();
  await expect(process.locator("textarea[data-option=post_process]")).toBeVisible();

  await page.getByRole("button", { name: "Edit printer settings" }).click();
  const printer = page.getByRole("dialog", { name: "Printer settings" });
  const printerCoverage = await visitPages(page, printer, printer.locator(".settings-body"), "printer", PRINTER_TITLES, PRINTER_PAGES);
  await expect(printer.locator("input:not([readonly]):not([disabled]), textarea:not([readonly])")).toHaveCount(0);
  await printer.getByRole("tab", { name: "Machine G-code", exact: true }).click();
  await expect(printer.locator("textarea[data-option=machine_start_gcode]")).toHaveValue(/G28/);
  await printer.getByRole("button", { name: "Close" }).click();

  await page.getByRole("button", { name: "Edit filament 1 settings" }).click();
  const filament = page.getByRole("dialog", { name: "Filament settings" });
  const filamentCoverage = await visitPages(page, filament, filament.locator(".settings-body"), "filament", FILAMENT_TITLES, FILAMENT_PAGES);
  await expect(filament.locator("input:not([readonly]):not([disabled]), textarea:not([readonly])")).toHaveCount(0);
  await filament.getByRole("tab", { name: "Filament", exact: true }).click();
  // Tab.cpp:3981-3982: "Print temperature" is one "Nozzle" line (initial layer, other layers).
  await expect(filament.getByTestId("line-Nozzle").locator("[data-option]")).toHaveCount(2);
  await filament.getByRole("button", { name: "Close" }).click();

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
    `${JSON.stringify({ fixture: "tests/parity/orca_cli_ender3.3mf", sha256: browserHash, coverage: { process: processCoverage, printer: printerCoverage, filament: filamentCoverage } }, null, 2)}\n`,
  );
  expect(errors).toEqual([]);
});
