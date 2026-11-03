import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { editedProject } from "./arrangeOracle";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { guiExport, matrix, orcaPlate, probedTowers } from "./platerOracle";

const ARTIFACTS = "target/web-e2e/plate-scale-to-fit-circle/";
const artifacts = fileURLToPath(new URL(`../../${ARTIFACTS}`, import.meta.url));
// Ender-3 (250 mm printable height): cube10 object 2 at (110, 110), its centre at z 5.
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";
const PROJECT = "Metadata/project_settings.config";
const ENDER3_AREA = "\"printable_area\": [\n\t\t\"0x0\",\n\t\t\"220x0\",\n\t\t\"220x220\",\n\t\t\"0x220\"\n\t]";

/**
 * A circular bed (`BuildVolume.cpp:46-75`): the 36 integer points exactly on the circle of radius 65
 * around (110, 110), in angle order, so the bed's RANSAC circle is exactly that one.
 */
function circularArea() {
  const points: [number, number][] = [];
  for (let x = -65; x <= 65; x++) {
    const y = Math.round(Math.sqrt(65 * 65 - x * x));
    if (x * x + y * y === 65 * 65) points.push([x, y], [x, -y]);
  }
  const unique = [...new Map(points.map((point) => [point.join(), point])).values()];
  unique.sort((a, b) => Math.atan2(a[1], a[0]) - Math.atan2(b[1], b[0]));
  expect(unique).toHaveLength(36);
  return `"printable_area": [${unique.map(([x, y]) => `"${110 + x}x${110 + y}"`).join(", ")}]`;
}

type Axis = "X" | "Y" | "Z";
const field = (page: Page, group: string, axis: Axis) => page.getByRole("textbox", { name: `${group} ${axis}`, exact: true });

async function expectFields(page: Page, group: string, values: readonly [string, string, string]) {
  for (const [index, axis] of (["X", "Y", "Z"] as const).entries()) await expect(field(page, group, axis)).toHaveValue(values[index]);
}

test("Scale to build volume fills the printable height on a circular bed, and slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(600_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const circle = `${ARTIFACTS}circle.3mf`;
  writeFileSync(`${artifacts}circle.3mf`, editedProject(FIXTURE, [[PROJECT, ENDER3_AREA, circularArea()]]));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${artifacts}circle.3mf`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  const row = page.getByTestId("object-list").getByRole("treeitem").nth(1);
  await row.click();

  // A 200 mm tall cube, so the fit is limited by the printable height.
  await page.getByRole("checkbox", { name: "Uniform scale" }).uncheck();
  await field(page, "Size", "Z").fill("200");
  await field(page, "Size", "Z").press("Enter");
  await expectFields(page, "Size", ["10.00", "10.00", "200.00"]);

  await row.click({ button: "right" });
  await page.getByRole("menu", { name: "Object menu" }).getByRole("menuitem", { name: "Scale to build volume" }).click();
  // fit_circle (Selection.cpp:1534-1569): s = min(65 / circle radius, 250 / 200) = 1.25, centred on
  // the bed circle and standing on the bed.
  const s = 250 / 200;
  await expectFields(page, "Size", ["12.50", "12.50", "250.00"]);
  await expectFields(page, "Position", ["110.00", "110.00", "125.00"]);
  await page.screenshot({ path: `${artifacts}fitted.png` });

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 300_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}fitted.3mf`;
  const fitted = [[s, 0, 0], [0, s, 0], [0, 0, 20 * s]];
  writeFileSync(oracle, guiExport(circle, [
    { source: 0, items: [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: matrix([fitted], [110, 110, 100 * s]) }] },
  ], await probedTowers(page)));
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["printable_area: 36 points on the circle of radius 65 around (110, 110)", "Uniform scale off", "Size Z 200", "Object menu: Scale to build volume"],
    oracle: "OrcaSlicer --slice 1 of the GUI-exported project with the upstream fit's transform (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
