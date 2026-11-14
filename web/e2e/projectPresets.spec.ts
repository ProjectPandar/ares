import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { FIRST_RUN_CONFIG, test } from "./failureDiagnostics";
import { nativeSelection, presetSelectProbe } from "./presetSelectOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/project-presets/", import.meta.url));
const PROJECT = `${repositoryRoot}tests/parity/arachne/default-prisms.3mf`;
const MODEL = `${repositoryRoot}tests/parity/arachne/hole.stl`;

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const storedPrinter = (page: Page) =>
  page.evaluate(() => (JSON.parse(localStorage.getItem("ares.config")!) as { printer: string | null }).printer);

test("opening a project selects its presets, and a model file keeps the selected printer", async ({ page }) => {
  const probe = presetSelectProbe();
  if (!probe) test.skip();
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  // The first-run config installs no Snapmaker printer.
  const native = nativeSelection(probe!, `${artifacts}native/`, PROJECT, []);
  const printer = native.printer.selected.name;
  const processPreset = native.process.selected.name;
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", FIRST_RUN_CONFIG.printer, { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const status = page.getByTestId("status");

  // An STL carries no config (Plater.cpp:9514-9515): the system printer stays selected.
  await page.getByTestId("import-input").setInputFiles(MODEL);
  await expect(status).toHaveText("Ready to slice");
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", FIRST_RUN_CONFIG.printer);
  expect(await storedPrinter(page)).toBe(FIRST_RUN_CONFIG.printer);

  // A project's config becomes the current presets (PresetBundle::load_config_model,
  // Plater.cpp:9622): every combo shows the preset it selects as its Project presets entry.
  await page.getByTestId("import-input").setInputFiles(PROJECT);
  await expect(status).toHaveText("Ready to slice");
  await expect(combo(page, "Printer preset")).toHaveText(printer);
  await expect(combo(page, "Process preset")).toHaveText(processPreset);
  await combo(page, "Printer preset").click();
  await expect(page.getByRole("listbox", { name: "Printer preset", exact: true }).getByRole("group", { name: "Project presets" })
    .getByRole("option", { selected: true })).toHaveText(printer);
  await page.keyboard.press("Escape");
  expect(await storedPrinter(page)).toBeNull();
  await page.screenshot({ path: `${artifacts}project-presets.png` });

  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(status).toContainText("Slice complete", { timeout: 240_000 });
  await page.screenshot({ path: `${artifacts}sliced.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ printer, process: processPreset }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
