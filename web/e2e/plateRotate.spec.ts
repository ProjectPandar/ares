import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { aboutX, aboutZ, guiExport, matrix, orcaPlate } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-rotate/", import.meta.url));
// Ender-3 (220 mm bed): cube10 object 2 at the bed centre (110, 110), its centre at z 5.
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";

const field = (page: Page, group: string, axis: "X" | "Y" | "Z") => page.getByRole("textbox", { name: `${group} ${axis}`, exact: true });

async function setField(page: Page, group: string, axis: "X" | "Y" | "Z", value: string) {
  await field(page, group, axis).fill(value);
  await field(page, group, axis).press("Enter");
}

async function importCube(page: Page, project: string) {
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await page.getByTestId("object-list").getByRole("treeitem").nth(1).click();
  await expect(field(page, "Rotate (absolute)", "Z")).toHaveValue("0.00");
}

test("an instance turns by the rotate gizmo ring and rotation fields, drops onto the bed, and slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}cube.3mf`;
  copyFileSync(`${repositoryRoot}${FIXTURE}`, project);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await importCube(page, project);

  // GLGizmoRotate3D: sweeping the Z ring an eighth of a turn round the gizmo centre on screen turns
  // the cube counter-clockwise about Z through its centre, leaving its offset and the other angles
  // alone. (A clockwise turn reads as X 180, Y 180: get_rotation_by_quaternion, Geometry.cpp:481-489.)
  await page.getByRole("toolbar", { name: "Gizmos" }).getByRole("button", { name: "Rotate" }).click();
  const box = (await page.getByTestId("viewport").locator("canvas").boundingBox())!;
  const gizmo = JSON.parse((await page.getByTestId("viewport").getAttribute("data-gizmo"))!) as Record<"center" | "z", [number, number]>;
  const [cx, cy] = [box.x + gizmo.center[0], box.y + gizmo.center[1]];
  const [dx, dy] = [gizmo.z[0] - gizmo.center[0], gizmo.z[1] - gizmo.center[1]];
  await page.mouse.move(cx + dx, cy + dy);
  await page.mouse.down();
  await page.mouse.move(cx + Math.SQRT1_2 * (dx + dy), cy + Math.SQRT1_2 * (dy - dx), { steps: 5 });
  await page.mouse.up();
  await expect(field(page, "Rotate (absolute)", "Z")).not.toHaveValue("0.00");
  await expect(field(page, "Rotate (absolute)", "X")).toHaveValue("0.00");
  await expect(field(page, "Rotate (absolute)", "Y")).toHaveValue("0.00");
  await expect(field(page, "Position", "X")).toHaveValue("110.00");
  await expect(field(page, "Position", "Y")).toHaveValue("110.00");
  await expect(field(page, "Position", "Z")).toHaveValue("5.00");
  await page.screenshot({ path: `${artifacts}ring.png` });

  // From the loaded cube again: a relative Z turn of 45° and an absolute X of 90°; the turned cube
  // stands on an edge-free face, its half height now 5√2, and do_rotate drops it onto the bed.
  await importCube(page, project);
  await setField(page, "Rotate (relative)", "Z", "45");
  await expect(field(page, "Rotate (absolute)", "Z")).toHaveValue("45.00");
  await expect(field(page, "Rotate (relative)", "Z")).toHaveValue("0.00");
  await setField(page, "Rotate (absolute)", "X", "90");
  await expect(field(page, "Rotate (absolute)", "X")).toHaveValue("90.00");
  await expect(field(page, "Position", "Z")).toHaveValue("7.07");
  await page.screenshot({ path: `${artifacts}rotated.png` });

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}rotated.3mf`;
  writeFileSync(oracle, guiExport(FIXTURE, [
    { source: 0, items: [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: matrix([aboutZ(45), aboutX(90)], [110, 110, 5 * Math.SQRT2]) }] },
  ]));
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["rotate gizmo Z ring eighth sweep (fresh load after)", "Rotate (relative) Z 45", "Rotate (absolute) X 90"],
    oracle: "OrcaSlicer 2.4.2 --slice 1 of the GUI-exported project (official and seed-replay seed 0)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
