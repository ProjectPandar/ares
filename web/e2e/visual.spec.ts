import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const label = process.env.ARES_VISUAL_LABEL ?? "current";
const artifacts = fileURLToPath(new URL(`../../target/web-e2e/visual/${label}/`, import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;

async function setTheme(page: Page, theme: "dark" | "light") {
  const current = await page.evaluate(() => document.documentElement.dataset.theme);
  if (current !== theme) await page.getByTitle(theme === "dark" ? "Dark theme" : "Light theme").click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

/** Prepare/Preview screenshots in both themes, for before/after fidelity review. */
test("visual snapshots", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await setTheme(page, "dark");
  await page.screenshot({ path: `${artifacts}empty-dark.png` });

  await page.getByTestId("import-input").setInputFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("layer-count")).toHaveText("50");

  for (const theme of ["dark", "light"] as const) {
    await setTheme(page, theme);
    await page.getByRole("link", { name: "Prepare" }).click();
    await page.screenshot({ path: `${artifacts}prepare-${theme}.png` });
    await page.getByRole("link", { name: "Preview" }).click();
    await page.screenshot({ path: `${artifacts}preview-${theme}.png` });
  }
});
