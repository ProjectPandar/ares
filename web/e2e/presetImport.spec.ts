import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/preset-import/", import.meta.url));

interface Compatible {
  processes: string[];
  filaments: string[];
  diagnostics: { kind: string; name: string; message: string }[];
  missing_parents: { kind: string; name: string; parent: string }[];
}

/** Native `PresetCatalog` filtering of the same imports and OrcaSlicer vendor files (ares-wasm `compatible_presets` test). */
interface Expected {
  bundle: string;
  filament: string;
  printer: string;
  process: string;
  missingVendorError: string;
  withoutProcess: Compatible;
  withProcess: Compatible;
}

const expected = JSON.parse(
  readFileSync(`${repositoryRoot}target/web-e2e/expected-compatible.json`, "utf8"),
) as Expected;

/** Names of a combo's group, in list order; Escape closes the list. */
async function groupOptions(page: Page, combo: string, group: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  const options = page.getByRole("listbox", { name: combo, exact: true }).getByRole("group", { name: group }).getByRole("option");
  const names = await options.allTextContents();
  await page.keyboard.press("Escape");
  return names;
}

async function choose(page: Page, combo: string, name: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  await page.getByRole("listbox", { name: combo, exact: true }).getByRole("option", { name, exact: true }).click();
  await expect(page.getByRole("button", { name: combo, exact: true })).toHaveText(name);
}

const timings = (page: Page) =>
  page.evaluate(() =>
    performance.getEntriesByType("measure")
      .filter((entry) => entry.name.startsWith("ares:"))
      .map(({ name, duration }) => ({ name, duration })),
  );

test("imported printer presets list the compatible processes and filaments", async ({ page }) => {
  test.setTimeout(240_000);
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/orca_cli_ender3.3mf`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  const projectPrinter = await page.getByRole("button", { name: "Printer preset", exact: true }).textContent();
  const sliceButton = page.getByRole("button", { name: "Slice plate" });
  await expect(sliceButton).toBeEnabled();

  // MainFrame::load_config_file: File > Import > Import Configs.
  await page.getByTestId("preset-import-input").setInputFiles(
    [expected.bundle, expected.filament].map((name) => `${repositoryRoot}tests/presets/${name}`),
  );
  expect(await groupOptions(page, "Printer preset", "User presets")).toEqual([expected.printer]);
  expect(await groupOptions(page, "Printer preset", "Project presets")).toEqual([projectPrinter]);

  // No system vendor installed: the engine's missing-parent error, verbatim.
  await choose(page, "Printer preset", expected.printer);
  await expect(page.getByTestId("preset-error")).toHaveText(expected.missingVendorError);
  await expect(sliceButton).toBeDisabled();
  await page.screenshot({ path: `${artifacts}missing-vendor.png` });

  // ConfigWizard: install the BBL vendor, as upstream requires before importing user presets.
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "System presets" });
  await wizard.getByRole("checkbox", { name: "Bambulab", exact: true }).check();
  await page.screenshot({ path: `${artifacts}vendors.png` });
  await wizard.getByRole("button", { name: "Done" }).click();
  await expect(page.getByTestId("preset-error")).toHaveCount(0);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });

  const processes = [
    ...await groupOptions(page, "Process preset", "User presets"),
    ...await groupOptions(page, "Process preset", "System presets"),
  ];
  expect(processes.sort()).toEqual([...expected.withoutProcess.processes].sort());
  await expect(page.getByTestId("preset-diagnostics")).toHaveCount(expected.withoutProcess.diagnostics.length ? 1 : 0);
  // Preset.cpp:1685-1691: presets whose parent is not loaded are skipped; the engine lists them.
  const missingParents = page.getByTestId("preset-missing-parents");
  await missingParents.locator("summary").click();
  await expect(missingParents.getByRole("listitem")).toHaveText(
    expected.withoutProcess.missing_parents.map(({ kind, name, parent }) => `${name} (${kind}): parent preset "${parent}" is not loaded`),
  );
  await page.screenshot({ path: `${artifacts}missing-parents.png` });
  await expect(sliceButton).toBeDisabled();
  await expect(sliceButton).toHaveAttribute("title", "Select a process preset to slice");
  await expect(page.getByTestId("preset-slice-blocker")).toHaveText("Select a process preset to slice.");
  await page.getByRole("button", { name: "Process preset", exact: true }).click();
  await page.screenshot({ path: `${artifacts}processes.png` });
  await page.keyboard.press("Escape");

  await choose(page, "Process preset", expected.process);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const filaments = [
    ...await groupOptions(page, "Filament 1 preset", "User presets"),
    ...await groupOptions(page, "Filament 1 preset", "System presets"),
  ];
  expect(filaments.sort()).toEqual([...expected.withProcess.filaments].sort());
  await expect(sliceButton).toHaveAttribute("title", "Select a filament preset for every filament to slice");
  await choose(page, "Filament 1 preset", expected.withProcess.filaments[0]);
  // A 3MF with every preset selected slices through sliceWithProfiles.
  await expect(sliceButton).toBeEnabled();
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  await page.getByRole("button", { name: "Filament 1 preset", exact: true }).click();
  await page.screenshot({ path: `${artifacts}filaments.png` });
  await page.keyboard.press("Escape");

  // Back to the project preset: slicing is enabled again.
  await choose(page, "Printer preset", projectPrinter!);
  await expect(sliceButton).toBeEnabled();

  const measures = await timings(page);
  const build = measures.filter((entry) => entry.name === "ares:catalog-build");
  const filter = measures.filter((entry) => entry.name === "ares:filter-compatible-presets");
  // One catalog handle per vendor change (without and with BBL); filtered without BBL, then without and with a process.
  // Filtering never serializes the whole catalog: that is only for slicing.
  expect(build).toHaveLength(2);
  expect(filter).toHaveLength(3);
  expect(measures.filter((entry) => entry.name === "ares:catalog-stringify")).toHaveLength(0);
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    vendors: ["BBL", "OrcaFilamentLibrary"],
    printer: expected.printer,
    processes: processes.length,
    filaments: filaments.length,
    missingParents: expected.withoutProcess.missing_parents,
    // Without BBL, with BBL; and the filters without BBL, with BBL, with BBL and a process.
    catalogBuildMs: build.map((entry) => entry.duration),
    filterMs: filter.map((entry) => entry.duration),
  }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
