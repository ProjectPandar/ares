import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const artifacts = fileURLToPath(new URL("../../target/web-e2e/canvas-hint/", import.meta.url));

interface Ratio {
  title: number;
  sub: number;
}

/** WCAG contrast of the hint's title and sub-line against the bed (`--bed`), as the browser resolves them. */
function contrast(page: Page): Promise<Ratio> {
  return page.evaluate(() => {
    const context = document.createElement("canvas").getContext("2d", { willReadFrequently: true })!;
    const rgb = (css: string) => {
      context.fillStyle = css;
      context.fillRect(0, 0, 1, 1);
      return [...context.getImageData(0, 0, 1, 1).data].slice(0, 3).map((value) => value / 255);
    };
    const luminance = (channels: number[]) =>
      channels
        .map((value) => (value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4))
        .reduce((sum, value, index) => sum + value * [0.2126, 0.7152, 0.0722][index], 0);
    const ratio = (a: number[], b: number[]) => {
      const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
      return (light + 0.05) / (dark + 0.05);
    };
    const bed = rgb(getComputedStyle(document.documentElement).getPropertyValue("--bed"));
    const hint = document.querySelector<HTMLElement>(".canvas-hint")!;
    return {
      title: ratio(rgb(getComputedStyle(hint.querySelector(".canvas-hint-title")!).color), bed),
      sub: ratio(rgb(getComputedStyle(hint).color), bed),
    };
  });
}

/**
 * The empty plater's hint is DOM text over the bed, which the canvas draws dark in both themes
 * (`--bed`, Bed3D::DEFAULT_MODEL_COLOR(_DARK) x0.8), so axe, which cannot see canvas pixels, misses it:
 * with the panel text colours it was illegible in light mode. The title and the sub-line keep WCAG AA
 * contrast (4.5:1) against the bed in each theme.
 */
test("the empty plater hint keeps WCAG AA contrast against the bed in both themes", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  const hint = page.locator(".canvas-hint");
  await expect(hint).toBeVisible({ timeout: 30_000 });

  const ratios: Record<string, Ratio> = {};
  for (const theme of ["dark", "light"] as const) {
    if (theme === "light") await page.getByRole("button", { name: "Light theme", exact: true }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
    ratios[theme] = await contrast(page);
    for (const [line, ratio] of Object.entries(ratios[theme])) {
      expect(ratio, `${theme} theme: ${line} contrast against the bed`).toBeGreaterThanOrEqual(4.5);
    }
    const box = (await hint.boundingBox())!;
    const margin = 120;
    await page.screenshot({
      path: `${artifacts}${theme}.png`,
      clip: {
        x: box.x - margin,
        y: box.y - margin,
        width: box.width + 2 * margin,
        height: box.height + 2 * margin,
      },
    });
  }
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify(ratios, null, 2)}\n`);
});
