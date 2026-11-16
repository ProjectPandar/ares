import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { orcaPlate } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-delete-object/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates: object 2 on plate 1, object 3 on plate 2. */
const FIXTURE = "tests/parity/custom-gcode-overview/two-plates.3mf";

/** `text` without the element opening at `open` (through `close`). */
function without(text: string, open: string, close: string): string {
  const start = text.indexOf(open);
  expect(start, open).toBeGreaterThanOrEqual(0);
  const end = text.indexOf(close, start) + close.length;
  return text.slice(0, start) + text.slice(end);
}

/** The project the GUI exports after deleting object 2: its resource, build item and settings gone. */
function withoutPlateOneCube(): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${FIXTURE}`));
  let model = strFromU8(entries["3D/3dmodel.model"]);
  model = without(model, "<object id=\"2\"", "</object>");
  model = without(model, "<item objectid=\"2\"", "/>");
  entries["3D/3dmodel.model"] = strToU8(model);
  let settings = strFromU8(entries["Metadata/model_settings.config"]);
  settings = without(settings, "<object id=\"2\"", "</object>");
  settings = without(settings, "<model_instance>\n      <metadata key=\"object_id\" value=\"2\"/>", "</model_instance>");
  entries["Metadata/model_settings.config"] = strToU8(settings);
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}

test("the object menu deletes an object, and the other plate slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  const tree = page.getByTestId("object-list");
  // Plate 1, its cube, plate 2, its cube.
  await expect(tree.getByRole("treeitem")).toHaveCount(4);
  await tree.getByRole("treeitem").nth(1).click({ button: "right" });
  const menu = page.getByRole("menu", { name: "Object menu" });
  await menu.getByRole("menuitem", { name: "Delete" }).click();
  await expect(tree.getByRole("treeitem")).toHaveCount(3);
  await page.screenshot({ path: `${artifacts}deleted.png` });

  await tree.getByRole("treeitem", { name: /^Plate 2\b/ }).click();
  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}without-object-2.3mf`;
  writeFileSync(oracle, withoutPlateOneCube());
  const legs = { official: orcaPlate(oracle, 2, "official", `${repositoryRoot}${FIXTURE}`), seeded: orcaPlate(oracle, 2, "seeded", `${repositoryRoot}${FIXTURE}`) };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["object menu: Delete plate 1's cube"],
    oracle: "OrcaSlicer --slice 2 of the exported project without the object (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 2 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
