import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { systemSubmenus, vendorProfiles } from "./filamentOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/filament-groups/", import.meta.url));
const COMBO = "Filament 1 preset";
const PRINTER = "My X1C 0.4 nozzle";
const USER_FILAMENT = "Bambu PLA Basic @BBL X1C - Mine";
// A filament the X1C model does not install by default (not in any BBL model's default_materials).
const ADDED = "Bambu PLA Galaxy";

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

/** The open filament list as `{group, submenus: [title, options][]}` in DOM order. */
async function comboGroups(page: Page) {
  await combo(page, COMBO).click();
  const groups = await page.getByRole("listbox", { name: COMBO, exact: true }).evaluate((list) =>
    [...list.querySelectorAll(":scope > [role=group]")].map((group) => ({
      group: group.getAttribute("aria-label")!,
      inline: [...group.querySelectorAll(":scope > [role=option]")].map((option) => option.textContent!),
      submenus: [...group.querySelectorAll(":scope > [role=group]")].map((submenu) => [
        submenu.getAttribute("aria-label")!,
        [...submenu.querySelectorAll("[role=option]")].map((option) => option.textContent!),
      ] as [string, string[]]),
    })));
  return groups;
}

test("filament combos group compatible installed presets like PlaterPresetComboBox; the filament dialog edits the installed set", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await page.getByTestId("preset-import-input").setInputFiles(`${repositoryRoot}tests/presets/My X1C 0.4 nozzle.orca_printer`);
  // The imported printer inherits the X1C 0.4 nozzle; a user printer matches filaments through its
  // parent (Preset.cpp:799-806).
  await combo(page, "Printer preset").click();
  await page.getByRole("option", { name: PRINTER, exact: true }).click();
  await expect(combo(page, "Printer preset")).toHaveText(PRINTER);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const printer = "Bambu Lab X1 Carbon 0.4 nozzle";

  // Oracle from the raw vendor files: installed (seeded) system filaments that list the printer
  // in `compatible_printers` (no BBL filament uses a condition), grouped and ordered by the
  // PresetComboBoxes.cpp rules.
  const bbl = vendorProfiles(["BBL"]);
  const compatible = [...bbl.seeded].filter((name) => bbl.filament(name).compatiblePrinters.includes(printer));
  expect([...bbl.seeded].filter((name) => bbl.filament(name).condition !== "")).toEqual([]);
  const expected = systemSubmenus(compatible, bbl.filament);
  const groups = await comboGroups(page);
  await page.screenshot({ path: `${artifacts}combo.png` });
  await page.keyboard.press("Escape");
  expect(groups.map(({ group }) => group)).toEqual(["User presets", "System presets"]);
  expect(groups[0].inline).toEqual([USER_FILAMENT]);
  expect(groups[1].inline).toEqual([]);
  expect(groups[1].submenus).toEqual(expected);
  // Installed but not compatible: another printer's preset of a default material.
  const incompatible = [...bbl.seeded].find((name) => name.endsWith("@BBL H2D"))!;
  expect(groups[1].submenus.flatMap(([, names]) => names)).not.toContain(incompatible);

  // Search: the lower-case substring match of the filament dialog.
  await combo(page, COMBO).click();
  await page.getByRole("searchbox", { name: `Search ${COMBO}` }).fill("PETG");
  const searched = await page.getByRole("listbox", { name: COMBO, exact: true })
    .getByRole("group", { name: "System presets", exact: true }).getByRole("option").allTextContents();
  await expect(page.getByRole("listbox", { name: COMBO, exact: true }).getByRole("group", { name: "User presets", exact: true })).toHaveCount(0);
  expect(searched).toEqual(expected.flatMap(([, names]) => names).filter((name) => name.toLowerCase().includes("petg")));
  await page.screenshot({ path: `${artifacts}search.png` });

  // "Add/Remove filaments": the dialog's type column starts with the FilamentPriority types and the
  // filament column lists the printer's compatible system filaments by short name.
  await page.getByRole("searchbox", { name: `Search ${COMBO}` }).fill("");
  await page.getByRole("option", { name: "Add/Remove filaments" }).click();
  const dialog = page.getByRole("dialog", { name: "Filament selection" });
  const types = dialog.getByRole("group", { name: "Filament type" });
  await expect(types.getByRole("checkbox").nth(1)).toBeVisible({ timeout: 60_000 });
  const typeNames = await types.locator("label").allTextContents();
  expect(typeNames.slice(0, 3)).toEqual(["all", "PLA", "ABS"]);
  await dialog.getByRole("searchbox", { name: "Search Vendor" }).fill("gener");
  expect(await dialog.getByRole("group", { name: "Vendor" }).locator("label").allTextContents()).toEqual(["all", "Generic"]);
  const filter = dialog.getByRole("searchbox", { name: "Filter filaments" });
  await filter.fill("::checked");
  const checkedItems = await dialog.getByRole("group", { name: "Filaments" }).locator("label").allTextContents();
  expect(checkedItems.sort()).toEqual([...new Set(compatible.map((name) => name.split("@")[0].trim()))].sort());
  await filter.fill(ADDED.toLowerCase());
  const item = dialog.getByRole("group", { name: "Filaments" }).getByRole("checkbox", { name: ADDED, exact: true });
  await expect(item).not.toBeChecked();
  await item.check();
  await page.screenshot({ path: `${artifacts}dialog.png` });
  await dialog.getByRole("button", { name: "Done" }).click();
  const added = [...bbl.filaments.keys()].filter((name) => name.split("@")[0].trim() === ADDED
    && bbl.filament(name).compatiblePrinters.includes(printer));
  expect(added.length).toBeGreaterThan(0);
  const after = await comboGroups(page);
  await page.keyboard.press("Escape");
  expect(after[1].submenus).toEqual(systemSubmenus([...compatible, ...added], bbl.filament));

  // The installed set is stored: the added filament is still listed after a reload.
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await expect(combo(page, "Printer preset")).toHaveText(PRINTER, { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const reloaded = await comboGroups(page);
  expect(reloaded[1].submenus).toEqual(after[1].submenus);
  await page.screenshot({ path: `${artifacts}after-reload.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ printer, submenus: reloaded[1].submenus, added }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
