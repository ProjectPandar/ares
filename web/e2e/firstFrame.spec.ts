import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/first-frame/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;

/** Distinct colours of the WebGL canvas: 1 while nothing has been drawn. */
function canvasColours(page: Page) {
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const probe = document.createElement("canvas");
    probe.width = 64;
    probe.height = 64;
    const context = probe.getContext("2d")!;
    context.drawImage(canvas, 0, 0, 64, 64);
    const pixels = context.getImageData(0, 0, 64, 64).data;
    const colours = new Set<number>();
    for (let index = 0; index < pixels.length; index += 4) {
      colours.add((pixels[index] << 16) | (pixels[index + 1] << 8) | pixels[index + 2]);
    }
    return colours.size;
  });
}

const renders = (page: Page) => page.evaluate(() => (window as unknown as { renders: number }).renders);

// Under load the compositor stops issuing frames for seconds while the main thread is idle
// (requestAnimationFrame callbacks measured 7.6 s late, headless SwiftShader at load ~180).
// OrcaSlicer renders a changed canvas immediately, not on a display frame
// (GLCanvas3D.cpp:4810-4811); the viewport must draw without animation frames.
test("the viewport draws without animation frames, once per change", async ({ page }) => {
  await page.addInitScript(() => {
    const state = window as unknown as { renders: number };
    state.renders = 0;
    // A stalled compositor: animation frames never come.
    window.requestAnimationFrame = () => 0;
    // WebGLRenderer::render clears once per frame (autoClear).
    const clear = WebGL2RenderingContext.prototype.clear;
    WebGL2RenderingContext.prototype.clear = function (mask) {
      state.renders++;
      clear.call(this, mask);
    };
  });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect.poll(() => canvasColours(page)).toBeGreaterThan(8);
  await page.screenshot({ path: `${artifacts}prepare.png` });

  // A drag of the model at the canvas centre: each pointer move renders at most once.
  const box = (await page.getByTestId("viewport").boundingBox())!;
  const instances = () => page.locator(".viewport").getAttribute("data-instances");
  const before = await instances();
  const [x, y] = [box.x + box.width / 2, box.y + box.height / 2];
  await page.mouse.move(x, y);
  await page.mouse.down();
  const beforeDrag = await renders(page);
  const moves = 10;
  await page.mouse.move(x + 200, y + 50, { steps: moves });
  await page.mouse.up();
  const dragRenders = (await renders(page)) - beforeDrag;
  expect(dragRenders).toBeGreaterThan(0);
  expect(dragRenders).toBeLessThanOrEqual(moves);
  // The release commits the drag once (GLCanvas3D::do_move), and that change renders.
  await expect.poll(instances).not.toBe(before);
  await expect(page.getByTestId("viewport")).not.toHaveAttribute("data-drawing");

  // Nothing renders while nothing changes.
  const idle = await renders(page);
  await page.waitForTimeout(1_000);
  expect(await renders(page)).toBe(idle);

  // The Preview canvas is a new viewport; its first frame shows the toolpaths.
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("layer-count")).toBeVisible();
  await expect.poll(() => canvasColours(page)).toBeGreaterThan(8);
  await page.screenshot({ path: `${artifacts}preview.png` });
});
