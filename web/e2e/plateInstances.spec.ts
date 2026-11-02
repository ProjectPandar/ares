import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { orcaPlate } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-instances/", import.meta.url));
/** An Ender-3 with a 10 mm cube at (110, 110), identify id 14. */
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";

/**
 * The project the GUI exports after two "Add instance": copies at (121, 121) and (132, 132), moved by
 * 5% of the 220 mm bed each (Plater.cpp:19289-19331), on plate 1 with identify ids 15 and 16 (the
 * session's stand-in for upstream's run-time id, one past the largest).
 */
function twoCopies(): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${FIXTURE}`));
  const model = strFromU8(entries["3D/3dmodel.model"]);
  entries["3D/3dmodel.model"] = strToU8(model.replace("</build>",
    "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 121 121 5\" printable=\"1\"/><item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 132 132 5\" printable=\"1\"/></build>"));
  const settings = strFromU8(entries["Metadata/model_settings.config"]);
  const instance = (id: number, identify: number) =>
    `    <model_instance>\n      <metadata key="object_id" value="2"/>\n      <metadata key="instance_id" value="${id}"/>\n      <metadata key="identify_id" value="${identify}"/>\n    </model_instance>\n`;
  entries["Metadata/model_settings.config"] = strToU8(settings.replace("  </plate>", `${instance(1, 15)}${instance(2, 16)}  </plate>`));
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}

test("the object menu adds and removes instances, and the copies slice like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  const row = page.getByTestId("object-list").getByRole("treeitem").nth(1);
  const menu = page.getByRole("menu", { name: "Object menu" });
  // One instance: "Remove instance" is off (can_decrease_instances).
  await row.click({ button: "right" });
  await expect(menu.getByRole("menuitem", { name: "Remove instance" })).toBeDisabled();
  await menu.getByRole("menuitem", { name: "Add instance" }).click();
  await expect(row).toContainText("×2");
  // The "+" shortcut on the selected object.
  await page.keyboard.press("+");
  await expect(row).toContainText("×3");
  await page.screenshot({ path: `${artifacts}three.png` });
  // "-" removes the last copy, "+" puts it back where it was.
  await page.keyboard.press("-");
  await expect(row).toContainText("×2");
  await page.keyboard.press("+");
  await expect(row).toContainText("×3");

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  const oracle = `${artifacts}two-copies.3mf`;
  writeFileSync(oracle, twoCopies());
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["object menu: Add instance", "+", "-", "+"],
    oracle: "OrcaSlicer --slice 1 of the exported project with the two copies (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
