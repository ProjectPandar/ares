import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { guiExport, matrix, orcaPlate, probedTowers } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-scale-to-fit/", import.meta.url));
// Ender-3 (220 x 220 x 250 mm bed): cube10 object 2 at the bed centre (110, 110), its centre at z 5.
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";
/** `BuildVolume::SceneEpsilon`. */
const EPS = 1e-4;

type Axis = "X" | "Y" | "Z";
const field = (page: Page, group: string, axis: Axis) => page.getByRole("textbox", { name: `${group} ${axis}`, exact: true });

async function expectFields(page: Page, group: string, values: readonly [string, string, string]) {
  for (const [index, axis] of (["X", "Y", "Z"] as const).entries()) await expect(field(page, group, axis)).toHaveValue(values[index]);
}

test("Scale to build volume fills the printable height on the plate centre, and slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(600_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}cube.3mf`;
  copyFileSync(`${repositoryRoot}${FIXTURE}`, project);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
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
  // fit_rectangle (Selection.cpp:1511-1532): s = (250 + 2 eps) / (200 + 0.02), centred on the bed box
  // and standing on the bed.
  const s = (250 + EPS - -EPS) / 200.02;
  await expectFields(page, "Size", [(10 * s).toFixed(2), (10 * s).toFixed(2), (200 * s).toFixed(2)]);
  await expectFields(page, "Position", ["110.00", "110.00", (100 * s).toFixed(2)]);
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
  writeFileSync(oracle, guiExport(FIXTURE, [
    { source: 0, items: [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: matrix([fitted], [110, 110, 100 * s]) }] },
  ], await probedTowers(page)));
  const legs = { official: orcaPlate(oracle, 1, "official", project), seeded: orcaPlate(oracle, 1, "seeded", project) };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["Uniform scale off", "Size Z 200", "Object menu: Scale to build volume"],
    oracle: "OrcaSlicer --slice 1 of the GUI-exported project with the upstream fit's transform (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
