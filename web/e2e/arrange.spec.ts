import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { appArranged, type Edit, editedProject, orcaArrange } from "./arrangeOracle";
import { test } from "./failureDiagnostics";

const artifacts = fileURLToPath(new URL("../../target/web-e2e/arrange/", import.meta.url));
/** Two 10 mm cubes on an Ender-3 (an i3 printer), the first with two instances. */
const CUBES = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";
/** An Ender-3 project with a cube on each of two plates. */
const TWO_PLATES = "tests/parity/custom-gcode-overview/two-plates.3mf";
/** A Bambu Lab X2D with one object on filament 1. */
const X2D = "tests/parity/generated-printer-ids/input.3mf";

const MODEL = "3D/3dmodel.model";
const OBJECTS = "Metadata/model_settings.config";
const PROJECT = "Metadata/project_settings.config";
/** The A3a "several objects" case: named cubes by layer, one turned 30° and one stretched to 30 x 10 mm. */
const SEVERAL: Edit[] = [
  [OBJECTS, "<object id=\"2\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>", "<object id=\"2\">\n    <metadata key=\"name\" value=\"first-cube\"/>"],
  [OBJECTS, "<object id=\"3\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>", "<object id=\"3\">\n    <metadata key=\"name\" value=\"second-cube\"/>"],
  [PROJECT, "\"print_sequence\": \"by object\"", "\"print_sequence\": \"by layer\""],
  [MODEL, "transform=\"1 0 0 0 1 0 0 0 1 50 110 2\"", "transform=\"0.866025404 0.5 0 -0.5 0.866025404 0 0 0 1 50 110 2\""],
  [MODEL, "transform=\"1 0 0 0 1 0 0 0 1 125 110 2\"", "transform=\"3 0 0 0 1 0 0 0 1 125 110 2\""],
];
/** The A3c "several objects" case: plate 2's cube off-centre with two more instances on plate 2. */
const PLATE_2: Edit[] = [
  [OBJECTS, "<object id=\"2\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>", "<object id=\"2\">\n    <metadata key=\"name\" value=\"plate-1-cube\"/>"],
  [OBJECTS, "<object id=\"3\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>", "<object id=\"3\">\n    <metadata key=\"name\" value=\"plate-2-cube\"/>"],
  [MODEL, " 374 110 5\"", " 300 60 5\""],
  [MODEL, "</build>", "<item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 420 160 5\" printable=\"1\"/><item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 330 170 5\" printable=\"1\"/></build>"],
];
/** The settings the CLI flags give: no rotation, no calibration region, no align to Y. */
const FLAGS = ["--allow-rotations=0", "--avoid-extrusion-cali-region=0", "--align-to-y-axis=0"];

const popup = (page: Page) => page.getByRole("dialog", { name: "Arrange settings" });
const option = (page: Page, name: string) => popup(page).getByRole("checkbox", { name });
/** Toggles a popup checkbox; it shows the new state once the engine applied the popup's rules. */
const toggle = (page: Page, name: string) => option(page, name).click();

/** Writes `fixture` with `edits` to the artifacts and opens it in the app. */
async function open(page: Page, name: string, fixture: string, edits: Edit[]) {
  const project = `${artifacts}${name}.3mf`;
  writeFileSync(project, editedProject(fixture, edits));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByRole("button", { name: "Arrange", exact: true })).toBeVisible({ timeout: 60_000 });
  return project;
}

/** Opens the popup with OrcaSlicer's CLI settings for the cases. */
async function cliSettings(page: Page) {
  await page.getByRole("button", { name: "Arrange", exact: true }).click();
  await expect(popup(page)).toBeVisible();
  await popup(page).getByRole("button", { name: "Reset" }).click();
  await toggle(page, "Align to Y axis");
  await expect(option(page, "Align to Y axis")).not.toBeChecked();
}

/** Waits for the arrange to finish without an error. */
async function arranged(page: Page) {
  await expect(page.getByTestId("status")).not.toContainText("Arranging", { timeout: 60_000 });
  await expect(page.getByTestId("status")).not.toContainText("Arrange failed");
}

// Each test writes its own artifacts; a failed test restarts the worker and reruns this.
test.beforeAll(() => mkdirSync(artifacts, { recursive: true }));

test("the arrange popup keeps OrcaSlicer's defaults, rules and stored settings", async ({ page }) => {
  await open(page, "popup", CUBES, SEVERAL);
  await page.getByRole("button", { name: "Arrange", exact: true }).click();
  // The ArrangeSettings defaults with align to Y on the i3 Ender-3 (Plater.cpp:3994-3999).
  await expect(page.getByRole("spinbutton", { name: "Spacing value" })).toHaveValue("0.00");
  await expect(option(page, "Auto rotate for arrangement")).not.toBeChecked();
  await expect(option(page, "Allow multiple materials on same plate")).toBeChecked();
  await expect(option(page, "Align to Y axis")).toBeChecked();
  // Not a Bambu Lab printer scanning the first layer (GLCanvas3D.cpp:6725-6734).
  await expect(option(page, "Avoid extrusion calibration region")).toHaveCount(0);
  // Auto rotate turns align to Y off and disables it (GLCanvas3D.cpp:6737-6745).
  await toggle(page, "Auto rotate for arrangement");
  await expect(option(page, "Auto rotate for arrangement")).toBeChecked();
  await expect(option(page, "Align to Y axis")).toBeDisabled();
  await expect(option(page, "Align to Y axis")).not.toBeChecked();
  await page.getByRole("spinbutton", { name: "Spacing value" }).fill("5");
  await expect(page.getByRole("spinbutton", { name: "Spacing value" })).toHaveValue("5.00");
  await page.screenshot({ path: `${artifacts}popup.png` });

  // Spacing and rotation are stored (appcfg->set), align to Y is not.
  await open(page, "popup", CUBES, SEVERAL);
  await page.getByRole("button", { name: "Arrange", exact: true }).click();
  await expect(page.getByRole("spinbutton", { name: "Spacing value" })).toHaveValue("5.00");
  await expect(option(page, "Auto rotate for arrangement")).toBeChecked();
  await expect(option(page, "Align to Y axis")).not.toBeChecked();
  // "Reset" restores the defaults and erases the stored values (GLCanvas3D.cpp:6757-6772).
  await popup(page).getByRole("button", { name: "Reset" }).click();
  await expect(page.getByRole("spinbutton", { name: "Spacing value" })).toHaveValue("0.00");
  await expect(option(page, "Auto rotate for arrangement")).not.toBeChecked();
  await expect(option(page, "Align to Y axis")).toBeChecked();
  await open(page, "popup", CUBES, SEVERAL);
  await page.getByRole("button", { name: "Arrange", exact: true }).click();
  await expect(page.getByRole("spinbutton", { name: "Spacing value" })).toHaveValue("0.00");
  // Each plate row carries the plate's arrange icon (PartPlate.cpp:2621-2627).
  await expect(page.getByRole("button", { name: "Arrange plate 1" })).toBeVisible();
});

test("Arrange all places the instances like the OrcaSlicer v2.5.0-beta CLI", async ({ page }) => {
  test.setTimeout(300_000);
  const project = await open(page, "arrange-all", CUBES, SEVERAL);
  const native = orcaArrange(project, FLAGS);
  await cliSettings(page);
  await popup(page).getByRole("button", { name: "Arrange", exact: true }).click();
  await arranged(page);
  await page.screenshot({ path: `${artifacts}arrange-all.png` });
  if (native) expect(await appArranged(page)).toEqual(native);
});

test("a plate's arrange icon arranges that plate like the OrcaSlicer v2.5.0-beta CLI --slice", async ({ page }) => {
  test.setTimeout(300_000);
  const project = await open(page, "arrange-plate-2", TWO_PLATES, PLATE_2);
  const native = orcaArrange(project, ["--slice", "2", ...FLAGS]);
  await cliSettings(page);
  await page.getByRole("button", { name: "Arrange plate 2" }).click();
  await arranged(page);
  await expect(page.getByRole("treeitem", { name: /^Plate 2\b/ })).toHaveAttribute("aria-selected", "true");
  await page.screenshot({ path: `${artifacts}arrange-plate-2.png` });
  if (native) expect(await appArranged(page, 1)).toEqual(native);
});

test("Arrange all packs a two-nozzle printer into the nozzles' shared bed like the OrcaSlicer v2.5.0-beta CLI", async ({ page }) => {
  test.setTimeout(300_000);
  // A Bambu Lab X2D (nozzle areas X 0..256 and X 20.5..256), its object moved off the shared bed's centre.
  const project = await open(page, "shared-bed", X2D, [
    [MODEL, "transform=\"1 0 0 0 1 0 0 0 1 138.25 128 5\"", "transform=\"1 0 0 0 1 0 0 0 1 60 60 5\""],
  ]);
  const native = orcaArrange(project, FLAGS);
  await page.getByRole("button", { name: "Arrange", exact: true }).click();
  await popup(page).getByRole("button", { name: "Reset" }).click();
  // The CLI flags' settings; the calibration-region option shows for Bambu Lab printers only.
  for (const name of ["Auto rotate for arrangement", "Align to Y axis", "Avoid extrusion calibration region"]) {
    if ((await option(page, name).count()) === 0) continue;
    if (await option(page, name).isChecked()) await toggle(page, name);
    await expect(option(page, name)).not.toBeChecked();
  }
  await popup(page).getByRole("button", { name: "Arrange", exact: true }).click();
  await arranged(page);
  await page.screenshot({ path: `${artifacts}shared-bed.png` });
  if (native) expect(await appArranged(page)).toEqual(native);
});
