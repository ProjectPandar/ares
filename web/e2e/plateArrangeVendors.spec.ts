import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { type Edit, editedProject } from "./arrangeOracle";
import { test } from "./failureDiagnostics";

const artifacts = fileURLToPath(new URL("../../target/web-e2e/arrange-vendors/", import.meta.url));
/** A Bambu Lab X2D (two nozzles, automatic filament map) with one object on filament 1 at (138.25, 128). */
const X2D = "tests/parity/generated-printer-ids/input.3mf";
const PROJECT = "Metadata/project_settings.config";
/**
 * PerExtruder mode with a full-height volume over the whole bed for extruder 2 only, and one area for
 * both nozzles (the arrange of several nozzles' shared area is not ported; the vendor test reads only
 * `printer_model`).
 */
const EDITS: Edit[] = [
  [
    PROJECT,
    "{\n  \"accel_to_decel_enable\"",
    "{\n  \"bed_exclude_volume_mode\": \"per_extruder\",\n  \"extruder_bed_exclude_volumes\": [\"\", \"0..250;-10x-10,266x-10,266x266,-10x266\"],\n  \"accel_to_decel_enable\"",
  ],
  [
    PROJECT,
    "\"extruder_printable_area\": [\n    \"0x0,256x0,256x256,0x256\",\n    \"20.5x0,256x0,256x256,20.5x256\"\n  ]",
    "\"extruder_printable_area\": []",
  ],
];

async function systemPrinters(page: Page) {
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  const listbox = page.getByRole("listbox", { name: "Printer preset", exact: true });
  const names = await listbox.getByRole("group", { name: "System presets" }).getByRole("option").allTextContents();
  await page.keyboard.press("Escape");
  return names;
}

/** Checks or unchecks the Bambulab vendor in the "System presets" wizard (ConfigWizard). */
async function installBbl(page: Page, installed: boolean) {
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).setChecked(installed);
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  // The printer list follows the catalog sync, which hands the plater the installed vendors first.
  await expect.poll(async () => (await systemPrinters(page)).includes("Bambu Lab X2D"), { timeout: 60_000 }).toBe(installed);
}

/** "Arrange all" with the popup's defaults, then the first instance's XY offset. */
async function arrangeAll(page: Page): Promise<[number, number]> {
  const popup = page.getByRole("dialog", { name: "Arrange settings" });
  // The popup stays open after an arrange.
  if (!(await popup.isVisible())) await page.getByRole("button", { name: "Arrange", exact: true }).click();
  await popup.getByRole("button", { name: "Reset" }).click();
  await popup.getByRole("button", { name: "Arrange", exact: true }).click();
  await expect(page.getByTestId("status")).not.toContainText("Arranging", { timeout: 60_000 });
  await expect(page.getByTestId("status")).not.toContainText("Arrange failed");
  const probe = JSON.parse((await page.locator(".viewport").getAttribute("data-instances"))!) as {
    objects: { instances: { matrix: number[] }[] }[];
  };
  const { matrix } = probe.objects[0].instances[0];
  return [matrix[12], matrix[13]];
}

test("the arrange reads the printer's vendor from the installed vendors, as PresetBundle does", async ({ page }) => {
  test.setTimeout(300_000);
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}x2d.3mf`;
  writeFileSync(project, editedProject(X2D, EDITS));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });

  // BBL installed before the project opens: the new plater receives it. Marlin_BBL under an automatic
  // map leaves the object's nozzle ids empty (ArrangeJob.cpp:73-75), so the extruder-2 volume applies
  // everywhere and the object stays unarranged where it was.
  await installBbl(page, true);
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByRole("button", { name: "Arrange", exact: true })).toBeVisible({ timeout: 60_000 });
  const withBbl = await arrangeAll(page);
  expect(withBbl).toEqual([138.25, 128]);
  await page.screenshot({ path: `${artifacts}with-bbl.png` });

  // BBL uninstalled: the catalog sync hands the plater the new set, the vendor is Unknown, the generic
  // automatic map puts filament 1 on extruder 1 (PrintConfig.cpp:14201-14202) and the object packs.
  await installBbl(page, false);
  const withoutBbl = await arrangeAll(page);
  expect(withoutBbl).not.toEqual([138.25, 128]);
  await page.screenshot({ path: `${artifacts}without-bbl.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: X2D,
    edits: ["PerExtruder, extruder-2 volume over the whole bed", "one nozzle area"],
    withBbl: { offset: withBbl, arranged: false },
    withoutBbl: { offset: withoutBbl, arranged: true },
  }, null, 2)}\n`);
});

test("the arrange keeps a first-layer-scanning Bambu Lab printer's calibration region clear", async ({ page }) => {
  test.setTimeout(300_000);
  mkdirSync(artifacts, { recursive: true });
  // The X2D scanning its first layer, its cube scaled to 230 mm deep: centred it spans Y 13..243, into
  // the (18, 0)-(240, 15) region ArrangeJob::process adds for a Bambu Lab vendor (ArrangeJob.cpp:732-734).
  const project = `${artifacts}x2d-scan.3mf`;
  writeFileSync(project, editedProject(X2D, [
    ["3D/3dmodel.model", "transform=\"1 0 0 0 1 0 0 0 1 138.25 128 5\"", "transform=\"1 0 0 0 23 0 0 0 1 138.25 128 5\""],
    [PROJECT, "\"scan_first_layer\": \"0\"", "\"scan_first_layer\": \"1\""],
  ]));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await installBbl(page, true);
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByRole("button", { name: "Arrange", exact: true })).toBeVisible({ timeout: 60_000 });

  const popup = page.getByRole("dialog", { name: "Arrange settings" });
  const avoid = popup.getByRole("checkbox", { name: "Avoid extrusion calibration region" });
  /** "Arrange all" with the calibration region avoided or not, then the cube's front edge. */
  async function frontEdge(avoided: boolean) {
    if (!(await popup.isVisible())) await page.getByRole("button", { name: "Arrange", exact: true }).click();
    await popup.getByRole("button", { name: "Reset" }).click();
    if ((await avoid.isChecked()) !== avoided) await avoid.click();
    await expect(avoid).toBeChecked({ checked: avoided });
    await popup.getByRole("button", { name: "Arrange", exact: true }).click();
    await expect(page.getByTestId("status")).not.toContainText("Arranging", { timeout: 60_000 });
    await expect(page.getByTestId("status")).not.toContainText("Arrange failed");
    const probe = JSON.parse((await page.locator(".viewport").getAttribute("data-instances"))!) as {
      objects: { instances: { matrix: number[] }[] }[];
    };
    return probe.objects[0].instances[0].matrix[13] - 115;
  }
  expect(await frontEdge(true)).toBeGreaterThanOrEqual(15);
  await page.screenshot({ path: `${artifacts}scan-region.png` });
  // Control: without "avoid" there is no region and the cube stays centred, in front of Y 15.
  expect(await frontEdge(false)).toBeLessThan(15);
});
