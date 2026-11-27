import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { expect, type Page, type TestInfo } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";

const ender = new URL("../../tests/parity/orca_cli_ender3.3mf", import.meta.url);
const bbl = new URL("../../tests/parity/adaptive-octree-spacing/baseline.3mf", import.meta.url);
const definitions = "32x32/PNG, 64x48/JPG";
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

async function sliceDownload(page: Page, info: TestInfo, fixture: URL, thumbnails: string) {
  const entries = unzipSync(readFileSync(fixture));
  const settings = "Metadata/project_settings.config";
  entries[settings] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[settings])), thumbnails }));
  const input = Buffer.from(zipSync(entries));
  writeFileSync(info.outputPath("input.3mf"), input);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles({
    name: "thumbnail-images.3mf",
    mimeType: "application/3mf",
    buffer: input,
  });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);
  const path = info.outputPath("plate.gcode");
  await download.saveAs(path);
  const bytes = readFileSync(path);
  writeFileSync(info.outputPath("hashes.json"), JSON.stringify({ input: sha256(input), gcode: sha256(bytes) }));
  return { path, gcode: bytes.toString("utf8") };
}

function moonrakerMetadata(path: string) {
  const {
    ARES_MOONRAKER_METADATA_PY: parser,
    ARES_MOONRAKER_PYTHON: python,
    ARES_MOONRAKER_PILLOW_PATH: pillow,
  } = process.env;
  if (!parser || !python || !pillow) throw new Error("Configure the pinned Moonraker extractor and Pillow");
  expect(sha256(readFileSync(parser))).toBe("ae2488ff23e6ddcaf961b063506dad1a7141b38b92102bc174422ed244641d2e");
  const extractor = [
    "import importlib.util, json, sys",
    "sys.path.insert(0, sys.argv[2])",
    "spec = importlib.util.spec_from_file_location('moonraker_metadata', sys.argv[1])",
    "module = importlib.util.module_from_spec(spec)",
    "spec.loader.exec_module(module)",
    "print(json.dumps(module.extract_metadata(sys.argv[3], [])))",
  ].join("\n");
  const result = execFileSync(python, ["-I", "-B", "-c", extractor, parser, pillow, path], { encoding: "utf8" });
  writeFileSync(resolve(dirname(path), "moonraker.json"), result);
  return JSON.parse(result) as { thumbnails: { width: number; height: number; size: number; relative_path: string }[] };
}

test("downloaded non-BBL G-code contains the configured plate thumbnail images", async ({ page }, info) => {
  const { path, gcode } = await sliceDownload(page, info, ender, definitions);
  expect(gcode).toMatch(/^; thumbnails = 32x32\/PNG, ?64x48\/JPG$/m);
  const blocks = [...gcode.matchAll(/^; (thumbnail(?:_JPG)?) begin (\d+)x(\d+) (\d+)\n([\s\S]*?)^; \1 end$/gm)];
  const sizes = [
    [32, 32],
    [64, 48],
  ];
  expect(blocks.map(([, format, w, h]) => [format, Number(w), Number(h)])).toEqual([
    ["thumbnail", ...sizes[0]],
    ["thumbnail_JPG", ...sizes[1]],
  ]);
  const files: Buffer[] = [];
  for (const [index, [, format, , , length, payload]] of blocks.entries()) {
    const rows = payload.trimEnd().split("\n");
    for (const row of rows) expect(row).toMatch(/^; [a-z\d+/=]{1,78}$/i);
    const encoded = rows.map((row) => row.slice(2)).join("");
    expect(encoded.length).toBe(Number(length));
    const extension = format === "thumbnail" ? "png" : "jpg";
    const bytes = Buffer.from(encoded, "base64");
    writeFileSync(info.outputPath(`thumbnail-${index}.${extension}`), bytes);
    files.push(bytes);
    const image = await page.evaluate(
      async ({ base64, mime }) => {
        const response = await fetch(`data:${mime};base64,${base64}`);
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
        const result = { size: [bitmap.width, bitmap.height], visible, colors: colors.size };
        bitmap.close();
        return result;
      },
      { base64: encoded, mime: extension === "jpg" ? "image/jpeg" : "image/png" },
    );
    expect(image.size).toEqual(sizes[index]);
    expect(image.visible).toBeGreaterThan(0);
    expect(image.colors).toBeGreaterThan(1);
    if (extension === "png") expect(image.visible).toBeLessThan(sizes[index][0] * sizes[index][1]);
  }
  const { thumbnails } = moonrakerMetadata(path);
  expect(thumbnails.map(({ width, height, size }) => [width, height, size])).toEqual(
    sizes.map(([width, height], index) => [width, height, files[index].length]),
  );
  for (const [index, { relative_path }] of thumbnails.entries()) {
    expect(readFileSync(resolve(dirname(path), relative_path))).toEqual(files[index]);
  }
});

test("downloaded BBL G-code omits configured thumbnails", async ({ page }, info) => {
  const { gcode } = await sliceDownload(page, info, bbl, definitions);
  expect(gcode).toMatch(/^; thumbnails = 32x32\/PNG, ?64x48\/JPG$/m);
  expect(gcode).not.toContain("; THUMBNAIL_BLOCK_START");
});

test("downloaded G-code without thumbnail definitions has no image blocks", async ({ page }, info) => {
  const { gcode } = await sliceDownload(page, info, ender, "");
  expect(gcode).toMatch(/^; thumbnails = ?$/m);
  expect(gcode).toContain("; EXECUTABLE_BLOCK_START");
  expect(gcode).not.toContain("; THUMBNAIL_BLOCK_START");
});
