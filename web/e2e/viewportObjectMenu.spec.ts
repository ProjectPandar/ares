import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/viewport-object-menu/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates; plate 1 frames its cube at the canvas centre. */
const FIXTURE = "tests/parity/custom-gcode-overview/two-plates.3mf";

const objectMenu = (page: Page) => page.getByRole("menu", { name: "Object menu" });
const instances = (page: Page) => page.locator(".viewport").getAttribute("data-instances");

async function canvasPixels(page: Page) {
  const viewport = page.getByTestId("viewport");
  await expect(viewport).not.toHaveAttribute("data-drawing");
  return viewport.evaluate((container) => container.querySelector("canvas")!.toDataURL("image/png"));
}

test("a right click on an object in the canvas shows the object menu, a right drag pans", async ({ page }) => {
  test.setTimeout(120_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  // The browser's own menu stays closed: every contextmenu event on the canvas is prevented.
  await page.evaluate(() => {
    const prevented: boolean[] = [];
    (window as unknown as { prevented: boolean[] }).prevented = prevented;
    window.addEventListener("contextmenu", (event) => prevented.push(event.defaultPrevented));
  });
  const prevented = () => page.evaluate(() => (window as unknown as { prevented: boolean[] }).prevented);

  const tree = page.getByTestId("object-list");
  const cube = tree.getByRole("treeitem").nth(1);
  await expect(cube).toHaveAttribute("aria-selected", "false");
  const box = (await page.getByTestId("viewport").locator("canvas").boundingBox())!;
  const [x, y] = [box.x + box.width / 2, box.y + box.height / 2];

  // Right click on the cube: selected, with the menu at the cursor.
  await page.mouse.click(x, y, { button: "right" });
  await expect(objectMenu(page)).toBeVisible();
  await expect(cube).toHaveAttribute("aria-selected", "true");
  const menuBox = (await objectMenu(page).boundingBox())!;
  expect([menuBox.x, menuBox.y]).toEqual([x, y]);
  await page.screenshot({ path: `${artifacts}menu.png` });

  // Its items act on the clicked object.
  const before = await instances(page);
  await objectMenu(page).getByRole("menuitem", { name: "Add instance" }).click();
  await expect(objectMenu(page)).toHaveCount(0);
  await expect(cube).toContainText("×2");
  const added = await instances(page);
  expect(added).not.toBe(before);
  await page.mouse.click(x, y, { button: "right" });
  await objectMenu(page).getByRole("menuitem", { name: "Set number of instances..." }).click();
  const dialog = page.getByRole("dialog", { name: "Copies of the selected object" });
  const field = dialog.getByRole("spinbutton", { name: "Number of copies:" });
  await expect(field).toHaveValue("2");
  await field.fill("3");
  await dialog.getByRole("button", { name: "OK" }).click();
  await expect(cube).toContainText("×3");
  await page.screenshot({ path: `${artifacts}copies.png` });

  // Escape closes the menu without an edit.
  await page.mouse.click(x, y, { button: "right" });
  await expect(objectMenu(page)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(objectMenu(page)).toHaveCount(0);
  const copies = await instances(page);

  // A right drag pans the camera and shows no menu.
  const unpanned = await canvasPixels(page);
  await page.mouse.move(x, y);
  await page.mouse.down({ button: "right" });
  await page.mouse.move(x + 150, y + 60, { steps: 10 });
  await page.mouse.up({ button: "right" });
  await page.waitForTimeout(500);
  await expect(objectMenu(page)).toHaveCount(0);
  const panned = await canvasPixels(page);
  expect(panned).not.toBe(unpanned);
  writeFileSync(`${artifacts}panned.png`, Buffer.from(panned.split(",")[1], "base64"));

  // A right click on empty space shows no menu and clears the selection.
  await page.mouse.click(box.x + 20, box.y + 20, { button: "right" });
  await page.waitForTimeout(500);
  await expect(objectMenu(page)).toHaveCount(0);
  await expect(cube).toHaveAttribute("aria-selected", "false");
  expect(await instances(page)).toBe(copies);

  // The Preview tab has no object menu.
  await page.getByRole("link", { name: "Preview" }).click();
  await page.mouse.click(x, y, { button: "right" });
  await page.waitForTimeout(500);
  await expect(objectMenu(page)).toHaveCount(0);

  const events = await prevented();
  expect(events.length).toBeGreaterThanOrEqual(6);
  expect(events.every(Boolean)).toBe(true);
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    menuAt: [menuBox.x, menuBox.y],
    cursor: [x, y],
    instances: { before, added, copies },
    contextmenuPrevented: events,
  }, null, 2)}\n`);
});
