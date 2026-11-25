import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { choose, PRESETS } from "./lhPresets";
import { shownTab } from "./shownTab";

// A closed cylinder (binary STL), 512 facets around and 60 mm tall: its toolpaths have about a million vertices.
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

async function openWithPresets(page: Page) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog");
  await wizard.getByRole("button", { name: PRESETS.vendor, exact: true }).click();
  await wizard.getByRole("checkbox", { name: PRESETS.printer, exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm", exact: true }).click();
  await choose(page, "mouse", "Printer preset", "System presets", PRESETS.printer);
  await choose(page, "mouse", "Process preset", "System presets", PRESETS.process);
  await choose(page, "mouse", "Filament 1 preset", "System presets", PRESETS.filament);
}

// The threshold of worker.mobile.spec.ts: a frame-sized heartbeat must not stall for this long.
const MAX_GAP_MS = 200;

// Ctrl+R slices and selects Preview (MainFrame.cpp:2411-2415), so the slice result reaches an open Preview. Its
// toolpath textures and E2E probes are prepared in the slicer worker; the main thread only uploads them.
test("a slice result that reaches an open Preview leaves the main thread responsive", async ({ page }, info) => {
  test.setTimeout(600_000);
  const artifacts = fileURLToPath(
    new URL(`../../target/web-e2e/preview-result-responsive/${info.project.name}/`, import.meta.url),
  );
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await openWithPresets(page);
  await page
    .getByTestId("import-input")
    .setInputFiles({ name: "cylinder.stl", buffer: cylinder(20, 60, 512), mimeType: "model/stl" });
  const status = page.getByTestId("status");
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });

  await page.keyboard.press("ControlOrMeta+r");
  await expect(page).toHaveURL(/#\/preview$/);
  const viewport = shownTab(page).getByTestId("viewport");
  await expect(viewport.locator("canvas")).toBeVisible();
  // From the open Preview until the task that sets the toolpaths, which writes data-segment-sizes.
  const started = await page.evaluate(() => {
    const beat = { last: performance.now(), maxGap: 0, beats: 0, done: false };
    (window as unknown as { beat: typeof beat }).beat = beat;
    const timer = setInterval(() => {
      const now = performance.now();
      beat.maxGap = Math.max(beat.maxGap, now - beat.last);
      beat.last = now;
      beat.beats += 1;
    }, 16);
    const view = document.querySelector(".tab-pane:not([hidden]) [data-testid=viewport]")!;
    new MutationObserver((_, observer) => {
      if (!view.hasAttribute("data-segment-sizes")) return;
      beat.maxGap = Math.max(beat.maxGap, performance.now() - beat.last);
      beat.done = true;
      clearInterval(timer);
      observer.disconnect();
    }).observe(view, { attributes: true, attributeFilter: ["data-segment-sizes"] });
    return {
      slicing: document.querySelector("[data-testid=status]")!.textContent,
      sized: view.hasAttribute("data-segment-sizes"),
    };
  });
  expect(started.slicing, "the heartbeat starts while the plate still slices").toMatch(/^Slicing plate/);
  expect(started.sized, "no toolpaths were set before the heartbeat started").toBe(false);
  await expect(viewport).toHaveAttribute("data-segment-sizes", { timeout: 480_000 });
  await expect(status).toContainText("Slice complete");
  const beat = await page.evaluate(
    () => (window as unknown as { beat: { maxGap: number; beats: number; done: boolean } }).beat,
  );
  const sizes = Number((await viewport.getAttribute("data-segment-sizes"))!.split(":")[0]);
  writeFileSync(
    `${artifacts}result.json`,
    `${JSON.stringify({ project: info.project.name, maxGapMs: beat.maxGap, beats: beat.beats, segmentSizes: sizes }, null, 2)}\n`,
  );

  expect(beat.done).toBe(true);
  expect(beat.maxGap, `no main-thread stall over ${MAX_GAP_MS} ms when the slice result reaches Preview`).toBeLessThan(
    MAX_GAP_MS,
  );
  expect(errors).toEqual([]);
});
