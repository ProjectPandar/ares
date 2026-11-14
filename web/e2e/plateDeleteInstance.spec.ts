import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { orcaPlate } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-delete-instance/", import.meta.url));
/** Two 10 mm cubes on an Ender-3: object 2 with instances at x = 50 and 125, object 3 at x = 200. */
const FIXTURE = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";

/**
 * The project the GUI exports after deleting object 2's first instance: one build item for its
 * remaining instance, and the plate map left as it was (ModelObject::delete_instance tells no
 * plate), the pair (2, 1) now written with identify_id 0 (bbs_3mf.cpp:8580-8600).
 */
function withoutFirstInstance(): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${FIXTURE}`));
  const model = strFromU8(entries["3D/3dmodel.model"]);
  const first = model.indexOf("<item objectid=\"2\"");
  expect(first, "object 2's first build item").toBeGreaterThanOrEqual(0);
  entries["3D/3dmodel.model"] = strToU8(model.slice(0, first) + model.slice(model.indexOf("/>", first) + 2));
  const settings = strFromU8(entries["Metadata/model_settings.config"]);
  const pairs = "<model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"0\"/><metadata key=\"identify_id\" value=\"14\"/></model_instance><model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"1\"/><metadata key=\"identify_id\" value=\"15\"/></model_instance>";
  expect(settings, "object 2's plate pairs").toContain(pairs);
  entries["Metadata/model_settings.config"] = strToU8(settings.replace(pairs,
    "<model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"0\"/><metadata key=\"identify_id\" value=\"15\"/></model_instance><model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"1\"/><metadata key=\"identify_id\" value=\"0\"/></model_instance>"));
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}

test("Delete on an instance item removes that instance, and the plate slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  const tree = page.getByTestId("object-list");
  const object = tree.getByRole("treeitem").filter({ hasText: "×2" });
  await object.getByRole("button", { name: "Toggle parts" }).click();
  await expect(tree.getByRole("treeitem", { name: "Instances", exact: true })).toBeVisible();
  await tree.getByRole("treeitem", { name: "Instance 1", exact: true }).click();
  await page.screenshot({ path: `${artifacts}selected.png` });
  await page.keyboard.press("Delete");
  await expect(tree.getByRole("treeitem").filter({ hasText: "×2" })).toHaveCount(0);
  await expect(tree.getByRole("treeitem", { name: "Instances", exact: true })).toHaveCount(0);
  await page.screenshot({ path: `${artifacts}deleted.png` });

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}without-first-instance.3mf`;
  writeFileSync(oracle, withoutFirstInstance());
  const legs = { official: orcaPlate(oracle, 1, "official", `${repositoryRoot}${FIXTURE}`), seeded: orcaPlate(oracle, 1, "seeded", `${repositoryRoot}${FIXTURE}`) };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["object list: Instance 1 of object 2, Delete"],
    oracle: "OrcaSlicer --slice 1 of the exported project without the instance (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
