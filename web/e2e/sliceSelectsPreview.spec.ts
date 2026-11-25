import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { shownTab } from "./shownTab";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const action = (page: Page, name: string) => page.getByRole("navigation").getByRole("button", { name, exact: true });

/** Records the status line at the moment the Preview tab becomes active. */
async function watchPreviewSwitch(page: Page) {
  await page.getByRole("link", { name: "Preview" }).evaluate((tab) => {
    new MutationObserver((_, observer) => {
      if (!tab.classList.contains("active")) return;
      (window as { statusAtSwitch?: string | null }).statusAtSwitch = document.querySelector("[data-testid=status]")!.textContent;
      observer.disconnect();
    }).observe(tab, { attributes: true, attributeFilter: ["class"] });
  });
}

/** The recorded status: the switch happened, and before the slice finished. */
async function expectSwitchBeforeComplete(page: Page) {
  const status = await page.evaluate(() => (window as { statusAtSwitch?: string | null }).statusAtSwitch);
  expect(status, "Preview tab activation recorded").toEqual(expect.any(String));
  expect(status).not.toContain("Slice complete");
}

// MainFrame.cpp:2411-2415: the Slice button posts the slice event and selects Preview at once, so
// the tab switches before the slice finishes.
test("Slice plate selects the Preview tab as the slice starts", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/cube10.stl`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await expect(page).toHaveURL(/#\/prepare$/);

  await watchPreviewSwitch(page);
  await action(page, "Slice plate").click();
  await expect(page).toHaveURL(/#\/preview$/);
  await expect(page.getByRole("link", { name: "Preview" })).toHaveClass(/active/);
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  await expectSwitchBeforeComplete(page);
  await expect(page.getByTestId("layer-count")).toHaveText(/^[1-9]\d*$/);
});

test("Slice all selects the Preview tab and ends on the last plate", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/custom-gcode-overview/two-plates.3mf`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await expect(page).toHaveURL(/#\/prepare$/);

  await watchPreviewSwitch(page);
  await action(page, "Slice all").click();
  await expect(page).toHaveURL(/#\/preview$/);
  await expect(page.getByTestId("slice-plate")).toHaveText("2", { timeout: 120_000 });
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  await expectSwitchBeforeComplete(page);
});

// OpenGLManager::init_glcontext (OpenGLManager.cpp:352-378) creates one context and View3D and
// Preview both draw with it (GUI_Preview.cpp:90, 289): the tab switch reuses the Prepare canvas's
// context and the programs linked in it.
test("the Preview canvas draws with the context of the Prepare canvas", async ({ page }) => {
  type Marked = WebGL2RenderingContext & { prepareMark?: boolean };
  const canvas = shownTab(page).getByTestId("viewport").locator("canvas");
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/cube10.stl`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await canvas.evaluate((node: HTMLCanvasElement) => {
    (node.getContext("webgl2") as Marked).prepareMark = true;
  });

  await action(page, "Slice plate").click();
  await expect(page).toHaveURL(/#\/preview$/);
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 120_000 });
  const marked = await canvas.evaluate((node: HTMLCanvasElement) => (node.getContext("webgl2") as Marked).prepareMark === true);
  expect(marked, "the Preview canvas has the Prepare canvas's WebGL context").toBe(true);
});
