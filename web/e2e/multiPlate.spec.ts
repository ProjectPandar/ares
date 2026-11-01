import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { guiExport, matrix, orcaPlate, probedTowers } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/multi-plate/", import.meta.url));
// Ender-3 (220 mm bed): cube10 object 2 on plate 1, object 3 on plate 2, per-plate custom G-codes.
const FIXTURE = "tests/parity/custom-gcode-overview/two-plates.3mf";
/** `plate_stride_x/y`: the bed and its 1/5 gap (PartPlate.cpp:4836-4849). */
const STRIDE = 220 * 1.2;

const plateRow = (page: Page, id: number) => page.getByRole("treeitem", { name: new RegExp(`^Plate ${id}\\b`) });
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
/** The notebook's SideButtons. */
const action = (page: Page, name: string) => page.getByRole("navigation").getByRole("button", { name, exact: true });

/** Plate origins and the current plate as the canvas draws them; `-0` reads as `0`. */
async function canvasPlates(page: Page) {
  const probe = JSON.parse((await page.getByTestId("viewport").getAttribute("data-plates"))!) as { origins: number[][]; current: number };
  return { origins: probe.origins.map((origin) => origin.map((value) => value + 0)), current: probe.current };
}

async function slice(page: Page, name: string) {
  await action(page, "Slice plate").click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  return exportCurrent(page, name);
}

async function exportCurrent(page: Page, name: string) {
  const [download] = await Promise.all([page.waitForEvent("download"), action(page, "Export G-code file").click()]);
  expect(download.suggestedFilename()).toBe(name);
  await download.saveAs(`${artifacts}${name}`);
  return readFileSync(`${artifacts}${name}`, "utf8");
}

test("plates render on the grid, slice one at a time or all, and follow add and delete like OrcaSlicer", async ({ page }) => {
  test.setTimeout(400_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}two-plates.3mf`;
  copyFileSync(`${repositoryRoot}${FIXTURE}`, project);
  const legs = ["official", "seeded"] as const;
  const results: Record<string, Record<string, string>> = {};
  const compare = (label: string, gcode: string, oracle: string, plate: number) => {
    results[label] = {};
    for (const leg of legs) {
      const native = orcaPlate(oracle, plate, leg);
      results[label][leg] = native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL";
    }
  };

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(plateRow(page, 2)).toBeVisible({ timeout: 60_000 });

  // PartPlateList::render: both plates on the 2-column grid, plate 1 current.
  expect(await canvasPlates(page)).toEqual({ origins: [[0, 0], [STRIDE, 0]], current: 0 });
  await expect(plateRow(page, 1)).toHaveAttribute("aria-selected", "true");
  await page.screenshot({ path: `${artifacts}plates.png` });

  // "Slice plate" slices the current plate; selecting plate 2 makes it current.
  const plate1 = await slice(page, "two-plates_plate_1.gcode");
  compare("plate 1", plate1, project, 1);
  await plateRow(page, 2).click();
  expect(await canvasPlates(page)).toEqual({ origins: [[0, 0], [STRIDE, 0]], current: 1 });
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("slice-summary")).toContainText("Plate not sliced");
  const plate2 = await slice(page, "two-plates_plate_2.gcode");
  await expect(page.getByTestId("slice-plate")).toHaveText("2");
  compare("plate 2", plate2, project, 2);
  // Back on plate 1 the preview shows its kept slice (PartPlate::m_gcode_result).
  await page.getByRole("link", { name: "Prepare" }).click();
  await plateRow(page, 1).click();
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("slice-plate")).toHaveText("1");
  await expect(page.getByTestId("gcode-bytes")).toHaveText(`${Buffer.byteLength(plate1).toLocaleString("en-US")} bytes`);
  await page.screenshot({ path: `${artifacts}preview-plate-1.png` });

  // "Slice all" slices every plate in turn and ends on the last; "Export all" saves each plate.
  await action(page, "Slice all").click();
  await expect(page.getByTestId("slice-plate")).toHaveText("2", { timeout: 120_000 });
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const downloads: string[] = [];
  page.on("download", (download) => downloads.push(download.suggestedFilename()));
  await action(page, "Export all").click();
  await expect.poll(() => downloads.sort()).toEqual(["two-plates_plate_1.gcode", "two-plates_plate_2.gcode"]);
  expect(await exportCurrent(page, "two-plates_plate_2.gcode")).toBe(plate2);

  // Three added plates make 5: the grid grows to 3 columns (compute_colum_count) and the new
  // plate is current. Deleting plate 1 moves plate 2 and its object up to plate 1's origin, and
  // plate 1's object to the unprintable plate origin: with 4 plates in 2 columns,
  // compute_origin(4 + 2 - 1, 3) = (2, -1) strides (PartPlate.cpp:3942-3949, 4598-4600).
  await page.getByRole("link", { name: "Prepare" }).click();
  for (let count = 0; count < 3; count++) await button(page, "Add plate").click();
  await expect(plateRow(page, 5)).toBeVisible();
  expect(await canvasPlates(page)).toEqual({ origins: [[0, 0], [STRIDE, 0], [2 * STRIDE, 0], [0, -STRIDE], [STRIDE, -STRIDE]], current: 4 });
  await button(page, "Delete plate 1").click();
  await expect(plateRow(page, 5)).toHaveCount(0);
  expect(await canvasPlates(page)).toEqual({ origins: [[0, 0], [STRIDE, 0], [0, -STRIDE], [STRIDE, -STRIDE]], current: 3 });
  await plateRow(page, 1).click();
  const tree = await page.getByTestId("object-list").getByRole("treeitem").evaluateAll((rows) =>
    rows.map((row) => (row.classList.contains("plate-row") ? row.querySelector(".object-name")!.textContent : "on plate")));
  expect(tree).toEqual(["Plate 1", "on plate", "Plate 2", "Plate 3", "Plate 4", "Outside", "on plate"]);
  await page.screenshot({ path: `${artifacts}after-delete.png` });
  const moved = await slice(page, "two-plates_plate_1.gcode");
  const oracle = `${artifacts}after-delete.3mf`;
  writeFileSync(oracle, guiExport(FIXTURE, [
    { source: 1, items: [{ objectId: 3, instanceId: 0, identifyId: 15, matrix: matrix([], [110, 110, 5]) }] },
    { source: null, items: [] }, { source: null, items: [] }, { source: null, items: [] },
  ], await probedTowers(page), [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: matrix([], [110 + 2 * STRIDE, 110 - STRIDE, 5]) }]));
  compare("after delete, plate 1", moved, oracle, 1);

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    oracle: "OrcaSlicer 2.4.2 --slice <plate> (official and seed-replay seed 0); after the delete, of the GUI-exported project",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [label, byLeg] of Object.entries(results)) {
    for (const [leg, outcome] of Object.entries(byLeg)) expect(outcome, `${label} vs ${leg} OrcaSlicer`).not.toBe("FAIL");
  }
});
