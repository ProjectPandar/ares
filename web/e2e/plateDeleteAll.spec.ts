import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plate-delete-all/", import.meta.url));
/** An Ender-3 project with a cube on each of two plates. */
const FIXTURE = "tests/parity/custom-gcode-overview/two-plates.3mf";

test("Ctrl+D deletes every object on the Prepare tab and keeps the plates", async ({ page }) => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${FIXTURE}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  const tree = page.getByTestId("object-list");
  // Plate 1, its cube, plate 2, its cube.
  await expect(tree.getByRole("treeitem")).toHaveCount(4);

  // MainFrame::can_delete_all (MainFrame.cpp:2128-2131): only on the Prepare tab.
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  await expect(page).toHaveURL(/\/preview$/);
  await page.keyboard.press("ControlOrMeta+d");
  await page.getByRole("link", { name: "Prepare", exact: true }).click();
  await expect(page).toHaveURL(/\/prepare$/);
  await expect(tree.getByRole("treeitem")).toHaveCount(4);

  await page.keyboard.press("ControlOrMeta+d");
  await expect(tree.getByRole("treeitem")).toHaveCount(2);
  await expect(tree.getByRole("treeitem", { name: /^Plate 1\b/ })).toBeVisible();
  await expect(tree.getByRole("treeitem", { name: /^Plate 2\b/ })).toBeVisible();
  await page.screenshot({ path: `${artifacts}deleted-all.png` });

  await page.keyboard.press("?");
  const list = page.getByRole("dialog", { name: "Keyboard Shortcuts" });
  for (const description of ["Delete Selected", "Delete All", "Add instance", "Remove instance"])
    await expect(list.getByText(description, { exact: true })).toBeVisible();
  await page.screenshot({ path: `${artifacts}shortcuts.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    fixture: FIXTURE,
    edits: ["Ctrl+D on Preview (ignored)", "Ctrl+D on Prepare"],
    tree: await tree.getByRole("treeitem").allTextContents(),
  }, null, 2)}\n`);
});
