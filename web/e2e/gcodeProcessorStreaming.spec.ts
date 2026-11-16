import { readFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

function cube200Stl() {
  const scale = (point: number[]) => point.map((coordinate) => coordinate * 200);
  const quads: [number[][], number[]][] = [
    [[[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]], [0, 0, -1]],
    [[[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]], [0, 0, 1]],
    [[[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]], [0, -1, 0]],
    [[[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]], [0, 1, 0]],
    [[[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]], [-1, 0, 0]],
    [[[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]], [1, 0, 0]],
  ];
  const buffer = Buffer.alloc(84 + 12 * 50);
  buffer.write("cube200", 0, "ascii");
  buffer.writeUInt32LE(12, 80);
  let offset = 84;
  for (const [quad, normal] of quads) {
    for (const triangle of [[quad[0], quad[1], quad[2]], [quad[0], quad[2], quad[3]]]) {
      for (const coordinate of normal) {
        buffer.writeFloatLE(coordinate, offset);
        offset += 4;
      }
      for (const point of triangle) {
        for (const coordinate of scale(point)) {
          buffer.writeFloatLE(coordinate, offset);
          offset += 4;
        }
      }
      offset += 2;
    }
  }
  return buffer;
}

test("slices a runtime cube and exposes matching viewer layers", async ({ page }, testInfo) => {
  test.setTimeout(3_600_000);
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles({
    name: "cube200.stl",
    mimeType: "model/stl",
    buffer: cube200Stl(),
  });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 120_000 });
  await page.getByRole("button", { name: "Slice plate" }).click();
  const status = page.getByTestId("status");
  await expect(status).toContainText(/Slice complete|Slicing failed/, { timeout: 600_000 });
  expect(await status.textContent()).toContain("Slice complete");
  await page.getByRole("link", { name: "Preview" }).click();
  const layerCount = page.getByTestId("layer-count");
  await expect(layerCount).toBeVisible({ timeout: 600_000 });
  const viewerLayers = Number(await layerCount.textContent());
  expect(viewerLayers).toBeGreaterThan(0);

  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export G-code file" }).click();
  const exported = await download;
  await exported.saveAs(testInfo.outputPath("cube200.gcode"));
  const gcodePath = await exported.path();
  expect(gcodePath).not.toBeNull();
  const gcode = readFileSync(gcodePath!, "utf8");
  const header = gcode.match(/^; total layer number: (\d+)$/m);
  expect(header).not.toBeNull();
  const headerLayers = Number(header![1]);
  expect(headerLayers).toBe(viewerLayers);
  await testInfo.attach("cube200-layers", {
    body: JSON.stringify({ viewerLayers, headerLayers, gcodeBytes: Buffer.byteLength(gcode) }),
    contentType: "application/json",
  });
});
