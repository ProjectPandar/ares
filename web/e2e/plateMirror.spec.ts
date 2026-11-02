import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { orcaPlate } from "./platerOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-mirror/", import.meta.url));
const FIXTURE = "tests/parity/orca_cli_ender3.3mf";
/** The asymmetric prism of `plater_rotation_pivot.rs`: the triangle (0, 0), (20, 0), (10, 16), 10 high. */
const PRISM = `<mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="20" y="0" z="0"/><vertex x="10" y="16" z="0"/><vertex x="0" y="0" z="10"/><vertex x="20" y="0" z="10"/><vertex x="10" y="16" z="10"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="3" v2="4" v3="5"/><triangle v1="0" v2="1" v3="4"/><triangle v1="0" v2="4" v3="3"/><triangle v1="2" v2="0" v3="3"/><triangle v1="2" v2="3" v3="5"/><triangle v1="1" v2="2" v3="5"/><triangle v1="1" v2="5" v3="4"/></triangles></mesh>`;

/** The Ender-3 project with its cube replaced by the prism, its build item at `item`. */
function prismProject(item: string): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${FIXTURE}`));
  const mesh = strFromU8(entries["3D/Objects/cube10.stl_1.model"]);
  entries["3D/Objects/cube10.stl_1.model"] = strToU8(mesh.slice(0, mesh.indexOf("<mesh>")) + PRISM + mesh.slice(mesh.indexOf("</mesh>") + "</mesh>".length));
  entries["3D/3dmodel.model"] = strToU8(strFromU8(entries["3D/3dmodel.model"]).replace("transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"", `transform="${item}"`));
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}

const field = (page: Page, group: string, axis: "X" | "Y" | "Z") => page.getByRole("textbox", { name: `${group} ${axis}`, exact: true });

test("the object menu mirrors an instance about its centre and slices like OrcaSlicer", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  // The prism's (0, 0, 0) corner at (100, 100, 0); it loads with its box centre (110, 108, 5) as offset.
  const project = `${artifacts}prism.3mf`;
  writeFileSync(project, prismProject("1 0 0 0 1 0 0 0 1 100 100 0"));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  // ObjectList's right click selects the object and shows its menu (GUI_Factories.cpp:1353-1363).
  await page.getByTestId("object-list").getByRole("treeitem").nth(1).click({ button: "right" });
  const menu = page.getByRole("menu", { name: "Object menu" });
  await expect(menu.getByRole("menuitem")).toHaveText(["Delete", "Add instance", "Remove instance", "Along X Axis", "Along Y Axis", "Along Z Axis"]);
  await page.screenshot({ path: `${artifacts}menu.png` });
  await menu.getByRole("menuitem", { name: "Along X Axis" }).click();
  await expect(menu).toHaveCount(0);
  // Mirrored about its box centre, the offset stays.
  for (const [axis, value] of [["X", "110.00"], ["Y", "108.00"], ["Z", "5.00"]] as const) await expect(field(page, "Position", axis)).toHaveValue(value);
  await page.screenshot({ path: `${artifacts}mirrored.png` });

  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}browser.gcode`);
  const gcode = readFileSync(`${artifacts}browser.gcode`, "utf8");

  // Native slices the prism's own mesh frame: (110, 108, 5) - diag(-1, 1, 1) (10, 8, 5) = (120, 100, 0).
  const oracle = `${artifacts}mirrored.3mf`;
  writeFileSync(oracle, prismProject("-1 0 0 0 1 0 0 0 1 120 100 0"));
  const legs = { official: orcaPlate(oracle, 1, "official"), seeded: orcaPlate(oracle, 1, "seeded") };
  const results = Object.fromEntries(Object.entries(legs).map(([leg, native]) =>
    [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"]));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: `${FIXTURE} with the asymmetric prism`,
    edits: ["object menu: Mirror Along X Axis"],
    oracle: "OrcaSlicer --slice 1 of the mirrored project (official and seed-replay legs)",
    evidence: "ordered_bytes_generator_only",
    results,
  }, null, 2)}\n`);
  for (const [leg, outcome] of Object.entries(results)) expect(outcome, `browser plate 1 vs ${leg} OrcaSlicer`).not.toBe("FAIL");
});
