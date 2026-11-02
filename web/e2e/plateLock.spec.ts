import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { appArranged, type Edit, editedProject, orcaArrange } from "./arrangeOracle";
import { test } from "./failureDiagnostics";

const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-lock/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates. */
const TWO_PLATES = "tests/parity/custom-gcode-overview/two-plates.3mf";
/** Plate 1's cube off the plate centre, so an arrange of it shows. */
const MOVED: Edit = ["3D/3dmodel.model", "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"", "transform=\"1 0 0 0 1 0 0 0 1 40 40 5\""];
/** Plate 1 locked in the 3MF (`bbs_3mf.cpp:4637-4640`), as the lock icon leaves it. */
const LOCKED: Edit = [
  "Metadata/model_settings.config",
  "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>",
  "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"true\"/>",
];
/** The settings the CLI flags give: no rotation, no calibration region, no align to Y. */
const FLAGS = ["--allow-rotations=0", "--avoid-extrusion-cali-region=0", "--align-to-y-axis=0"];

const popup = (page: Page) => page.getByRole("dialog", { name: "Arrange settings" });

test("a locked plate keeps its objects when arranging, like the OrcaSlicer v2.5.0-beta CLI", async ({ page }) => {
  test.setTimeout(300_000);
  mkdirSync(artifacts, { recursive: true });
  const project = `${artifacts}unlocked.3mf`;
  writeFileSync(project, editedProject(TWO_PLATES, [MOVED]));
  // Native arranges the project as the app holds it after the lock: plate 1 locked.
  const locked = `${artifacts}locked.3mf`;
  writeFileSync(locked, editedProject(TWO_PLATES, [MOVED, LOCKED]));
  const native = orcaArrange(locked, FLAGS);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  const lock = page.getByRole("button", { name: "Lock plate 1" });
  await expect(lock).toHaveAttribute("aria-pressed", "false", { timeout: 60_000 });
  await expect(lock).toHaveAttribute("title", "Lock current plate");
  await lock.click();
  await expect(lock).toHaveAttribute("aria-pressed", "true");
  await expect(lock).toHaveAttribute("title", "Unlock current plate");

  // "Arrange all" with the CLI's settings.
  await page.getByRole("button", { name: "Arrange", exact: true }).click();
  await popup(page).getByRole("button", { name: "Reset" }).click();
  await popup(page).getByRole("checkbox", { name: "Align to Y axis" }).click();
  await expect(popup(page).getByRole("checkbox", { name: "Align to Y axis" })).not.toBeChecked();
  await popup(page).getByRole("button", { name: "Arrange", exact: true }).click();
  await expect(page.getByTestId("status")).not.toContainText("Arranging", { timeout: 60_000 });
  await expect(page.getByTestId("status")).not.toContainText("Arrange failed");
  await page.screenshot({ path: `${artifacts}arranged.png` });
  if (native) expect(await appArranged(page)).toEqual(native);

  // The plate's own arrange icon on a locked plate (`ArrangeJob.cpp:548-552`).
  const before = await appArranged(page);
  await page.getByRole("button", { name: "Arrange plate 1" }).click();
  await expect(page.getByTestId("status")).toContainText("This plate is locked.");
  expect(await appArranged(page)).toEqual(before);
  await page.screenshot({ path: `${artifacts}locked-arrange.png` });
});
