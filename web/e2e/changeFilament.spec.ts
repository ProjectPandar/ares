import { readFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";

// The reference project assigns coral.stl to T0 and azure.stl to T1.
const assigned = readFileSync(new URL("../../tests/parity/type2-tower-entry-z/j1-two-tools.3mf", import.meta.url));

function initiallySameFilament() {
  const files = unzipSync(assigned);
  const path = "Metadata/model_settings.config";
  const model = strFromU8(files[path]);
  expect(model.match(/<metadata key="extruder" value="2"\s*\/>/g)).toHaveLength(1);
  files[path] = strToU8(model.replace(/(<metadata key="extruder" value=")2("\s*\/>)/, (_, before, after) => `${before}1${after}`));
  return Buffer.from(zipSync(files));
}

async function setup(page: Page, vendor: string) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: `All ${vendor} models`, exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
}

async function openProject(page: Page, buffer: Buffer) {
  await page.getByTestId("import-input").setInputFiles({ name: "two-cubes.3mf", mimeType: "application/octet-stream", buffer });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
}

async function sliceDownload(page: Page, path: string) {
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code" }).click()]);
  await download.saveAs(path);
  return readFileSync(path);
}

function objectTools(gcode: string) {
  const tools: Record<string, Set<number>> = {};
  let tool = 0;
  let object: string | undefined;
  for (const line of gcode.split("\n")) {
    const command = line.match(/^T([01])(?:\s|$)/);
    if (command) tool = Number(command[1]);
    if (line.startsWith("; printing object ")) object = line.match(/^; printing object ((?:coral|azure)\.stl) id:/)?.[1];
    if (line.startsWith("; stop printing object ")) object = undefined;
    if (object && /^G1\b(?=.* [XY]-?\d)(?=.* E(?:\d|\.\d))/.test(line)) (tools[object] ??= new Set()).add(tool);
  }
  return Object.fromEntries(Object.entries(tools).map(([name, used]) => [name, [...used]]));
}

test("changing one object's filament changes its extrusion tool and the preview legend", async ({ page }, info) => {
  await setup(page, "Snapmaker");
  await openProject(page, initiallySameFilament());
  const object = page.getByTestId("object-list").getByRole("treeitem").filter({ hasText: "azure.stl" });
  await expect(object.getByRole("img", { name: "Filament 1" })).toBeVisible();
  await object.getByText("azure.stl", { exact: true }).click({ button: "right" });
  await page.getByRole("menuitem", { name: "Change Filament", exact: true }).click();
  await page.getByRole("menuitem", { name: "Filament 2: Snapmaker J1 PLA", exact: true }).click();
  await expect(object.getByRole("img", { name: "Filament 2" })).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(object.getByRole("img", { name: "Filament 1" })).toBeVisible();
  await page.keyboard.press("Control+Shift+z");
  await expect(object.getByRole("img", { name: "Filament 2" })).toBeVisible();
  const edited = await sliceDownload(page, info.outputPath("assigned-in-prepare.gcode"));
  expect(objectTools(edited.toString("utf8"))).toEqual({ "coral.stl": [0], "azure.stl": [1] });
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  await page.getByRole("combobox", { name: "View type" }).selectOption("ColorPrint");
  const legend = page.getByRole("table", { name: "Filament usage" });
  await expect(legend.getByRole("cell", { name: "1", exact: true }).first()).toBeVisible();
  await expect(legend.getByRole("cell", { name: "2", exact: true }).first()).toBeVisible();

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await openProject(page, assigned);
  const direct = await sliceDownload(page, info.outputPath("assigned-in-project.gcode"));
  expect(edited.equals(direct)).toBe(true);
});

test("Prepare offers Default for a parameter modifier but not for a normal part", async ({ page }) => {
  const project = readFileSync(new URL("../../tests/parity/modifier-feature-filament/control.3mf", import.meta.url));
  await setup(page, "Bambu Lab");
  await openProject(page, project);
  const tree = page.getByTestId("object-list");
  await tree.getByRole("button", { name: "Toggle parts" }).click();
  await tree.getByText("cube10.stl", { exact: true }).last().click({ button: "right" });
  await page.getByRole("menuitem", { name: "Change Filament", exact: true }).click();
  await expect(page.getByRole("menuitem", { name: "Default", exact: true })).toHaveCount(0);
  await page.keyboard.press("Escape");
  await tree.getByText("epsilon-modifier", { exact: true }).click({ button: "right" });
  await page.getByRole("menuitem", { name: "Change Filament", exact: true }).click();
  await page.getByRole("menuitem", { name: "Default", exact: true }).click();
  await expect(page.getByRole("menu", { name: "Object menu" })).toHaveCount(0);
});

test("the filament cell edits only its row and slices like the assigned project", async ({ page }, info) => {
  await setup(page, "Snapmaker");
  await openProject(page, initiallySameFilament());
  const tree = page.getByTestId("object-list");
  const coral = tree.getByRole("treeitem").filter({ hasText: "coral.stl" });
  const azure = tree.getByRole("treeitem").filter({ hasText: "azure.stl" });
  await coral.getByText("coral.stl", { exact: true }).click();
  await azure.getByText("azure.stl", { exact: true }).click({ modifiers: ["Control"] });
  await expect(coral).toHaveAttribute("aria-selected", "true");
  await expect(azure).toHaveAttribute("aria-selected", "true");
  await azure.getByRole("button", { name: "Edit filament for azure.stl" }).click({ timeout: 5_000 });
  const menu = page.getByRole("menu", { name: "Filament for azure.stl" });
  const one = menu.getByRole("menuitemradio", { name: "Filament 1" });
  const two = menu.getByRole("menuitemradio", { name: "Filament 2" });
  await expect(menu.getByRole("menuitemradio")).toHaveCount(2);
  await expect(one).toHaveAttribute("aria-checked", "true");
  await expect(two).toHaveAttribute("aria-checked", "false");
  const files = unzipSync(assigned);
  const { filament_colour: colours } = JSON.parse(strFromU8(files["Metadata/project_settings.config"])) as { filament_colour: string[] };
  const rgb = (hex: string) => `rgb(${[1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16)).join(", ")})`;
  await expect(one.getByText("1", { exact: true })).toHaveCSS("background-color", rgb(colours[0]));
  await expect(two.getByText("2", { exact: true })).toHaveCSS("background-color", rgb(colours[1]));
  await two.click();
  await expect(menu).toHaveCount(0);
  await expect(coral.getByRole("img", { name: "Filament 1" })).toBeVisible();
  await expect(azure.getByRole("img", { name: "Filament 2" })).toBeVisible();

  const edited = await sliceDownload(page, info.outputPath("inline-assigned.gcode"));
  expect(objectTools(edited.toString("utf8"))).toEqual({ "coral.stl": [0], "azure.stl": [1] });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await openProject(page, assigned);
  const baked = await sliceDownload(page, info.outputPath("baked-assignment.gcode"));
  expect(edited.equals(baked)).toBe(true);
});

test("part and modifier filament cells offer row-specific choices and object edits undo together", async ({ page }) => {
  const fixture = readFileSync(new URL("../../tests/parity/modifier-feature-filament/control.3mf", import.meta.url));
  const files = unzipSync(fixture);
  const path = "Metadata/model_settings.config";
  const xml = strFromU8(files[path]);
  files[path] = strToU8(xml.replace(/<part id="(?:1|3)"[^>]*>/g, (part) =>
    `${part}<metadata key="extruder" value="${part.includes('id="1"') ? 1 : 2}"/>`));
  await setup(page, "Bambu Lab");
  await openProject(page, Buffer.from(zipSync(files)));
  const tree = page.getByTestId("object-list");
  await tree.getByRole("button", { name: "Toggle parts" }).click();
  const [object, part] = await tree.getByRole("treeitem").filter({ hasText: "cube10.stl" }).all();
  const modifier = tree.getByRole("treeitem").filter({ hasText: "epsilon-modifier" });
  const slot = (row: typeof part, name: string) => row.getByRole("img", { name, exact: true });
  await expect(slot(part, "Filament 1")).toBeVisible();
  await expect(slot(modifier, "Filament 2")).toBeVisible();

  const partCell = part.getByRole("button", { name: "Edit filament for cube10.stl" });
  const partMenu = page.getByRole("menu", { name: "Filament for cube10.stl" });
  const viewport = page.getByTestId("viewport");
  await expect(viewport.locator("canvas")).toBeVisible();
  await expect(viewport).not.toHaveAttribute("data-drawing", "");
  const beforeCancel = await viewport.screenshot();
  await partCell.press("Enter", { timeout: 5_000 });
  await expect(partMenu.getByRole("menuitemradio", { name: "Default" })).toHaveCount(0);
  await expect(partMenu.getByRole("menuitemradio", { name: "Filament 1" })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(partMenu).toHaveCount(0);
  await expect(partCell).toBeFocused();
  await expect(slot(part, "Filament 1")).toBeVisible();
  expect((await viewport.screenshot()).equals(beforeCancel)).toBe(true);
  await partCell.click();
  await partMenu.getByRole("menuitemradio", { name: "Filament 2" }).click();
  await expect(slot(part, "Filament 2")).toBeVisible();
  await expect(slot(object, "Filament 1")).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(slot(part, "Filament 1")).toBeVisible();

  await modifier.getByRole("button", { name: "Edit filament for epsilon-modifier" }).click();
  await page.getByRole("menu", { name: "Filament for epsilon-modifier" }).getByRole("menuitemradio", { name: "Default" }).click();
  await expect(slot(modifier, "Default")).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(slot(modifier, "Filament 2")).toBeVisible();

  await object.getByText("cube10.stl", { exact: true }).click({ button: "right" });
  await page.getByRole("menuitem", { name: "Change Filament", exact: true }).click();
  await page.getByRole("menuitem", { name: /^Filament 2:/ }).click();
  await expect(slot(part, "Filament 2")).toBeVisible();
  await expect(slot(modifier, "Filament 2")).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(slot(part, "Filament 1")).toBeVisible();

  await object.getByRole("button", { name: "Edit filament for cube10.stl" }).press("Space");
  await partMenu.getByRole("menuitemradio", { name: "Filament 2" }).click();
  await expect(slot(part, "Filament 2")).toBeVisible();
  await expect(slot(modifier, "Default")).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(slot(part, "Filament 1")).toBeVisible();
  await expect(slot(modifier, "Filament 2")).toBeVisible();
  await page.keyboard.press("Control+Shift+z");
  await expect(slot(part, "Filament 2")).toBeVisible();
  await expect(slot(modifier, "Default")).toBeVisible();
});
