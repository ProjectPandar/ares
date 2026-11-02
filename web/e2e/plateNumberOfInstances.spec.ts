import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { CUBE_FIXTURE, cubeWithTwoCopies } from "./instanceOracle";
import { normalize } from "./orcaGcode";
import { orcaPlate } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-number-of-instances/", import.meta.url));

test("\"Set number of instances\" reaches the count, and the copies slice like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${CUBE_FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  const row = page.getByTestId("object-list").getByRole("treeitem").nth(1);
  await row.click({ button: "right" });
  await page.getByRole("menu", { name: "Object menu" }).getByRole("menuitem", { name: "Set number of instances..." }).click();
  // GetNumberFromUser, preset to the current count (Plater.cpp:19394-19395).
  const dialog = page.getByRole("dialog", { name: "Copies of the selected object" });
  const field = dialog.getByRole("spinbutton", { name: "Number of copies:" });
  await expect(field).toHaveValue("1");
  await field.fill("3");
  await page.screenshot({ path: `${artifacts}dialog.png` });
  await dialog.getByRole("button", { name: "OK" }).click();
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
  writeFileSync(oracle, cubeWithTwoCopies());
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: CUBE_FIXTURE,
    edits: ["object menu: Set number of instances = 3"],
    oracle: "OrcaSlicer --slice 1 of the exported project with the two copies (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
