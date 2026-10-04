import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";
import { oracleToolpaths } from "./oracleToolpaths";

/** Hash of the downscaled viewport canvas, to detect view changes. */
function canvasDigest(page: Page) {
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const probe = document.createElement("canvas");
    probe.width = 256;
    probe.height = 144;
    const context = probe.getContext("2d")!;
    context.drawImage(canvas, 0, 0, 256, 144);
    return Array.from(context.getImageData(0, 0, 256, 144).data).join(",");
  }).then((pixels) => createHash("sha256").update(pixels).digest("hex"));
}

/**
 * Feeds upstream GCodeProcessor moves for the sliced G-code into the preview
 * and drives the layer slider and legend against them.
 */
export async function checkToolpathPreview(page: Page, oracle: string, gcodePath: string, artifacts: string, errors: string[]) {
  const toolpaths = oracleToolpaths(oracle, gcodePath, artifacts);
  const layers = toolpaths.layerZs.length;
  const extrudeRoles = new Set(toolpaths.roles.filter((_, index) => toolpaths.moveTypes[index] === 10));
  expect(layers).toBe(50);

  await page.evaluate((data) => {
    const typed = {
      vertexCount: data.vertexCount,
      positions: Float32Array.from(data.positions),
      widths: Float32Array.from(data.widths),
      heights: Float32Array.from(data.heights),
      feedrates: Float32Array.from(data.feedrates),
      fanSpeeds: Float32Array.from(data.fanSpeeds),
      temperatures: Float32Array.from(data.temperatures),
      times: Float32Array.from(data.times),
      roles: Uint8Array.from(data.roles),
      moveTypes: Uint8Array.from(data.moveTypes),
      extruderIds: Uint8Array.from(data.extruderIds),
      colorIds: Uint8Array.from(data.colorIds),
      layerIds: Uint32Array.from(data.layerIds),
      layerZs: Float32Array.from(data.layerZs),
      layerRanges: Uint32Array.from(data.layerRanges),
      toolColors: [],
      colorPrintColors: [],
      spiralVaseMode: false,
    };
    (window as unknown as { aresE2E: { showToolpaths(value: unknown): void } }).aresE2E.showToolpaths(typed);
  }, toolpaths);

  await expect(page.getByTestId("layer-count")).toHaveText(String(layers));
  const legend = page.getByTestId("legend");
  await expect(legend.getByRole("button")).toHaveCount(extrudeRoles.size + 2);
  for (const label of ["Outer wall", "Inner wall", "Sparse infill", "Travel"]) {
    await expect(legend).toContainText(label);
  }
  const top = page.getByRole("slider", { name: "Top layer" });
  await expect(top).toHaveAttribute("aria-valuenow", String(layers));
  const full = await canvasDigest(page);
  await page.screenshot({ path: `${artifacts}preview-toolpaths.png` });

  expect(errors, "page errors after loading toolpaths").toEqual([]);
  await top.focus();
  await page.keyboard.press("Home");
  await expect(top).toHaveAttribute("aria-valuenow", "1");
  await expect(page.getByTestId("layer-slider")).toContainText(toolpaths.layerZs[0].toFixed(2));
  await expect.poll(() => canvasDigest(page), { message: "first layer only must differ from all layers" }).not.toBe(full);
  await page.keyboard.press("PageUp");
  await page.keyboard.press("PageUp");
  await expect(top).toHaveAttribute("aria-valuenow", "21");
  await page.screenshot({ path: `${artifacts}preview-layer-21.png` });
  await page.keyboard.press("End");
  await expect(top).toHaveAttribute("aria-valuenow", String(layers));
  await expect.poll(() => canvasDigest(page)).toBe(full);

  const outerWall = legend.getByRole("button", { name: "Outer wall" });
  await outerWall.click();
  await expect(outerWall).toHaveAttribute("aria-pressed", "false");
  await expect.poll(() => canvasDigest(page), { message: "hiding outer walls must change the view" }).not.toBe(full);
  await page.screenshot({ path: `${artifacts}preview-no-outer-wall.png` });
  await outerWall.click();
  await expect.poll(() => canvasDigest(page)).toBe(full);

  writeFileSync(
    `${artifacts}toolpaths.json`,
    `${JSON.stringify({ vertices: toolpaths.vertexCount, layers, roles: [...extrudeRoles].sort((a, b) => a - b) }, null, 2)}\n`,
  );
}
