import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Browser, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const artifacts = fileURLToPath(new URL("../../target/web-e2e/viewport-resize/", import.meta.url));

/** The canvas as one resize's frame paints it, read when the resize observers of that frame ran. */
interface Sample {
  width: number;
  height: number;
  /** Fraction of the canvas pixels that are opaque: 0 for a cleared drawing buffer. */
  opaque: number;
  colours: number;
}

const sizes = [
  [1400, 860],
  [1200, 800],
  [1000, 760],
  [1180, 700],
  [1366, 768],
  [1600, 900],
];

/**
 * Resizing the canvas clears its drawing buffer. A ResizeObserver created after the viewport's
 * runs after it in the same rendering update, right before the frame is painted, so what it
 * reads is what that frame shows.
 */
function observeFrames(page: Page) {
  return page.getByTestId("viewport").evaluate((container) => {
    const samples: Sample[] = [];
    (window as unknown as { resizeSamples: Sample[] }).resizeSamples = samples;
    const probe = document.createElement("canvas");
    probe.width = 64;
    probe.height = 64;
    const context = probe.getContext("2d", { willReadFrequently: true })!;
    new ResizeObserver(() => {
      const canvas = container.querySelector("canvas")!;
      context.clearRect(0, 0, 64, 64);
      context.drawImage(canvas, 0, 0, 64, 64);
      const pixels = context.getImageData(0, 0, 64, 64).data;
      const colours = new Set<number>();
      let opaque = 0;
      for (let index = 0; index < pixels.length; index += 4) {
        if (pixels[index + 3] === 255) opaque++;
        colours.add((pixels[index] << 16) | (pixels[index + 1] << 8) | pixels[index + 2]);
      }
      samples.push({ width: canvas.width, height: canvas.height, opaque: opaque / (64 * 64), colours: colours.size });
    }).observe(container);
  });
}

/** The viewport's distances to the window edges; they stay fixed while the window resizes. */
interface Margins {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/**
 * The fraction of the middle of the viewport, in each screencast frame, darker than the empty
 * viewport background: the plate covers most of it, a blank canvas none.
 */
async function plateCoverage(browser: Browser, frames: string[], margins: Margins, background: number) {
  const page = await browser.newPage();
  const coverage = await page.evaluate(
    async ({ images, edges, empty }) => {
      const probe = document.createElement("canvas");
      probe.width = 64;
      probe.height = 64;
      const context = probe.getContext("2d", { willReadFrequently: true })!;
      const fractions: number[] = [];
      for (const data of images) {
        const image = new Image();
        image.src = `data:image/png;base64,${data}`;
        await image.decode();
        const width = image.naturalWidth - edges.left - edges.right;
        const height = image.naturalHeight - edges.top - edges.bottom;
        const [x, y] = [edges.left + width / 4, edges.top + height / 4];
        context.drawImage(image, x, y, width / 2, height / 2, 0, 0, 64, 64);
        const pixels = context.getImageData(0, 0, 64, 64).data;
        let dark = 0;
        for (let index = 0; index < pixels.length; index += 4) {
          if (pixels[index] + pixels[index + 1] + pixels[index + 2] < empty - 30) dark++;
        }
        fractions.push(dark / (64 * 64));
      }
      return fractions;
    },
    { images: frames, edges: margins, empty: background },
  );
  await page.close();
  return coverage;
}

const samples = (page: Page) =>
  page.evaluate(() => (window as unknown as { resizeSamples: Sample[] }).resizeSamples.slice());

test("a resized viewport paints the plate in the frame of the resize", async ({ browser, page }) => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await observeFrames(page);
  await expect.poll(async () => (await samples(page)).length).toBe(1);
  await page.screenshot({ path: `${artifacts}before.png` });
  const margins = await page.getByTestId("viewport").evaluate((container): Margins => {
    const box = container.getBoundingClientRect();
    return { left: box.left, top: box.top, right: innerWidth - box.right, bottom: innerHeight - box.bottom };
  });
  // The brightness of the empty viewport: the corner of the canvas shows the scene background.
  const background = await page.getByTestId("viewport").evaluate((container) => {
    const probe = document.createElement("canvas").getContext("2d")!;
    probe.drawImage(container.querySelector("canvas")!, 0, 0, 1, 1, 0, 0, 1, 1);
    const [red, green, blue] = probe.getImageData(0, 0, 1, 1).data;
    return red + green + blue;
  });

  // Every composited frame of the resizes.
  const screencast = await page.context().newCDPSession(page);
  const composited: string[] = [];
  screencast.on("Page.screencastFrame", ({ data, sessionId }) => {
    void screencast.send("Page.screencastFrameAck", { sessionId });
    composited.push(data);
  });
  await screencast.send("Page.startScreencast", { format: "png", everyNthFrame: 1 });

  for (const [index, [width, height]] of sizes.entries()) {
    const seen = (await samples(page)).length;
    await page.setViewportSize({ width, height });
    await expect.poll(async () => (await samples(page)).length).toBeGreaterThan(seen);
    await page.screenshot({ path: `${artifacts}resize-${index}-${width}x${height}.png` });
  }
  await screencast.send("Page.stopScreencast");
  mkdirSync(`${artifacts}screencast`, { recursive: true });
  for (const [index, data] of composited.entries()) {
    writeFileSync(`${artifacts}screencast/${String(index).padStart(3, "0")}.png`, Buffer.from(data, "base64"));
  }

  const coverage = await plateCoverage(browser, composited, margins, background);
  const frames = await samples(page);
  writeFileSync(`${artifacts}frames.json`, `${JSON.stringify(frames, null, 2)}\n`);
  writeFileSync(`${artifacts}screencast-plate-coverage.json`, `${JSON.stringify(coverage, null, 2)}\n`);
  for (const frame of frames) {
    expect(frame.opaque, JSON.stringify(frame)).toBe(1);
    expect(frame.colours, JSON.stringify(frame)).toBeGreaterThan(8);
  }
  expect(composited.length).toBeGreaterThan(sizes.length);
  for (const [index, fraction] of coverage.entries()) expect(fraction, `screencast/${index}`).toBeGreaterThan(0.5);
});
