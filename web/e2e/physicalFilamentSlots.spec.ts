import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const fixture = (path: string) => fileURLToPath(new URL(`../../tests/parity/${path}`, import.meta.url));

async function openProject(page: Page, vendor: string, file: string) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: `All ${vendor} models`, exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(file);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
}

test("adding and merging physical filaments updates Prepare assignments and can be undone", async ({ page }) => {
  await openProject(page, "Bambu Lab", fixture("modifier-feature-filament/control.3mf"));
  const rows = page.getByTestId("object-list");
  const obj = rows.getByRole("treeitem").filter({ hasText: "cube10.stl" }).first();
  await expect(obj.getByRole("img", { name: "Filament 1", exact: true })).toBeVisible();

  const add = page.getByRole("button", { name: "Add one filament", exact: true });
  const removeLast = page.getByRole("button", { name: "Remove last filament", exact: true });
  const third = page.getByTestId("filament-swatch-3");
  await add.click({ timeout: 5_000 });
  await expect(third).toBeVisible();
  await removeLast.click();
  await expect(third).toHaveCount(0);
  await page.keyboard.press("Control+z");
  await expect(third).toBeVisible();

  const actions = page.getByRole("button", { name: "Filament 1 actions", exact: true });
  await actions.click();
  await page.getByRole("menuitem", { name: "Edit", exact: true }).click();
  const settings = page.getByRole("dialog", { name: "Filament settings", exact: true });
  await expect(settings).toBeVisible();
  await settings.getByRole("button", { name: "Close", exact: true }).click();

  await actions.click();
  await page.getByRole("menuitem", { name: "Merge with", exact: true }).click();
  await page.getByRole("menuitem", { name: /^Filament 3:/ }).click();
  await expect(third).toHaveCount(0);
  await expect(obj.getByRole("img", { name: "Filament 2", exact: true })).toBeVisible();
  await page.keyboard.press("Control+z");
  await expect(third).toBeVisible();
  await expect(obj.getByRole("img", { name: "Filament 1", exact: true })).toBeVisible();
  await page.keyboard.press("Control+Shift+z");
  await expect(obj.getByRole("img", { name: "Filament 2", exact: true })).toBeVisible();

  await actions.click();
  await page.getByRole("menuitem", { name: "Delete", exact: true }).click();
  await expect(page.getByTestId("filament-swatch-2")).toHaveCount(0);
  await expect(removeLast).toHaveCount(0);
});

test("fixed-head slots can open the editor without showing count controls", async ({ page }) => {
  await openProject(page, "Snapmaker", fixture("type2-tower-entry-z/j1-two-tools.3mf"));
  await expect(page.getByRole("button", { name: "Add one filament", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Remove last filament", exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "Edit filament 2 settings", exact: true }).click();
  await expect(page.getByRole("dialog", { name: "Filament settings", exact: true })).toBeVisible();
});
