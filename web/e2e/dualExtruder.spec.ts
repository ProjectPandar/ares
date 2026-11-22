import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";
import { flowOptions } from "./flowOptionsOracle";
import { normalize } from "./orcaGcode";
import { platerExport, withoutCliGroups } from "./platerExport";
import { nativeSelection, presetSelectProbe } from "./presetSelectOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/dual-extruder/", import.meta.url));
// H2D, Generic TPU + Bambu PLA Basic, both nozzles Standard 0.4. The fixture's master extruder 1
// rejects TPU on the left nozzle; its provenance names master 2 as the legal variant.
const FIXTURE = `${repositoryRoot}tests/parity/master-extruder-id/stock-right-only.3mf`;
const LEGAL = { master_extruder_id: "2" };
// The sidebar edits the test makes, as project config values.
const EDITS = { filament_map_mode: "Manual", filament_map: ["2", "1"], nozzle_volume_type: ["Standard", "High Flow"] };

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const select = (page: Page, name: string) => page.getByRole("combobox", { name, exact: true });

/** A copy of the fixture whose project config holds `edit` (slice.spec.ts `orcaEditedGcode`). */
function project(name: string, edit: Record<string, unknown>) {
  const entries = unzipSync(readFileSync(FIXTURE));
  const config = "Metadata/project_settings.config";
  entries[config] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[config])), ...edit }));
  writeFileSync(`${artifacts}${name}`, zipSync(entries));
  return `${artifacts}${name}`;
}

function orcaGcode(file: string) {
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-dual-`)}/`;
  execFileSync(`${repositoryRoot}scripts/orca-parity.sh`, ["--slice", "0", "--outputdir", work, file], { cwd: work, stdio: ["ignore", "pipe", "pipe"] });
  const gcode = readFileSync(`${work}plate_1.gcode`, "utf8");
  writeFileSync(`${artifacts}native.gcode`, gcode);
  return normalize(gcode, "OrcaSlicer");
}

/** The flow combos list the printer's upstream options; extruder 0 is the left nozzle (Plater.cpp:4111-4112). */
async function expectFlowOptions(page: Page, printer: string) {
  for (const [index, nozzle] of ["Left Nozzle", "Right Nozzle"].entries())
    expect(await select(page, `${nozzle} flow`).locator("option").allTextContents(), `${printer} ${nozzle} flow options`)
      .toEqual(flowOptions("BBL", printer, index));
}

const nozzleFilaments = (page: Page, nozzle: string) =>
  page.getByRole("group", { name: nozzle, exact: true }).getByRole("button").evaluateAll((buttons) => buttons.map((button) => button.textContent!.trim()));

test("an H2D project shows the left and right nozzles and slices its filament map and flow like OrcaSlicer", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the project-edit oracle", "an extraction of the official OrcaSlicer 2.5.0-beta AppImage, see scripts/orca-parity.sh");
  const probe = presetSelectProbe();
  if (orca && !probe) throw new Error("ARES_ORCA_PRESET_SELECT_BIN is unset while ORCA_APPDIR is set: the comparison needs the plater export");
  test.setTimeout(400_000);
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
  const legal = project("legal.3mf", LEGAL);
  await page.getByTestId("import-input").setInputFiles(legal);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  // The Left / Right Nozzle groups replace the nozzle combo (Plater.cpp:641-652, 2645-2648).
  await expect(page.getByRole("group", { name: "Left Nozzle", exact: true })).toBeVisible({ timeout: 60_000 });
  await expect(select(page, "Nozzle diameter")).toHaveCount(0);
  for (const nozzle of ["Left Nozzle", "Right Nozzle"]) {
    await expect(select(page, `${nozzle} diameter`)).toHaveValue("0.4");
    await expect(select(page, `${nozzle} flow`)).toHaveValue("Standard");
  }
  await expectFlowOptions(page, "Bambu Lab H2D 0.4 nozzle");
  const grouping = page.getByRole("radiogroup", { name: "Filament grouping" });
  await expect(grouping.getByRole("radio", { name: /Filament-Saving Mode/ })).toBeChecked();
  await expect(grouping.getByRole("radio", { name: /Convenience Mode/ })).toBeDisabled();
  await page.screenshot({ path: `${artifacts}project.png` });

  // Custom Mode: both filaments start on the left (filament_map 1,1); TPU moves right.
  await grouping.getByRole("radio", { name: /Custom Mode/ }).check();
  expect(await nozzleFilaments(page, "Left Nozzle")).toEqual(["1TPU", "2PLA"]);
  expect(await nozzleFilaments(page, "Right Nozzle")).toEqual([]);
  await page.getByRole("button", { name: "Move filament 1 to the right nozzle" }).click();
  expect(await nozzleFilaments(page, "Left Nozzle")).toEqual(["2PLA"]);
  expect(await nozzleFilaments(page, "Right Nozzle")).toEqual(["1TPU"]);
  await select(page, "Right Nozzle flow").selectOption("High Flow");
  await page.screenshot({ path: `${artifacts}manual-map.png` });

  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code" }).click()]);
  await download.saveAs(`${artifacts}web.gcode`);
  const web = readFileSync(`${artifacts}web.gcode`, "utf8");
  expect(web).toContain("; filament_map = 2,1");
  expect(web).toContain("; nozzle_volume_type = Standard,High Flow");
  // A key the project lacks comes from its system printer preset (OrcaSlicer.cpp:3167-3200,
  // PresetBundle::load_config_file_config): the H2D's 50, not the option default 40.
  expect(web).toContain("; extruder_clearance_dist_to_rod = 50\n");
  // The plater opens the 3MF through load_config_model (Plater.cpp:9622): each preset loads with LoadAndSelect::Always
  // (PresetBundle.cpp:6300-6354), and load_external_config (Preset.cpp:1090-1119) replaces every key outside
  // different_settings_to_system with the current system preset value, since project_different_keys always lists the
  // *_settings_id keys (PresetBundle.cpp:5525-5532). The fixture lists no changed key, so the plater slices the
  // system H2D values, while a CLI slice of the edited 3MF keeps the stored ones (OrcaSlicer.cpp:2775-2791). The
  // reference is therefore the 3MF the plater saves after the edits, sliced by OrcaSlicer.
  if (orca) {
    const selection = nativeSelection(probe!, `${artifacts}selection/`, legal, ["BBL"]);
    const native = withoutCliGroups(orcaGcode(platerExport(selection, legal, `${artifacts}edited.3mf`, EDITS)), selection);
    expect(normalize(web, "Ares"), "the sidebar edits slice as the plater export").toBe(native);
  }

  // Different diameters cannot print together: the question picks one for both nozzles; Cancel keeps both.
  await select(page, "Left Nozzle diameter").selectOption("0.6");
  const question = page.getByRole("dialog", { name: "Switch diameter" });
  await expect(question.getByRole("button", { name: "Left nozzle: 0.6mm" })).toBeVisible();
  await expect(question.getByRole("button", { name: "Right nozzle: 0.4mm" })).toBeVisible();
  await page.screenshot({ path: `${artifacts}switch-diameter.png` });
  await question.getByRole("button", { name: "Cancel" }).click();
  await expect(select(page, "Left Nozzle diameter")).toHaveValue("0.4");
  await select(page, "Left Nozzle diameter").selectOption("0.6");
  await question.getByRole("button", { name: "Left nozzle: 0.6mm" }).click();
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "Bambu Lab H2D 0.6 nozzle", { timeout: 60_000 });
  await expect(select(page, "Right Nozzle diameter")).toHaveValue("0.6");
  await expectFlowOptions(page, "Bambu Lab H2D 0.6 nozzle");

  // The selected printer keeps its volume types (AppConfig nozzle_volume_types) and the preferred
  // filament map mode across a reload.
  await select(page, "Right Nozzle flow").selectOption("High Flow");
  await page.reload();
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", "Bambu Lab H2D 0.6 nozzle", { timeout: 60_000 });
  await expect(select(page, "Right Nozzle flow")).toHaveValue("High Flow");
  await expect(select(page, "Left Nozzle flow")).toHaveValue("Standard");
  await expect(page.getByRole("radiogroup", { name: "Filament grouping" }).getByRole("radio", { name: /Custom Mode/ })).toBeChecked();
  await page.screenshot({ path: `${artifacts}after-reload.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ edits: EDITS, legal: LEGAL, oracle: orca ? "OrcaSlicer 2.5.0-beta" : null }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
