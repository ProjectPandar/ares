import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { guiExport, matrix, orcaPlate, probedTowers } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-scale/", import.meta.url));
// Ender-3 (220 mm bed): cube10 object 2 at the bed centre (110, 110), its centre at z 5.
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";

type Axis = "X" | "Y" | "Z";
const field = (page: Page, group: string, axis: Axis) => page.getByRole("textbox", { name: `${group} ${axis}`, exact: true });

async function setField(page: Page, group: string, axis: Axis, value: string) {
  await field(page, group, axis).fill(value);
  await field(page, group, axis).press("Enter");
}

async function expectFields(page: Page, group: string, values: readonly [string, string, string]) {
  for (const [index, axis] of (["X", "Y", "Z"] as const).entries()) await expect(field(page, group, axis)).toHaveValue(values[index]);
}

test("the Scale and Size fields scale an instance about its centre, drop it onto the bed, and slice like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}cube.3mf`;
  copyFileSync(`${repositoryRoot}${FIXTURE}`, project);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await page.getByTestId("object-list").getByRole("treeitem").nth(1).click();
  await expectFields(page, "Scale", ["100.00", "100.00", "100.00"]);
  await expectFields(page, "Size", ["10.00", "10.00", "10.00"]);
  await expect(page.getByRole("checkbox", { name: "Uniform scale" })).toBeChecked();
  await expect(page.getByRole("button", { name: "Reset scale" })).toHaveCount(0);

  // Uniform scale (the default): Scale X 120 scales every axis about the cube's centre; the 12 mm
  // cube's bottom at z -1 drops back onto the bed (GLCanvas3D::do_scale).
  await setField(page, "Scale", "X", "120");
  await expectFields(page, "Scale", ["120.00", "120.00", "120.00"]);
  await expectFields(page, "Size", ["12.00", "12.00", "12.00"]);
  await expectFields(page, "Position", ["110.00", "110.00", "6.00"]);
  await page.screenshot({ path: `${artifacts}uniform.png` });

  // reset_scale_value: back to 100 % and onto the bed again.
  await page.getByRole("button", { name: "Reset scale" }).click();
  await expectFields(page, "Scale", ["100.00", "100.00", "100.00"]);
  await expectFields(page, "Position", ["110.00", "110.00", "5.00"]);
  await expect(page.getByRole("button", { name: "Reset scale" })).toHaveCount(0);

  // Without the uniform toggle, Size X 12 scales X alone; the bottom stays on the bed.
  await page.getByRole("checkbox", { name: "Uniform scale" }).uncheck();
  await setField(page, "Size", "X", "12");
  await expectFields(page, "Size", ["12.00", "10.00", "10.00"]);
  await expectFields(page, "Scale", ["120.00", "100.00", "100.00"]);
  await expectFields(page, "Position", ["110.00", "110.00", "5.00"]);
  await page.screenshot({ path: `${artifacts}size-x.png` });

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}scaled.3mf`;
  const scaleX = [[1.2, 0, 0], [0, 1, 0], [0, 0, 1]];
  writeFileSync(oracle, guiExport(FIXTURE, [
    { source: 0, items: [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: matrix([scaleX], [110, 110, 5]) }] },
  ], await probedTowers(page)));
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["Scale X 120 (uniform)", "Reset scale", "Uniform scale off", "Size X 12"],
    oracle: "OrcaSlicer --slice 1 of the GUI-exported project (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
