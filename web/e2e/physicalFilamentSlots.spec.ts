import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { platerExport, withoutCliGroups } from "./platerExport";
import { orcaPlate } from "./platerOracle";
import { nativeSelection, presetSelectProbe, PROJECT_CONFIG } from "./presetSelectOracle";
import { withSelectedPresets } from "./selectedPresetLines";

const fixture = (path: string) => fileURLToPath(new URL(`../../tests/parity/${path}`, import.meta.url));

async function openProject(page: Page, vendor: string, file: string | { name: string; mimeType: string; buffer: Buffer }) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: `All ${vendor} models`, exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(file);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
}

test("adding and merging physical filaments updates Prepare assignments and can be undone", async ({ page }) => {
  await openProject(page, "Bambu Lab", fixture("modifier-feature-filament/control.3mf"));
  const rows = page.getByTestId("object-list");
  const obj = rows.getByRole("treeitem").filter({ hasText: "cube10.stl" }).first();
  await expect(obj.getByRole("img", { name: "Filament 1", exact: true })).toBeVisible();

  const add = page.getByRole("button", { name: "Add one filament", exact: true });
  const removeLast = page.getByRole("button", { name: "Remove last filament", exact: true });
  const third = page.getByTestId("filament-swatch-3");
  await add.click({ timeout: 5_000 });
  await expect(third).toBeVisible();
  await removeLast.click();
  await expect(third).toHaveCount(0);
  await page.keyboard.press("Control+z");
  await expect(third).toBeVisible();

  const actions = page.getByRole("button", { name: "Filament 1 actions", exact: true });
  await actions.click();
  await page.getByRole("menuitem", { name: "Edit", exact: true }).click();
  const settings = page.getByRole("dialog", { name: "Filament settings", exact: true });
  await expect(settings).toBeVisible();
  await settings.getByRole("button", { name: "Close", exact: true }).click();

  await actions.click();
  await page.getByRole("menuitem", { name: "Merge with", exact: true }).click();
  await page.getByRole("menuitem", { name: /^Filament 3:/ }).click();
  await expect(third).toHaveCount(0);
  await expect(obj.getByRole("img", { name: "Filament 2", exact: true })).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(third).toBeVisible();
  await expect(obj.getByRole("img", { name: "Filament 1", exact: true })).toBeVisible();
  await page.keyboard.press("Control+Shift+z");
  await expect(obj.getByRole("img", { name: "Filament 2", exact: true })).toBeVisible();

  await actions.click();
  await page.getByRole("menuitem", { name: "Delete", exact: true }).click();
  await expect(page.getByTestId("filament-swatch-2")).toHaveCount(0);
  await expect(removeLast).toHaveCount(0);
});

test("the filament editor follows the surviving position after deleting a slot", async ({ page }) => {
  const entries = unzipSync(readFileSync(fixture("modifier-feature-filament/control.3mf")));
  const key = "Metadata/project_settings.config";
  const settings = JSON.parse(strFromU8(entries[key]));
  settings.filament_density = ["1.23", "1.45"];
  settings.different_settings_to_system = ["", "filament_density", "filament_density", ""];
  entries[key] = strToU8(JSON.stringify(settings));
  await openProject(page, "Bambu Lab", { name: "two-densities.3mf", mimeType: "application/octet-stream", buffer: Buffer.from(zipSync(entries)) });

  const editor = page.getByRole("dialog", { name: "Filament settings", exact: true });
  const density = editor.getByLabel("filament_density", { exact: true });
  const actions = page.getByRole("button", { name: "Filament 1 actions", exact: true });
  const editFirst = async () => {
    await actions.click();
    await page.getByRole("menuitem", { name: "Edit", exact: true }).click();
    await editor.getByRole("slider", { name: "Settings mode" }).press("End");
  };
  await editFirst();
  await expect(density).toHaveValue("1.23");
  await editor.getByRole("button", { name: "Close", exact: true }).click();

  await actions.click();
  await page.getByRole("menuitem", { name: "Delete", exact: true }).click();
  await page.getByRole("button", { name: "Edit filament 1 settings", exact: true }).click();
  await expect(density).toHaveValue("1.45");
  await editor.getByRole("button", { name: "Close", exact: true }).click();

  await page.keyboard.press("Control+z");
  await editFirst();
  await expect(density).toHaveValue("1.23");
});

async function sliceDownload(page: Page, path: string) {
  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(path);
  return readFileSync(path, "utf8");
}

function modelTools(gcode: string) {
  const tools = new Map<string, Set<number>>();
  let tool = 0;
  let object: string | undefined;
  for (const line of gcode.split("\n")) {
    const command = /^T(\d+)(?:\s|$)/.exec(line);
    // GCodeProcessor.cpp:7128–7134 treats these BBL commands as special, not tool changes.
    if (command && ![255, 1000, 1001, 1100, 65279, 65535].includes(Number(command[1]))) tool = Number(command[1]);
    if (line.startsWith("; printing object ")) object = /^; printing object (colour[01]\.stl) id:/.exec(line)?.[1];
    if (line.startsWith("; stop printing object ")) object = undefined;
    if (object && /^G1\b(?=.* [XY]-?\d)(?=.* E(?:\d|\.\d))/.test(line)) {
      if (!tools.has(object)) tools.set(object, new Set());
      tools.get(object)!.add(tool);
    }
  }
  return Object.fromEntries([...tools].map(([name, indices]) => [name, [...indices]]));
}

test("merging into a physical slot changes object extrusion to the target and matches OrcaSlicer", async ({ page }, info) => {
  test.setTimeout(300_000);
  // The two-cube 3MF already represents the expected assignments after Merge. We
  // never use an Ares export as the input to the independent desktop route.
  const entries = unzipSync(readFileSync(fixture("type1-compacted-exit/x1c-two-slots.3mf")));
  const settings = JSON.parse(strFromU8(entries[PROJECT_CONFIG]));
  // Test object material assignment without unrelated prime-tower arc quantization.
  settings.enable_prime_tower = "0";
  // A single requested size avoids the CLI-only preset refresh's multi-size formatting.
  const thumbnailSize = "48x48/PNG";
  settings.thumbnails = thumbnailSize;
  settings.different_settings_to_system[0] = "enable_prime_tower";
  settings.different_settings_to_system[settings.different_settings_to_system.length - 1] = "thumbnails";
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify(settings));
  const expected = info.outputPath("two-materials.3mf");
  writeFileSync(expected, zipSync(entries));
  await openProject(page, "Bambu Lab", expected);
  const object = page.getByTestId("object-list").getByRole("treeitem").filter({ hasText: "colour0.stl" });
  await expect(object.getByRole("img", { name: "Filament 2", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Add one filament", exact: true }).click();
  await expect(page.getByTestId("filament-swatch-3")).toBeVisible();
  await object.getByRole("button", { name: "Edit filament for colour0.stl", exact: true }).click();
  await page.getByRole("menuitemradio", { name: "Filament 3", exact: true }).click();
  const before = await sliceDownload(page, info.outputPath("before-merge.gcode"));
  expect(modelTools(before)).toEqual({ "colour1.stl": [0], "colour0.stl": [2] });

  await page.getByRole("button", { name: "Filament 3 actions", exact: true }).click();
  await page.getByRole("menuitem", { name: "Merge with", exact: true }).click();
  await page.getByRole("menuitem", { name: /^Filament 2:/ }).click();
  await expect(page.getByTestId("filament-swatch-3")).toHaveCount(0);
  await expect(object.getByRole("img", { name: "Filament 2", exact: true })).toBeVisible();
  const after = await sliceDownload(page, info.outputPath("after-merge.gcode"));
  const expectedTools = { "colour1.stl": [0], "colour0.stl": [1] };
  expect(modelTools(after)).toEqual(expectedTools);

  const probe = presetSelectProbe();
  test.skip(probe === null, "OrcaSlicer preset selection probe unavailable");
  const selection = nativeSelection(probe!, `${info.outputPath("preset-selection")}/`, expected, ["BBL"], {
    filament_slot_edits: [{ append: "#008B00" }, { remove_last: true }],
  });
  const config = JSON.parse(selection.full_fff_config);
  // PresetBundle.cpp:4375–4465, 8598–8671: removing the appended position
  // leaves both preset positions, colours and the original directed flush pair.
  expect(selection.filament_presets).toEqual(["Bambu PLA Basic @BBL X1C", "Bambu PLA Basic @BBL X1C"]);
  expect(config.filament_colour).toEqual(["#CF3020", "#307CC0"]);
  expect(config.flush_volumes_matrix.map(Number)).toEqual([0, 205, 229, 0]);
  const reference = platerExport(selection, expected, info.outputPath("merged-reference.3mf"));
  const leg = process.env.ARES_PARITY_SEEDED_REFERENCE === "1" ? "seeded" : "official";
  const native = orcaPlate(reference, 1, leg);
  test.skip(native === null, `${leg} OrcaSlicer unavailable`);
  expect(modelTools(native!)).toEqual(expectedTools);
  const composed = withoutCliGroups(withSelectedPresets(native!, selection), selection);
  const actual = normalize(after, "Ares");
  writeFileSync(info.outputPath("native.normalized.gcode"), composed);
  writeFileSync(info.outputPath("browser.normalized.gcode"), actual);
  expect(composed.split("\n")).toContain(`; thumbnails = ${thumbnailSize}`);
  expect(actual.split("\n")).toContain(`; thumbnails = ${thumbnailSize}`);
  expect(actual === composed, `download equals ${leg} OrcaSlicer`).toBe(true);
});

test("fixed-head slots can open the editor without showing count controls", async ({ page }) => {
  await openProject(page, "Snapmaker", fixture("type2-tower-entry-z/j1-two-tools.3mf"));
  await expect(page.getByRole("button", { name: "Add one filament", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Remove last filament", exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "Edit filament 2 settings", exact: true }).click();
  await expect(page.getByRole("dialog", { name: "Filament settings", exact: true })).toBeVisible();
});
