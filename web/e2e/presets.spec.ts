import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { nativeSelection, presetSelectProbe } from "./presetSelectOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/presets/", import.meta.url));

interface Expected {
  printer: string;
  process: string;
  filaments: string[];
  colours: string[];
}

/**
 * The presets the plater selects when it opens `fixture` (PresetBundle::load_config_model, Plater.cpp:9622) with
 * the first-run config's installed vendors, and the project's filament colours.
 */
function nativePresets(probe: string, fixture: string, name: string): Expected {
  const native = nativeSelection(probe, `${artifacts}${name}/`, `${repositoryRoot}${fixture}`, []);
  return {
    printer: native.printer.selected.name,
    process: native.process.selected.name,
    filaments: native.filament_presets,
    colours: native.project_config.filament_colour.serialized.split(";").map((item) => item.replace(/^"|"$/g, "")),
  };
}

const rgb = (hex: string) =>
  `rgb(${[1, 3, 5].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16)).join(", ")})`;

/** Opens a PlaterPresetComboBox and checks its Project presets group; Escape closes it. */
async function checkCombo(page: Page, fixture: string, combo: string, selected: string, entries: string[]) {
  const button = page.getByRole("button", { name: combo, exact: true });
  await expect(button).toHaveText(selected);
  await expect(button).toHaveAttribute("aria-expanded", "false");
  await button.click();
  await expect(button).toHaveAttribute("aria-expanded", "true");
  const list = page.getByRole("listbox", { name: combo, exact: true });
  const group = list.getByRole("group", { name: "Project presets" });
  await expect(group.getByRole("option")).toHaveText(entries);
  await expect(group.getByRole("option", { selected: true })).toHaveText(selected);
  await page.screenshot({ path: `${artifacts}${fixture}-${combo.replace(/\W+/g, "-").toLowerCase()}.png` });
  await page.keyboard.press("Escape");
  await expect(list).toHaveCount(0);
  await expect(button).toHaveAttribute("aria-expanded", "false");
}

test("preset combo boxes show the project's embedded presets", async ({ page }) => {
  const probe = presetSelectProbe();
  if (!probe) test.skip();
  test.setTimeout(240_000);
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });
  const fixtures = [
    { name: "default-prisms", path: "tests/parity/arachne/default-prisms.3mf" },
    // Two filament slots sharing one preset: the list holds each preset name once.
    { name: "ksr-fdmtest-v4", path: "tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf" },
  ];
  const results: Record<string, Expected> = {};

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  for (const fixture of fixtures) {
    const expected = nativePresets(probe!, fixture.path, fixture.name);
    results[fixture.name] = expected;
    await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${fixture.path}`);
    await expect(page.getByTestId("status")).toHaveText("Ready to slice");

    const unique = [...new Set(expected.filaments)];
    await checkCombo(page, fixture.name, "Printer preset", expected.printer, [expected.printer]);
    await checkCombo(page, fixture.name, "Process preset", expected.process, [expected.process]);
    for (const [index, filament] of expected.filaments.entries()) {
      await checkCombo(page, fixture.name, `Filament ${index + 1} preset`, filament, unique);
      await expect(page.getByTestId(`filament-swatch-${index + 1}`)).toHaveCSS("background-color", rgb(expected.colours[index]));
    }
    await expect(page.getByRole("button", { name: `Filament ${expected.filaments.length + 1} preset` })).toHaveCount(0);
  }

  // Outside clicks close an open list as well.
  await page.getByRole("button", { name: "Printer preset" }).click();
  await page.getByTestId("viewport").click({ position: { x: 20, y: 400 } });
  await expect(page.getByRole("listbox")).toHaveCount(0);

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify(results, null, 2)}\n`);
  expect(errors).toEqual([]);
});
