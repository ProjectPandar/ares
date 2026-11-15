import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plater-toolbar/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates; plate 1 frames its cube at the canvas centre. */
const TWO_PLATES = "tests/parity/custom-gcode-overview/two-plates.3mf";
/** An Ender-3 project with cube10 at the bed centre (110, 110), its centre at z 5. */
const CUBE = "tests/parity/orca_cli_ender3.3mf";

/** A toolbar item as shown: accessible name, tooltip, icon file, enabled and pressed; "|" a separator. */
type Item = string | [name: string, tooltip: string, icon: string, enabled: boolean, pressed?: boolean];

const SELECT = ":\nPlease select at least one object.";
/**
 * GLCanvas3D::_init_main_toolbar and update_shortcut_tooltips (GLCanvas3D.cpp:7491-7664), the
 * separators, GLGizmosManager::init's selectable gizmos for one filament (GLGizmosManager.cpp:243-260)
 * and _init_assemble_view_toolbar (:7730-7778), with nothing selected.
 */
const UNSELECTED: Item[] = [
  ["Add", "Add [Ctrl+I]", "toolbar_open", true],
  ["Add plate", "Add plate", "toolbar_add_plate", true],
  ["Auto orient", "Auto orient all/selected objects [Q]\nAuto orient all objects on current plate [Shift+Q]", "toolbar_orient", false],
  ["Arrange", "Arrange all objects [A]\nArrange objects on selected plates [Shift+A]", "toolbar_arrange", true, false],
  "|",
  ["Add instance", "Add instance [+]", "instance_add", false],
  ["Remove instance", "Remove instance [-]", "instance_remove", false],
  ["Split to objects", "Split to objects", "split_objects", false],
  ["Split to parts", "Split to parts", "split_parts", false],
  ["Variable layer height", "Variable layer height", "toolbar_variable_layer_height", false],
  "|",
  ["Move", `Move${SELECT} [M]`, "toolbar_move", false, false],
  ["Rotate", `Rotate${SELECT} [R]`, "toolbar_rotate", false, false],
  ["Scale", `Scale${SELECT} [S]`, "toolbar_scale", false, false],
  ["Lay on Face", "Lay on Face [F]", "toolbar_flatten", false, false],
  ["Cut", "Cut [C]", "toolbar_cut", false, false],
  ["Mesh Boolean", "Mesh Boolean [B]", "toolbar_meshboolean", false, false],
  ["Support Painting", "Support Painting [L]", "toolbar_support", false, false],
  ["Seam painting", "Seam painting [P]", "toolbar_seam", false, false],
  ["Paint-on fuzzy skin", "Paint-on fuzzy skin [H]", "toolbar_fuzzy_skin_paint", false, false],
  ["Texture displacement", "Texture displacement [D]", "toolbar_texture_displacement", false, false],
  ["Emboss", "Emboss [T]", "toolbar_text", false, false],
  ["Measure", "Measure [U]", "toolbar_measure", false, false],
  ["Assemble", "Assemble [Y]", "toolbar_assembly", false, false],
  ["Brim Ears", "Brim Ears [E]", "toolbar_brimears", false, false],
  "|",
  ["Assembly View", "Assembly View", "toolbar_assemble", false],
];

/** With one cube of one instance selected: the instance and Move/Rotate/Scale items enable. */
const SELECTED = UNSELECTED.map((item): Item => {
  if (typeof item === "string") return item;
  const [name, , icon, , pressed] = item;
  if (name === "Add instance") return [name, "Add instance [+]", icon, true];
  if (["Move", "Rotate", "Scale"].includes(name)) return [name, `${name} [${name[0]}]`, icon, true, pressed];
  return item;
});

const toolbar = (page: Page) => page.getByTestId("plater-toolbar");
const gizmoButton = (page: Page, name: string) => page.getByRole("toolbar", { name: "Gizmos" }).getByRole("button", { name, exact: true });
const mainButton = (page: Page, name: string) => page.getByRole("toolbar", { name: "Main toolbar" }).getByRole("button", { name, exact: true });
const instances = (page: Page) => page.locator(".viewport").getAttribute("data-instances");
const plates = async (page: Page) => JSON.parse((await page.getByTestId("viewport").getAttribute("data-plates"))!) as { origins: number[][]; current: number };
const gizmo = (page: Page) => page.getByTestId("viewport").getAttribute("data-gizmo");

/** The toolbar items in order, icons named without the `_dark` suffix the dark theme shows. */
async function shownItems(page: Page, dark: boolean): Promise<Item[]> {
  const items = await toolbar(page).locator(".toolbar-item, .toolbar-separator").evaluateAll((elements) => elements.map((element) => {
    if (element.classList.contains("toolbar-separator")) return "|";
    const file = element.querySelector("img")!.getAttribute("src")!.split("/").pop()!.match(/^(.*?)(-[\w-]{8})?\.svg$/)![1];
    const pressed = element.getAttribute("aria-pressed");
    const shown = [element.getAttribute("aria-label")!, element.getAttribute("title")!, file, !(element as HTMLButtonElement).disabled];
    return pressed === null ? shown : [...shown, pressed === "true"];
  }));
  return items.map((item) => {
    if (typeof item === "string") return item;
    const [name, tooltip, file, ...state] = item as [string, string, string, boolean, boolean?];
    const icon = file.replace(/_dark$/, "");
    // GLGizmoTextureDisplacement has one icon for both themes.
    expect(file, `${name} icon in the ${dark ? "dark" : "light"} theme`).toBe(dark && icon !== "toolbar_texture_displacement" ? `${icon}_dark` : icon);
    return [name, tooltip, icon, ...state] as Item;
  });
}

async function canvasPng(page: Page) {
  const viewport = page.getByTestId("viewport");
  await expect(viewport).not.toHaveAttribute("data-drawing");
  return viewport.evaluate((container) => container.querySelector("canvas")!.toDataURL("image/png"));
}

/** The canvas pixels on its border that differ from its top-left pixel, the background. */
async function borderMarks(page: Page) {
  const png = await canvasPng(page);
  return page.evaluate(async (url) => {
    const image = new Image();
    image.src = url;
    await image.decode();
    const canvas = new OffscreenCanvas(image.width, image.height);
    const context = canvas.getContext("2d")!;
    context.drawImage(image, 0, 0);
    const { data, width, height } = context.getImageData(0, 0, image.width, image.height);
    const at = (x: number, y: number) => data.slice((y * width + x) * 4, (y * width + x) * 4 + 3).join(",");
    const background = at(0, 0);
    let marks = 0;
    for (let x = 0; x < width; x++) marks += Number(at(x, 0) !== background) + Number(at(x, height - 1) !== background);
    for (let y = 0; y < height; y++) marks += Number(at(0, y) !== background) + Number(at(width - 1, y) !== background);
    return marks;
  }, png);
}

async function load(page: Page, fixture: string) {
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${fixture}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
}

test("the Prepare toolbar follows GLToolbar and GLGizmosManager and runs its items", async ({ page }) => {
  test.setTimeout(180_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await load(page, TWO_PLATES);
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  // [main] | [gizmos] | [assembly view] side by side, centred at the canvas top; the camera view
  // toolbar centred at its bottom.
  const canvas = (await page.getByTestId("viewport").locator("canvas").boundingBox())!;
  const bar = (await toolbar(page).boundingBox())!;
  expect(Math.abs(bar.x + bar.width / 2 - (canvas.x + canvas.width / 2))).toBeLessThanOrEqual(1);
  expect(bar.y - canvas.y).toBeGreaterThanOrEqual(0);
  expect(bar.y - canvas.y).toBeLessThanOrEqual(12);
  const parts = await Promise.all(["Main toolbar", "Gizmos", "Assembly view"].map(async (name) => (await page.getByRole("toolbar", { name }).boundingBox())!));
  expect(parts[0].x + parts[0].width).toBeLessThanOrEqual(parts[1].x);
  expect(parts[1].x + parts[1].width).toBeLessThanOrEqual(parts[2].x);
  const views = (await page.getByTestId("view-toolbar").boundingBox())!;
  expect(Math.abs(views.x + views.width / 2 - (canvas.x + canvas.width / 2))).toBeLessThanOrEqual(1);
  expect(canvas.y + canvas.height - (views.y + views.height)).toBeGreaterThanOrEqual(0);
  expect(canvas.y + canvas.height - (views.y + views.height)).toBeLessThanOrEqual(16);
  await expect(page.locator(".gizmo-toolbar")).toHaveCount(0);

  expect(await shownItems(page, true)).toEqual(UNSELECTED);
  await toolbar(page).screenshot({ path: `${artifacts}toolbar-dark.png` });
  await page.screenshot({ path: `${artifacts}prepare-dark.png` });
  await page.getByRole("button", { name: "Light theme", exact: true }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  expect(await shownItems(page, false)).toEqual(UNSELECTED);
  await toolbar(page).screenshot({ path: `${artifacts}toolbar-light.png` });
  await page.screenshot({ path: `${artifacts}prepare-light.png` });
  await page.getByRole("button", { name: "Dark theme", exact: true }).click();

  // Without a selection the gizmo keys open nothing (GLGizmosManager::open_gizmo_by_shortcut).
  await page.keyboard.press("m");
  await page.waitForTimeout(300);
  expect(await gizmo(page)).toBeNull();

  // A selected cube enables the instance and transform items; none is open at first.
  const tree = page.getByTestId("object-list");
  const cube = tree.getByRole("treeitem").nth(1);
  await cube.click();
  await expect.poll(() => shownItems(page, true)).toEqual(SELECTED);

  // One gizmo at a time: its button or key opens it, the same again closes it (GLGizmosManager::open_gizmo).
  const open = async (mode: string | null) => {
    await expect.poll(async () => (JSON.parse((await gizmo(page)) ?? "null") as { mode: string } | null)?.mode ?? null).toBe(mode);
    for (const name of ["Move", "Rotate", "Scale"]) await expect(gizmoButton(page, name)).toHaveAttribute("aria-pressed", String(name.toLowerCase() === mode));
  };
  await gizmoButton(page, "Move").click();
  await open("move");
  await gizmoButton(page, "Move").click();
  await open(null);
  await page.keyboard.press("r");
  await open("rotate");
  await page.keyboard.press("r");
  await open(null);
  await page.keyboard.press("s");
  await open("scale");
  await toolbar(page).screenshot({ path: `${artifacts}toolbar-scale-open.png` });
  await page.screenshot({ path: `${artifacts}scale-gizmo.png` });
  await gizmoButton(page, "Rotate").click();
  await open("rotate");
  await page.keyboard.press("m");
  await open("move");

  // With a gizmo open a right click on the cube shows no object menu (GLCanvas3D.cpp:5459).
  const [x, y] = [canvas.x + canvas.width / 2, canvas.y + canvas.height / 2];
  await page.mouse.click(x, y, { button: "right" });
  await page.waitForTimeout(500);
  await expect(page.getByRole("menu", { name: "Object menu" })).toHaveCount(0);
  await expect(cube).toHaveAttribute("aria-selected", "true");
  await open("move");
  // Clearing the selection closes it (GLCanvas3D::deselect_all).
  await page.keyboard.press("Escape");
  await open(null);
  await expect(gizmoButton(page, "Move")).toBeDisabled();
  await cube.click();
  await open(null);

  // "Add instance" and "Remove instance" (Plater::increase_instances / decrease_instances), one undo step each.
  await mainButton(page, "Add instance").click();
  await expect(cube).toContainText("×2");
  await expect(mainButton(page, "Remove instance")).toBeEnabled();
  await mainButton(page, "Remove instance").click();
  await expect(cube).not.toContainText("×2");
  await page.keyboard.press("ControlOrMeta+z");
  await expect(cube).toContainText("×2");
  await page.keyboard.press("ControlOrMeta+z");
  await expect(cube).not.toContainText("×2");

  // "Arrange" toggles its settings popup below the toolbar; its "Arrange" moves the objects, one undo step.
  const arranged = await instances(page);
  await mainButton(page, "Arrange").click();
  const popup = page.getByRole("dialog", { name: "Arrange settings" });
  await expect(popup).toBeVisible();
  await expect(mainButton(page, "Arrange")).toHaveAttribute("aria-pressed", "true");
  expect((await popup.boundingBox())!.y).toBeGreaterThanOrEqual(bar.y + bar.height);
  await page.screenshot({ path: `${artifacts}arrange-popup.png` });
  await popup.getByRole("button", { name: "Arrange", exact: true }).click();
  await expect.poll(() => instances(page)).not.toBe(arranged);
  await mainButton(page, "Arrange").click();
  await expect(popup).toHaveCount(0);
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => instances(page)).toBe(arranged);

  // "Add plate" appends a plate, makes it current and frames every plate: none touches the canvas
  // border (Plater::priv::on_action_add_plate, REQUIRES_ZOOM_TO_ALL_PLATE).
  expect((await plates(page)).origins).toHaveLength(2);
  await mainButton(page, "Add plate").click();
  await expect.poll(async () => (await plates(page)).origins.length).toBe(3);
  expect((await plates(page)).current).toBe(2);
  await expect.poll(() => borderMarks(page)).toBe(0);
  writeFileSync(`${artifacts}added-plate.png`, Buffer.from((await canvasPng(page)).split(",")[1], "base64"));
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(async () => (await plates(page)).origins.length).toBe(2);

  // "Add" opens the import file picker: on_action_add runs add_file (Plater.cpp:13559-13566, 19001-19005), whose
  // import_model dialog takes several files (wxFD_MULTIPLE, GUI_App.cpp:5296-5305).
  const [chooser] = await Promise.all([page.waitForEvent("filechooser"), mainButton(page, "Add").click()]);
  expect(chooser.isMultiple()).toBe(true);
  await chooser.setFiles(`${repositoryRoot}${CUBE}`);
  await expect.poll(async () => (await plates(page)).origins, { timeout: 60_000 }).toHaveLength(1);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });

  writeFileSync(`${artifacts}toolbar.json`, `${JSON.stringify({ fixture: TWO_PLATES, unselected: UNSELECTED, selected: SELECTED }, null, 2)}\n`);
});

type Axis = "X" | "Y" | "Z";
const field = (page: Page, group: string, axis: Axis) => page.getByRole("textbox", { name: `${group} ${axis}`, exact: true });
const sizes = async (page: Page) => Promise.all((["X", "Y", "Z"] as const).map(async (axis) => Number(await field(page, "Size", axis).inputValue())));

/** Drags from a grabber of the open scale gizmo away from the gizmo centre on screen by `factor` of their distance. */
async function dragGrabber(page: Page, grabber: "x" | "y" | "z" | "uniform", factor: number) {
  const canvas = (await page.getByTestId("viewport").locator("canvas").boundingBox())!;
  const handles = JSON.parse((await gizmo(page))!) as Record<"center" | typeof grabber, [number, number]>;
  const [gx, gy] = [canvas.x + handles[grabber][0], canvas.y + handles[grabber][1]];
  const [dx, dy] = [handles[grabber][0] - handles.center[0], handles[grabber][1] - handles.center[1]];
  await page.mouse.move(gx, gy);
  await page.mouse.down();
  await page.mouse.move(gx + factor * dx, gy + factor * dy, { steps: 10 });
  await page.mouse.up();
}

test("the scale gizmo's grabbers scale one axis or all, drop the instance onto the bed, one undo step a drag", async ({ page }) => {
  test.setTimeout(120_000);
  const directory = `${artifacts}scale/`;
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await load(page, CUBE);
  await page.getByTestId("object-list").getByRole("treeitem").nth(1).click();
  await expect.poll(() => sizes(page)).toEqual([10, 10, 10]);
  // Framed, the 10 mm cube's 16 px grabbers stand apart.
  await page.getByRole("button", { name: "Fit to objects", exact: true }).click();
  await page.keyboard.press("s");
  await expect.poll(async () => JSON.parse((await gizmo(page)) ?? "{}") as Record<string, unknown>)
    .toMatchObject({ mode: "scale", center: expect.any(Array), x: expect.any(Array), y: expect.any(Array), z: expect.any(Array), uniform: expect.any(Array) });
  await page.screenshot({ path: `${directory}grabbers.png` });
  const results: Record<string, number[]> = {};

  // An X grabber scales X alone about the box centre (GLGizmoScale3D::do_scale_along_axis).
  await dragGrabber(page, "x", 0.5);
  await expect.poll(async () => (await sizes(page))[0]).toBeGreaterThan(10.5);
  results.x = await sizes(page);
  expect(results.x.slice(1)).toEqual([10, 10]);
  await expect(field(page, "Position", "X")).toHaveValue("110.00");
  await expect(field(page, "Position", "Z")).toHaveValue("5.00");
  await page.screenshot({ path: `${directory}x.png` });
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => sizes(page)).toEqual([10, 10, 10]);

  // The Z grabber on the top face scales Z; do_scale drops the cube back onto the bed.
  await dragGrabber(page, "z", 0.5);
  await expect.poll(async () => (await sizes(page))[2]).toBeGreaterThan(10.5);
  results.z = await sizes(page);
  expect(results.z.slice(0, 2)).toEqual([10, 10]);
  await expect(field(page, "Position", "Z")).toHaveValue((results.z[2] / 2).toFixed(2));
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => sizes(page)).toEqual([10, 10, 10]);
  await expect(field(page, "Position", "Z")).toHaveValue("5.00");

  // A corner grabber scales every axis by one ratio, standing on the bed (do_scale_uniform).
  await dragGrabber(page, "uniform", 0.5);
  await expect.poll(async () => (await sizes(page))[0]).toBeGreaterThan(10.5);
  results.uniform = await sizes(page);
  expect(new Set(results.uniform).size).toBe(1);
  await expect(field(page, "Position", "Z")).toHaveValue((results.uniform[2] / 2).toFixed(2));
  await page.screenshot({ path: `${directory}uniform.png` });
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => sizes(page)).toEqual([10, 10, 10]);

  // Shift snaps the ratio to 0.05 steps (calc_ratio's m_snap_step): the 10 mm cube to 0.5 mm.
  await page.keyboard.down("Shift");
  await dragGrabber(page, "uniform", 0.37);
  await page.keyboard.up("Shift");
  await expect.poll(async () => (await sizes(page))[0]).toBeGreaterThan(10.4);
  results.snapped = await sizes(page);
  expect(results.snapped.map((size) => (size * 2) % 1)).toEqual([0, 0, 0]);
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => sizes(page)).toEqual([10, 10, 10]);

  writeFileSync(`${directory}result.json`, `${JSON.stringify({ fixture: CUBE, sizes: results }, null, 2)}\n`);
});
