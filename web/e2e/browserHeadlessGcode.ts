import { expect, type Page } from "@playwright/test";

const imageMarker =
  /^;\s*(?:THUMBNAIL_BLOCK_\w+|thumbnail(?:_\w+)?\s+(?:begin|end)\b|bigtree thumbnail end\b|[gs]image:)/im;
const headerEnd = "; HEADER_BLOCK_END\n\n";
const pngSignature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

/**
 * ARD-0113: web opts in to thumbnails; headless CLI references may have none.
 * These three comparisons use PNG-only fixtures. Keep the existing comparison
 * of all other bytes, and never strip anything when both streams can be compared as-is.
 */
export async function browserGcodeForHeadlessComparison(page: Page, browser: string, reference: string) {
  if (imageMarker.test(reference)) return browser;

  const definitions = [...reference.matchAll(/^; thumbnails =([^\r\n]*)$/gm)];
  expect(definitions.length, "one effective thumbnail setting at most").toBeLessThanOrEqual(1);
  const configured = definitions[0]?.[1].trim();
  if (!configured) {
    expect(browser, "no image blocks without thumbnail definitions").not.toMatch(imageMarker);
    return browser;
  }

  const header = browser.indexOf(headerEnd);
  expect(header, "image blocks follow the G-code header").toBeGreaterThanOrEqual(0);
  const first = header + headerEnd.length;
  const block =
    /; THUMBNAIL_BLOCK_START\n\n;\n; thumbnail begin ([1-9]\d*)x([1-9]\d*) ([1-9]\d*)\n((?:; [A-Za-z0-9+/=]{1,78}\n)+); thumbnail end\n; THUMBNAIL_BLOCK_END\n\n/y;
  block.lastIndex = first;
  for (const definition of configured.split(",")) {
    const expected = /^([1-9]\d*)x([1-9]\d*)\/PNG$/.exec(definition.trim());
    if (!expected) throw new Error(`Expected an explicit PNG thumbnail definition, got ${definition}`);
    const image = block.exec(browser);
    if (!image) throw new Error(`Expected a complete PNG thumbnail block for ${definition}`);
    expect(image.slice(1, 3), "thumbnail dimensions and configuration order").toEqual(expected.slice(1, 3));
    const rows = image[4]
      .trimEnd()
      .split("\n")
      .map((row) => row.slice(2));
    expect(
      rows.slice(0, -1).every((row) => row.length === 78),
      "all but the final image row have 78 characters",
    ).toBe(true);
    const encoded = rows.join("");
    const bytes = Buffer.from(encoded, "base64");
    expect(encoded.length, "declared image length").toBe(Number(image[3]));
    expect(bytes.toString("base64"), "canonical base64 image payload").toBe(encoded);
    expect(bytes.subarray(0, pngSignature.length), "PNG signature").toEqual(pngSignature);
    const dimensions = await page.evaluate(async (base64) => {
      const response = await fetch(`data:image/png;base64,${base64}`);
      const bitmap = await createImageBitmap(await response.blob());
      const size = [bitmap.width, bitmap.height];
      bitmap.close();
      return size;
    }, encoded);
    expect(dimensions, "decoded PNG dimensions").toEqual(expected.slice(1, 3).map(Number));
  }

  const nonImage = browser.slice(0, first) + browser.slice(block.lastIndex);
  expect(nonImage, "no extra, misplaced or partial image blocks").not.toMatch(imageMarker);
  return nonImage;
}
