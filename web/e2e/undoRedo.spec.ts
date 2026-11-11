import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/undo-redo/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates. */
const FIXTURE = "tests/parity/custom-gcode-overview/two-plates.3mf";

/** What the Prepare tab shows of the plater: drawn instances and plates, the object list, the canvas. */
interface Shown {
  instances: string | null;
  plates: string | null;
  tree: string[];
  /** SHA-256 of the canvas pixels. */
  canvas: string;
}

async function shown(page: Page, png?: string): Promise<Shown> {
  const viewport = page.getByTestId("viewport");
  await expect(viewport).not.toHaveAttribute("data-drawing");
  const pixels = await viewport.evaluate((container) => container.querySelector("canvas")!.toDataURL("image/png"));
  if (png) writeFileSync(`${artifacts}${png}.png`, Buffer.from(pixels.split(",")[1], "base64"));
  return {
    instances: await page.locator(".viewport").getAttribute("data-instances"),
    plates: await viewport.getAttribute("data-plates"),
    tree: await page.getByTestId("object-list").getByRole("treeitem")
      .evaluateAll((items) => items.map((item) => `${item.getAttribute("aria-selected")} ${item.textContent}`)),
    canvas: await page.evaluate(async (data) => {
      const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(data));
      return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    }, pixels),
  };
}

/**
 * The state without tower positions: a loaded scene has none until an edit returns them
 * (`Scene.wipeTower`), so the loaded state is compared without them.
 */
function withoutTower(state: Shown) {
  const { wipeTower, ...instances } = JSON.parse(state.instances!) as { wipeTower?: unknown };
  return { state: { ...state, instances: JSON.stringify(instances) }, wipeTower };
}

/** Expects the shown state to be `expected`, the loaded state 0 with the tower positions edits report. */
async function expectShown(page: Page, expected: Shown, states: Shown[], message: string, view = (state: Shown) => state) {
  if (expected !== states[0]) {
    await expect.poll(async () => view(await shown(page)), message).toEqual(view(expected));
    return;
  }
  await expect.poll(async () => withoutTower(view(await shown(page))), message)
    .toEqual({ state: withoutTower(view(expected)).state, wipeTower: withoutTower(states[1]).wipeTower });
}

/**
 * The state without the canvas pixels: the viewport can draw an edge pixel of the same scene
 * differently when its meshes were rebuilt rather than moved into place.
 */
function withoutCanvas(state: Shown): Shown {
  return { ...state, canvas: "" };
}

/** Waits for an edit to show, then records the state as the next edit's undo snapshot sees it. */
async function edited(page: Page, before: Shown, png: string) {
  await expect.poll(async () => (await page.locator(".viewport").getAttribute("data-instances")) !== before.instances).toBe(true);
  return shown(page, png);
}

async function slicedGcode(page: Page, name: string) {
  await page.getByRole("navigation").getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("navigation").getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  await download.saveAs(`${artifacts}${name}`);
  await page.getByRole("link", { name: "Prepare" }).click();
  return readFileSync(`${artifacts}${name}`);
}

test("Ctrl+Z and Ctrl+Y step the plater back and forward through its edits", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  const loaded = await slicedGcode(page, "loaded.gcode");

  const tree = page.getByTestId("object-list");
  await tree.getByRole("treeitem").nth(1).click();
  const states = [await shown(page, "state-0")];
  const last = () => states.at(-1)!;

  // A canvas drag of the selected cube ("Move Object").
  const canvas = page.getByTestId("viewport").locator("canvas");
  const box = (await canvas.boundingBox())!;
  const [x, y] = [box.x + box.width / 2, box.y + box.height / 2];
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + 120, y, { steps: 10 });
  await page.mouse.up();
  states.push(await edited(page, last(), "state-1"));
  // "Add instance".
  await page.keyboard.press("+");
  states.push(await edited(page, last(), "state-2"));
  // A Position field.
  const positionX = page.getByRole("textbox", { name: "Position X", exact: true });
  await positionX.fill("60");
  await positionX.press("Enter");
  await positionX.blur();
  states.push(await edited(page, last(), "state-3"));
  // "Arrange", then "Delete" of the selected object.
  await page.keyboard.press("a");
  states.push(await edited(page, last(), "state-4"));
  await page.keyboard.press("Delete");
  states.push(await edited(page, last(), "state-5"));
  expect(new Set(states.map(({ instances }) => instances)).size).toBe(states.length);

  for (let index = states.length - 2; index >= 0; index--) {
    await page.keyboard.press("ControlOrMeta+z");
    await expectShown(page, states[index], states, `undo to state ${index}`);
    await shown(page, `undo-${index}`);
  }
  // Nothing before the first edit.
  await page.keyboard.press("ControlOrMeta+z");
  await page.waitForTimeout(500);
  await expectShown(page, states[0], states, "no undo before the first edit");

  for (let index = 1; index < states.length; index++) {
    await page.keyboard.press(index % 2 ? "ControlOrMeta+y" : "ControlOrMeta+Shift+z");
    await expect.poll(() => shown(page), `redo to state ${index}`).toEqual(states[index]);
    await shown(page, `redo-${index}`);
  }
  await page.keyboard.press("ControlOrMeta+y");
  await page.waitForTimeout(500);
  expect(await shown(page)).toEqual(states.at(-1));

  // An undo invalidates the slice result.
  await slicedGcode(page, "edited.gcode");
  await expect(page.getByTestId("status")).toContainText("Slice complete");
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => shown(page)).toEqual(states[4]);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  // A new edit after undoing drops the states ahead.
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(() => shown(page)).toEqual(states[3]);
  await page.keyboard.press("+");
  const branched = await edited(page, states[3], "branched");
  await page.keyboard.press("ControlOrMeta+y");
  await page.waitForTimeout(500);
  expect(await shown(page)).toEqual(branched);

  // A focused field keeps the browser's own undo.
  await positionX.focus();
  await page.keyboard.press("ControlOrMeta+z");
  await page.waitForTimeout(500);
  await positionX.blur();
  expect((await shown(page)).instances).toBe(branched.instances);

  // Back to the loaded project: plate 1 slices to the same G-code as before any edit.
  await page.keyboard.press("ControlOrMeta+z");
  await expect.poll(async () => withoutCanvas(await shown(page))).toEqual(withoutCanvas(states[3]));
  for (let index = 2; index >= 0; index--) {
    await page.keyboard.press("ControlOrMeta+z");
    await expectShown(page, states[index], states, `undo to state ${index}`, withoutCanvas);
  }
  const undone = await slicedGcode(page, "undone.gcode");
  expect(undone.equals(loaded), "plate 1 G-code after undoing every edit").toBe(true);

  // The Preview tab has no plater undo or redo.
  const settled = (await shown(page)).instances;
  await page.getByRole("link", { name: "Preview" }).click();
  await page.keyboard.press("ControlOrMeta+y");
  await page.keyboard.press("ControlOrMeta+Shift+z");
  await page.waitForTimeout(500);
  await page.getByRole("link", { name: "Prepare" }).click();
  expect((await shown(page)).instances).toBe(settled);

  // Loading a project starts an empty history.
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  const reloaded = await shown(page, "reloaded");
  await page.keyboard.press("ControlOrMeta+z");
  await page.waitForTimeout(500);
  expect((await shown(page)).instances).toBe(reloaded.instances);

  writeFileSync(`${artifacts}states.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["canvas drag", "+ (Add instance)", "Position X = 60", "A (Arrange)", "Delete"],
    states,
    branched,
    gcode: { loaded: loaded.length, undone: undone.length, identical: undone.equals(loaded) },
  }, null, 2)}\n`);
});
