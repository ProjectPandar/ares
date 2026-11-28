import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Locator, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { diffStyles, type ArtboardResult, pixelDiff, writeGallery } from "./dialog-design/compare";
import { artboards, nestedReference, renderReference, settle } from "./dialog-design/design";
import { readStyles } from "./dialog-design/probe";
import { OPENINGS } from "./dialog-design/states";

/**
 * Every dialog artboard of the Design canvas workspace (design/ares-slicer-dialogs) beside the app
 * in the same state, theme and viewport: the computed styles of each part of the design's anatomy,
 * and the dialog's pixels. The gallery is target/web-e2e/dialog-design/index.html.
 */
const artifacts = fileURLToPath(new URL("../../target/web-e2e/dialog-design/", import.meta.url));
const boards = artboards();
// Each opening seeds the app config its artboard shows.
test.use({ firstRunConfig: false });

async function capture(target: Page | Locator, directory: string, name: string) {
  await target.screenshot({ path: `${directory}${name}.png` });
  return readFileSync(`${directory}${name}.png`);
}

for (const artboard of boards) {
  const opening = OPENINGS[artboard.id];
  test(`dialog design parity: ${artboard.id}`, async ({ page, context, baseURL }) => {
    test.setTimeout(180_000);
    const directory = `${artifacts}${artboard.id}/`;
    mkdirSync(directory, { recursive: true });

    const reference = await renderReference(page, artboard);
    await page.screenshot({ path: `${directory}reference.png` });
    const designPng = await capture(reference, directory, "dialog-reference");
    const design = await readStyles(reference);
    const nested = await nestedReference(page);
    const nestedDesign = nested && (await readStyles(nested));

    expect(opening.theme, "the opening's theme is the artboard's").toBe(artboard.theme);
    await page.setViewportSize({ width: artboard.width, height: artboard.height });
    const root = await opening.open({ page, context, baseURL });
    await settle(page);
    await page.screenshot({ path: `${directory}actual.png` });
    const appPng = await capture(root, directory, "dialog-actual");
    const app = await readStyles(root, opening.appSelectors);
    const nestedApp =
      nestedDesign && (await readStyles(page.locator("dialog.dialog[open]").nth(1), opening.appSelectors));
    if (nestedDesign && nestedApp) {
      for (const [part, styles] of Object.entries(nestedDesign)) design[`nested ${part}`] = styles;
      for (const [part, styles] of Object.entries(nestedApp)) app[`nested ${part}`] = styles;
    }

    const canvas = await context.newPage();
    const { png, ...pixels } = await pixelDiff(canvas, designPng, appPng);
    await canvas.close();
    writeFileSync(`${directory}diff.png`, png);
    const styles = diffStyles(design, app, opening.ignored);
    const result: ArtboardResult = {
      id: artboard.id,
      title: artboard.title,
      theme: artboard.theme,
      viewport: { width: artboard.width, height: artboard.height },
      styles,
      pixels,
      notes: opening.notes,
    };
    writeFileSync(`${directory}result.json`, `${JSON.stringify(result, null, 2)}\n`);
    writeGallery(
      artifacts,
      boards.map(({ id }) => id),
    );
    expect.soft(styles.mismatches, "the dialog's computed styles equal the artboard's").toBe(0);
  });
}

if (boards.length === 0)
  test("dialog design parity", () => {
    test.skip(true, "design/ (the Design canvas workspace) is not in this checkout");
  });
