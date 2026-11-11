import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { act, choose, PRESETS } from "./lhPresets";

// A closed cylinder (binary STL) whose slice takes seconds: 512 facets around, 60 mm tall.
function cylinder(radius: number, height: number, segments: number): Buffer {
  const stl = Buffer.alloc(84 + segments * 4 * 50);
  stl.writeUInt32LE(segments * 4, 80);
  let offset = 84;
  const facet = (...points: number[][]) => {
    offset += 12;
    for (const point of points) for (const value of point) offset = stl.writeFloatLE(value, offset);
    offset += 2;
  };
  for (let i = 0; i < segments; i += 1) {
    const [a, b] = [i, i + 1].map((k) => (2 * Math.PI * k) / segments);
    const [x0, y0, x1, y1] = [Math.cos(a), Math.sin(a), Math.cos(b), Math.sin(b)].map((v) => v * radius);
    facet([0, 0, 0], [x1, y1, 0], [x0, y0, 0]);
    facet([0, 0, height], [x0, y0, height], [x1, y1, height]);
    facet([x0, y0, 0], [x1, y1, 0], [x1, y1, height]);
    facet([x0, y0, 0], [x1, y1, height], [x0, y0, height]);
  }
  return stl;
}

// A frame-sized heartbeat on the main thread: a slice that ran there would stall it for seconds.
const MAX_GAP_MS = 200;
const MIN_SLICE_MS = 2_000;

test("the 3D view caps its pixel ratio at 2 and a large model slices in the worker while the UI responds", async ({ page }, info) => {
  test.setTimeout(600_000);
  const artifacts = fileURLToPath(new URL(`../../target/web-e2e/mobile-worker/${info.project.name}/`, import.meta.url));
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const model = `${artifacts}cylinder.stl`;
  writeFileSync(model, cylinder(20, 60, 512));
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  const canvas = page.locator("[data-testid=viewport] canvas");
  const pixels = () => canvas.evaluate((element: HTMLCanvasElement) => ({
    devicePixelRatio,
    width: element.width,
    expected: Math.floor(element.clientWidth * Math.min(devicePixelRatio, 2)),
  }));
  await expect.poll(async () => (await pixels()).width, "the 3D view has a drawing buffer").toBeGreaterThan(0);
  const { width, expected } = await pixels();
  expect.soft(width, "canvas width is clientWidth x min(devicePixelRatio, 2)").toBe(expected);

  const chooser = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Import", exact: true }).tap();
  await (await chooser).setFiles(model);
  const status = page.getByTestId("status");
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
  // The same LH presets as the touch flow; phones open the sidebar drawer first.
  const panel = page.getByRole("button", { name: "Settings panel", exact: true });
  if (await panel.isVisible()) await act("touch", panel);
  await act("touch", page.getByRole("button", { name: "Printer preset", exact: true }));
  await act("touch", page.getByRole("option", { name: "Select/Remove printers (system presets)" }));
  const wizard = page.getByRole("dialog", { name: "System presets" });
  await act("touch", wizard.getByRole("checkbox", { name: PRESETS.vendor, exact: true }));
  await act("touch", wizard.getByRole("button", { name: "Done" }));
  await choose(page, "touch", "Printer preset", "System presets", PRESETS.printer);
  await choose(page, "touch", "Process preset", "System presets", PRESETS.process);
  await choose(page, "touch", "Filament 1 preset", "System presets", PRESETS.filament);
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  if (await panel.isVisible()) await act("touch", panel);

  await page.evaluate(() => {
    const beat = { start: performance.now(), last: performance.now(), maxGap: 0, beats: 0, end: 0 };
    (window as unknown as { beat: typeof beat }).beat = beat;
    const timer = setInterval(() => {
      const now = performance.now();
      beat.maxGap = Math.max(beat.maxGap, now - beat.last);
      beat.last = now;
      beat.beats += 1;
    }, 16);
    const bar = document.querySelector("[data-testid=status]")!;
    new MutationObserver((_, observer) => {
      if (!bar.textContent?.includes("Slice complete")) return;
      const now = performance.now();
      beat.maxGap = Math.max(beat.maxGap, now - beat.last);
      beat.end = now;
      clearInterval(timer);
      observer.disconnect();
    }).observe(bar, { childList: true, subtree: true, characterData: true });
  });
  await page.getByRole("button", { name: "Slice plate", exact: true }).tap();
  await expect(status).toHaveText(/^Slicing plate 1: /);
  // The UI answers a tap mid-slice.
  await page.getByRole("button", { name: "Light theme" }).tap();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light", { timeout: 1_000 });
  await expect(status, "the theme switched while the plate was still slicing").toHaveText(/^Slicing plate 1: /);
  await expect(status).toContainText("Slice complete", { timeout: 480_000 });
  const beat = await page.evaluate(() => (window as unknown as { beat: { start: number; end: number; maxGap: number; beats: number } }).beat);
  const sliceMs = beat.end - beat.start;
  const report = { project: info.project.name, pixels: await pixels(), sliceMs, maxGapMs: beat.maxGap, beats: beat.beats, workers: page.workers().length };
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify(report, null, 2)}\n`);
  await page.screenshot({ path: `${artifacts}sliced.png` });

  expect(sliceMs, "the model is large enough that a main-thread slice would freeze the UI").toBeGreaterThan(MIN_SLICE_MS);
  expect(beat.maxGap, `no main-thread stall over ${MAX_GAP_MS} ms while slicing`).toBeLessThan(MAX_GAP_MS);
  expect(page.workers().length, "the slicer runs in a Web Worker").toBeGreaterThan(0);
  expect(errors).toEqual([]);
});
