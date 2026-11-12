import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";

const fixture = fileURLToPath(new URL("../../tests/parity/orca_cli_ender3.3mf", import.meta.url));
const sizes = [[32, 24], [64, 48]];

test("downloaded non-BBL G-code contains the configured plate thumbnail images", async ({ page }, info) => {
  const entries = unzipSync(readFileSync(fixture));
  const settings = "Metadata/project_settings.config";
  entries[settings] = strToU8(JSON.stringify({
    ...JSON.parse(strFromU8(entries[settings])),
    thumbnails: "32x24/PNG, 64x48/PNG",
  }));
  const input = Buffer.from(zipSync(entries));
  writeFileSync(info.outputPath("input.3mf"), input);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles({
    name: "thumbnail-images.3mf", mimeType: "application/3mf", buffer: input,
  });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });

  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  const output = info.outputPath("plate.gcode");
  await download.saveAs(output);
  const bytes = readFileSync(output);
  const text = bytes.toString("utf8");
  const blocks = [...text.matchAll(/; thumbnail begin (\d+)x(\d+) (\d+)\n([\s\S]*?); thumbnail end/g)];
  const dimensions = blocks.map((block) => [Number(block[1]), Number(block[2])]);
  writeFileSync(info.outputPath("result.json"), `${JSON.stringify({
    inputSha256: createHash("sha256").update(input).digest("hex"),
    gcodeSha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.length, requested: sizes, dimensions,
    thumbnails: text.match(/^; thumbnails = (.*)$/m)?.[1],
  }, null, 2)}\n`);
  await page.screenshot({ path: info.outputPath("sliced.png") });
  expect(text).toMatch(/^; thumbnails = 32x24\/PNG, ?64x48\/PNG$/m);
  expect(dimensions, "each configured size has a PNG thumbnail block in the downloaded G-code").toEqual(sizes);

  for (const [index, block] of blocks.entries()) {
    const rows = block[4].trimEnd().split("\n");
    for (const row of rows) expect(row).toMatch(/^; [A-Za-z0-9+/=]{1,78}$/);
    const encoded = rows.map((row) => row.slice(2)).join("");
    expect(encoded.length).toBe(Number(block[3]));
    writeFileSync(info.outputPath(`thumbnail-${index}.png`), Buffer.from(encoded, "base64"));
    const image = await page.evaluate(async (base64) => {
      const response = await fetch(`data:image/png;base64,${base64}`);
      const bitmap = await createImageBitmap(await response.blob());
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
      const context = canvas.getContext("2d")!;
      context.drawImage(bitmap, 0, 0);
      const pixels = context.getImageData(0, 0, bitmap.width, bitmap.height).data;
      let visible = 0;
      const colors = new Set<number>();
      for (let pixel = 0; pixel < pixels.length; pixel += 4) {
        if (pixels[pixel + 3] > 0) {
          visible++;
          colors.add((pixels[pixel] << 16) | (pixels[pixel + 1] << 8) | pixels[pixel + 2]);
        }
      }
      const result = { width: bitmap.width, height: bitmap.height, visible, colors: colors.size };
      bitmap.close();
      return result;
    }, encoded);
    expect([image.width, image.height]).toEqual(sizes[index]);
    expect(image.visible, "thumbnail contains rendered geometry").toBeGreaterThan(0);
    expect(image.colors, "thumbnail is not a flat background").toBeGreaterThan(1);
  }
});
