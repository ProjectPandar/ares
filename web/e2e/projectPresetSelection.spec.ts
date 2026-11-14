import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { nativeSelection, presetSelectProbe, PROJECT_CONFIG } from "./presetSelectOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/project-preset-selection/", import.meta.url));
// X1C, one filament; process "0.20mm Standard @BBL X1C - Mine" inherits the system
// "0.20mm Standard @BBL X1C", and every different_settings_to_system entry is empty.
const FIXTURE = `${repositoryRoot}tests/parity/bbl-eager-enforce/all-surfaces.3mf`;
// Snapmaker A250, one filament; every inherits_group entry is empty.
const SNAPMAKER = `${repositoryRoot}tests/parity/arachne/default-prisms.3mf`;
// An unprotected process value unequal to the system preset's 2 (fdm_process_common.json).
const EDIT = { wall_loops: "3" };

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

async function sliceAndExport(page: Page, project: string, gcode: string) {
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code" }).click()]);
  await download.saveAs(gcode);
  return readFileSync(gcode, "utf8");
}

/** The value of `; key = value` in the G-code config block. */
const configValue = (gcode: string, key: string) => gcode.match(new RegExp(`^; ${key} = (.*)$`, "m"))?.[1];

test.beforeAll(() => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
});

test("a project slices with the presets the plater selects when it opens the project (LoadAndSelect::Always)", async ({ page }) => {
  const probe = presetSelectProbe();
  test.setTimeout(300_000);
  const entries = unzipSync(readFileSync(FIXTURE));
  entries[PROJECT_CONFIG] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[PROJECT_CONFIG])), ...EDIT }));
  writeFileSync(`${artifacts}edited.3mf`, zipSync(entries));

  const native = probe ? nativeSelection(probe, `${artifacts}native/`, `${artifacts}edited.3mf`, ["BBL"]).process : null;
  if (native) {
    // Native premise: no "- Mine" preset is loaded, so the parent is selected, and Always refreshes
    // the unprotected wall_loops from it although every difference entry is empty.
    expect(native.selected.name, "Native selects the parent system process").toBe("0.20mm Standard @BBL X1C");
    expect(native.edited.config.wall_loops.serialized, "Native refreshes wall_loops from the parent").not.toBe(EDIT.wall_loops);
  }

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
  const gcode = await sliceAndExport(page, `${artifacts}edited.3mf`, `${artifacts}web.gcode`);
  if (native) {
    expect(configValue(gcode, "wall_loops"), "browser wall_loops").toBe(native.edited.config.wall_loops.serialized);
  }
  expect(errors).toEqual([]);
});

test("a project of a printer vendor that is not installed slices with the project-local presets the plater selects", async ({ page }) => {
  const probe = presetSelectProbe();
  test.setTimeout(300_000);
  // The first-run config installs no Snapmaker printer.
  const native = probe ? nativeSelection(probe, `${artifacts}not-installed/`, SNAPMAKER, []) : null;
  if (native) {
    // Native premise: no preset of the stored names is loaded, so load_external_preset loads and
    // selects project-local "<name>(<file>)" presets (Preset.cpp:3173-3230).
    expect(native.printer.selected.name, "Native selects a project-local printer").toBe("Snapmaker A250 (0.4 nozzle)(default-prisms.3mf)");
  }

  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  // The plater has no compatible-printers check of the CLI main (OrcaSlicer.cpp:2575-2620, 2653-2654):
  // the slice runs with the selected presets, whose names full_fff_config writes (PresetBundle.cpp:5818-5820).
  const gcode = await sliceAndExport(page, SNAPMAKER, `${artifacts}not-installed.gcode`);
  if (native) {
    expect(configValue(gcode, "printer_settings_id"), "browser printer").toBe(native.printer.selected.name);
    expect(configValue(gcode, "print_settings_id"), "browser process").toBe(native.process.selected.name);
    const filaments = configValue(gcode, "filament_settings_id")?.split(";").map((name) => name.replace(/^"|"$/g, ""));
    expect(filaments, "browser filaments").toEqual(native.filament_presets);
  }
  expect(errors).toEqual([]);
});
