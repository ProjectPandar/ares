import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { guiExport, matrix, orcaPlate, probedTowers } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-move/", import.meta.url));
// Ender-3 (220 mm bed): cube10 object 2 at the bed centre (110, 110), its centre at z 5.
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";

const position = (page: Page, axis: "X" | "Y" | "Z") => page.getByRole("textbox", { name: `Position ${axis}`, exact: true });

async function setPosition(page: Page, axis: "X" | "Y" | "Z", value: string) {
  await position(page, axis).fill(value);
  await position(page, axis).press("Enter");
}

test("an instance moves by drag and Position fields, drops onto the bed, and slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}cube.3mf`;
  copyFileSync(`${repositoryRoot}${FIXTURE}`, project);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  // Selecting the object shows its instance offset in world coordinates.
  await page.getByTestId("object-list").getByRole("treeitem").nth(1).click();
  await expect(position(page, "X")).toHaveValue("110.00");
  await expect(position(page, "Y")).toHaveValue("110.00");
  await expect(position(page, "Z")).toHaveValue("5.00");

  // GLCanvas3D::on_mouse: a press on the cube at the plate view's centre and a horizontal drag
  // move it along X on the plane through the pressed point.
  const canvas = page.getByTestId("viewport").locator("canvas");
  const box = (await canvas.boundingBox())!;
  const [x, y] = [box.x + box.width / 2, box.y + box.height / 2];
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + 120, y, { steps: 10 });
  await page.mouse.up();
  await expect.poll(async () => Number(await position(page, "X").inputValue())).toBeGreaterThan(115);
  await expect(position(page, "Y")).toHaveValue("110.00");
  await expect(position(page, "Z")).toHaveValue("5.00");
  await page.screenshot({ path: `${artifacts}dragged.png` });

  // GLGizmoMove3D, opened from the gizmo toolbar: a drag on the X grabber, along the arrow on
  // screen, moves the instance along X only.
  const dragged = await position(page, "X").inputValue();
  await page.getByRole("toolbar", { name: "Gizmos" }).getByRole("button", { name: "Move", exact: true }).click();
  const gizmo = JSON.parse((await page.getByTestId("viewport").getAttribute("data-gizmo"))!) as Record<"center" | "x", [number, number]>;
  await page.mouse.move(box.x + gizmo.x[0], box.y + gizmo.x[1]);
  await page.mouse.down();
  await page.mouse.move(box.x + 2 * gizmo.x[0] - gizmo.center[0], box.y + 2 * gizmo.x[1] - gizmo.center[1], { steps: 10 });
  await page.mouse.up();
  await expect(position(page, "X")).not.toHaveValue(dragged);
  await expect(position(page, "Y")).toHaveValue("110.00");
  await expect(position(page, "Z")).toHaveValue("5.00");
  await page.screenshot({ path: `${artifacts}gizmo-x.png` });

  // Position fields set world X/Y; a raised Z drops back onto the bed (GLCanvas3D::do_move).
  await setPosition(page, "X", "145.5");
  await expect(position(page, "X")).toHaveValue("145.50");
  await setPosition(page, "Y", "95.5");
  await expect(position(page, "Y")).toHaveValue("95.50");
  await setPosition(page, "Z", "12");
  await expect(position(page, "Z")).toHaveValue("5.00");
  await page.screenshot({ path: `${artifacts}moved.png` });

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}moved.3mf`;
  writeFileSync(oracle, guiExport(FIXTURE, [
    { source: 0, items: [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: matrix([], [145.5, 95.5, 5]) }] },
  ], await probedTowers(page)));
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["drag along X", "move gizmo X arrow", "Position X 145.5", "Position Y 95.5", "Position Z 12 drops to 5"],
    oracle: "OrcaSlicer 2.4.2 --slice 1 of the GUI-exported project (official and seed-replay seed 0)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
