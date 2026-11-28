import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expect, type Locator, type Page } from "@playwright/test";

/**
 * The Design canvas workspace's dialog artboards (design/README.md): local only, since `/design/` is
 * git-ignored, so the parity run happens where the workspace exists.
 */
export const designRoot = fileURLToPath(new URL("../../../design/ares-slicer-dialogs/", import.meta.url));

interface ArtboardFile {
  id: string;
  title: string;
  file: string;
  width: number;
  height: number;
}

export interface Artboard extends ArtboardFile {
  theme: "light" | "dark";
}

/** The dialog of every artboard: the `<dialog class="dialog">`, or the arrange `<aside class="popup">`. */
export const ROOT = "dialog.dialog[open], .popup";

export function artboards(): Artboard[] {
  if (!existsSync(`${designRoot}design.json`)) return [];
  const { artboards: files } = JSON.parse(readFileSync(`${designRoot}design.json`, "utf8")) as {
    artboards: ArtboardFile[];
  };
  return files.map((file) => ({
    ...file,
    theme: /<html[^>]+data-theme="dark"/.test(readFileSync(`${designRoot}${file.file}`, "utf8")) ? "dark" : "light",
  }));
}

/** The artboard rendered at its own size, as the Design canvas shows it. */
export async function renderReference(page: Page, artboard: Artboard): Promise<Locator> {
  await page.setViewportSize({ width: artboard.width, height: artboard.height });
  await page.goto(pathToFileURL(`${designRoot}${artboard.file}`).href);
  const root = page.locator(ROOT).first();
  await expect(root).toBeVisible();
  await settle(page);
  return root;
}

/** The artboard's nested dialog, when it shows one on top of the first (the wizard's Error). */
export async function nestedReference(page: Page): Promise<Locator | null> {
  const nested = page.locator("[data-nested][open]");
  return (await nested.count()) ? nested.first() : null;
}

/** Neutral focus and pointer, then the frames in flight: what a screenshot of a dialog at rest shows. */
export async function settle(page: Page) {
  await page.mouse.move(0, 0);
  await page.evaluate(() => document.fonts.ready);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.evaluate(
    () => new Promise<void>((done) => requestAnimationFrame(() => requestAnimationFrame(() => done()))),
  );
}
