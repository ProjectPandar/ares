import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/about/", import.meta.url));
// The release version (bumped with every batch) and the OrcaSlicer version the core tracks.
const version: string = JSON.parse(readFileSync(`${repositoryRoot}web/app/package.json`, "utf8")).version;
const orca = /ORCA_SLICER_COMPATIBILITY_VERSION: &str = "([^"]+)"/
  .exec(readFileSync(`${repositoryRoot}crates/ares-core/src/generation.rs`, "utf8"))![1];

/** MainFrame::generate_help_menu (MainFrame.cpp:2903-2966) in the top bar's drop-down (:3683). */
async function openFromHelp(page: Page, item: string) {
  await page.getByRole("button", { name: "Help" }).click();
  const menu = page.getByRole("menu", { name: "Help" });
  await expect(menu.getByRole("menuitem")).toHaveText(["Keyboard Shortcuts", "About Ares"]);
  await menu.getByRole("menuitem", { name: item }).click();
  await expect(menu).toHaveCount(0);
}

const background = (page: Page) =>
  page.getByRole("dialog", { name: "About Ares" }).evaluate((element) => getComputedStyle(element).backgroundColor);

test("Help opens the keyboard shortcuts and an About dialog with the release version", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });

  await openFromHelp(page, "Keyboard Shortcuts");
  const shortcuts = page.getByRole("dialog", { name: "Keyboard Shortcuts" });
  await expect(shortcuts.getByRole("heading", { name: "Global shortcuts" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(shortcuts).toBeHidden();

  // AboutDialog (AboutDialog.cpp:230-360): title, version, attribution, licence.
  await openFromHelp(page, "About Ares");
  const about = page.getByRole("dialog", { name: "About Ares" });
  await expect(about).toBeVisible();
  await expect(about.getByTestId("about-version")).toHaveText(version);
  await expect(about).toContainText(`Rust port of the OrcaSlicer ${orca} slicing core`);
  for (const project of ["Slic3r", "PrusaSlicer", "Bambu Studio", "OrcaSlicer"]) await expect(about).toContainText(project);
  await expect(about.getByRole("link", { name: "GNU Affero General Public License, version 3" }))
    .toHaveAttribute("href", "https://www.gnu.org/licenses/agpl-3.0.html");
  await page.screenshot({ path: `${artifacts}about-dark.png` });
  const dark = await background(page);
  await page.keyboard.press("Escape");
  await expect(about).toBeHidden();

  await page.getByRole("button", { name: "Light theme" }).click();
  await openFromHelp(page, "About Ares");
  await expect(about.getByTestId("about-version")).toHaveText(version);
  const light = await background(page);
  expect(light, "the dialog follows the theme").not.toBe(dark);
  await page.screenshot({ path: `${artifacts}about-light.png` });
  await about.getByRole("button", { name: "Close" }).click();
  await expect(about).toBeHidden();

  // Phone width: the menu stays reachable and the dialog fits the screen.
  await page.setViewportSize({ width: 390, height: 844 });
  await openFromHelp(page, "About Ares");
  const box = (await about.boundingBox())!;
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(390);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  await page.screenshot({ path: `${artifacts}about-phone.png` });
  await page.keyboard.press("Escape");
  await expect(about).toBeHidden();

  expect(errors).toEqual([]);
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ version, orca, themes: { dark, light }, phone: box }, null, 2)}\n`);
});
