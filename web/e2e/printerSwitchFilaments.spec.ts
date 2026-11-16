import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, unzipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";
import { cliSwitchedExport, withoutCliGroups } from "./platerExport";
import { type NativeSelection, nativeSelection, PROJECT_CONFIG, presetSelectProbe } from "./presetSelectOracle";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;
const artifactRoot = fileURLToPath(new URL("../../target/web-e2e/printer-switch-filaments/", import.meta.url));
const ONE_FILAMENT = `${repositoryRoot}tests/parity/arachne/default-prisms.3mf`;
const THREE_FILAMENTS = `${repositoryRoot}tests/parity/filament-group-flush-context/x2d_pla_three_objects_default_purge.3mf`;
// The variant combo is "Variant" for a printer of several extruders, else "Nozzle diameter".
const U1 = { model: "Snapmaker U1", combo: "Variant", variant: "0.4", printer: "Snapmaker U1 (0.4 nozzle)" };
const A350 = { model: "Snapmaker A350", combo: "Nozzle diameter", variant: "0.4", printer: "Snapmaker A350 (0.4 nozzle)" };

/** The GuideFrame "All Snapmaker models" check: every model with its `nozzle_diameter` variants (PresetBundle.cpp:8319-8327). */
function snapmakerModels() {
  const vendor = JSON.parse(readFileSync(`${profiles}Snapmaker.json`, "utf8")) as { machine_model_list: { name: string; sub_path: string }[] };
  return Object.fromEntries(vendor.machine_model_list.map(({ name, sub_path }) =>
    [name, (JSON.parse(readFileSync(`${profiles}Snapmaker/${sub_path}`, "utf8")) as { nozzle_diameter: string }).nozzle_diameter.split(";")]));
}

function freshArtifacts(name: string) {
  const artifacts = `${artifactRoot}${name}/`;
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  return artifacts;
}

function embeddedFilaments(project: string) {
  return (JSON.parse(strFromU8(unzipSync(readFileSync(project))[PROJECT_CONFIG])) as { filament_settings_id: string[] }).filament_settings_id;
}

/**
 * The plater session: the 3MF opens, the GuideFrame installs every Snapmaker model and the sidebar picks `printer`.
 * No AppConfig is loaded, so the picked printer has no remembered selection unless the wizard left it selected.
 */
function platerSession(probe: string, artifacts: string, project: string, printer: string) {
  return nativeSelection(probe, `${artifacts}plater-session/`, project, [], {
    wizard: { Snapmaker: snapmakerModels() },
    select: { printer },
  });
}

/** The same session in the browser: open the 3MF, install every Snapmaker model, pick the model and its variant. */
async function selectInBrowser(page: Page, project: string, { model, combo: variants, variant, printer }: typeof U1) {
  const combo = page.getByRole("button", { name: "Printer preset", exact: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await combo.click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Snapmaker models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await combo.click();
  await page.getByRole("option", { name: model, exact: true }).click();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await page.getByRole("combobox", { name: variants, exact: true }).selectOption(variant);
  await expect(combo).toHaveAttribute("title", printer, { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

/** The sidebar shows the session's process, its filament presets and their `filament_colour` swatches. */
async function expectSidebar(page: Page, session: NativeSelection) {
  const config = JSON.parse(session.full_fff_config) as { print_settings_id: string; filament_colour: string[] };
  await expect(page.getByRole("button", { name: "Process preset", exact: true }), "the process").toHaveText(config.print_settings_id);
  await expect(page.getByRole("button", { name: /^Filament \d+ preset$/ }), "the filament presets").toHaveText(session.filament_presets);
  await expect(page.getByTestId(/^filament-swatch-\d+$/), "one swatch per filament").toHaveCount(config.filament_colour.length);
  for (const [index, colour] of config.filament_colour.entries()) {
    const [r, g, b] = [1, 3, 5].map((at) => Number.parseInt(colour.slice(at, at + 2), 16));
    await expect(page.getByTestId(`filament-swatch-${index + 1}`), `filament ${index + 1} colour`).toHaveCSS("background-color", `rgb(${r}, ${g}, ${b})`);
  }
}

function orcaGcode(artifacts: string, file: string) {
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-printer-switch-`)}/`;
  execFileSync(`${repositoryRoot}scripts/orca-parity.sh`, ["--slice", "0", "--outputdir", work, file], { cwd: work, stdio: ["ignore", "pipe", "pipe"] });
  const gcode = readFileSync(`${work}plate_1.gcode`, "utf8");
  writeFileSync(`${artifacts}native.gcode`, gcode);
  return normalize(gcode, "OrcaSlicer");
}

test("selecting the never-used four-nozzle Snapmaker U1 for a one-filament 3MF shows the plater session's process, filaments and colours", async ({ page }) => {
  const probe = presetSelectProbe();
  if (!probe) test.skip();
  test.setTimeout(300_000);
  const artifacts = freshArtifacts("u1-sidebar");
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  expect(embeddedFilaments(ONE_FILAMENT), "the project embeds one filament").toHaveLength(1);
  // Tab::select_preset runs update_selections for the printer (Tab.cpp:7740-7742): with nothing remembered the print
  // and slot 1 are the printer's preferred compatible presets and the colours "#26A69A" (PresetBundle.cpp:3914-4003),
  // then min_filament_count pads the slots with the last one (8611-8617).
  const session = platerSession(probe!, artifacts, ONE_FILAMENT, U1.printer);
  expect(session.filament_presets, "the U1 selects four filaments").toHaveLength(4);
  await selectInBrowser(page, ONE_FILAMENT, U1);
  await page.screenshot({ path: `${artifacts}prepare.png` });
  await expectSidebar(page, session);
  expect(errors).toEqual([]);
});

test("a one-filament 3MF sliced with the four-filament Snapmaker U1 selected matches OrcaSlicer slicing the plater's saved project on the U1 bed", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the plater export slicing oracle", "OrcaSlicer 2.5.0");
  const probe = presetSelectProbe();
  if (!orca || !probe) test.skip();
  test.setTimeout(600_000);
  const artifacts = freshArtifacts("u1-slice");
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  // The plater's project config is full_fff_config, which takes one filament per selected filament preset
  // (PresetBundle.cpp:5545, 5651-5660), so the selection, not the project, decides the filament count.
  const session = platerSession(probe!, artifacts, ONE_FILAMENT, U1.printer);
  // The plater saves full_config_secure() as the project config (Plater.cpp:19779); OrcaSlicer slices that 3MF as the
  // CLI machine switch the web slice follows leaves it (cliSwitchedExport). The Native CLI itself crashes on this
  // route: a one-filament BBL 3MF loaded with four U1 filaments.
  const exported = cliSwitchedExport(session, ONE_FILAMENT, `${artifacts}plater-export.3mf`);
  const native = withoutCliGroups(orcaGcode(artifacts, `${artifacts}plater-export.3mf`), exported);

  await selectInBrowser(page, ONE_FILAMENT, U1);
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toHaveText(/^(Slice complete|Slicing failed)/, { timeout: 240_000 });
  await expect(page.getByTestId("status")).toContainText("Slice complete");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export G-code" }).click();
  const path = `${artifacts}browser.gcode`;
  await (await download).saveAs(path);
  const browser = normalize(readFileSync(path, "utf8"), "Ares");
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    oracle: "OrcaSlicer 2.5.0 slicing the plater's saved project after the preset-select probe's U1 session, as the CLI machine switch leaves it",
    printer: U1.printer,
    filaments: session.filament_presets,
    bytes: browser.length,
  }, null, 2)}\n`);
  expect(browser === native, "browser G-code equals the native slice of the plater's saved project").toBe(true);
  expect(errors).toEqual([]);
});

test("a three-filament 3MF switched to the never-used single-nozzle Snapmaker A350 takes the plater session's one filament and colour", async ({ page }) => {
  const probe = presetSelectProbe();
  if (!probe) test.skip();
  test.setTimeout(300_000);
  const artifacts = freshArtifacts("a350-sidebar");
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  expect(embeddedFilaments(THREE_FILAMENTS), "the project embeds three filaments").toHaveLength(3);
  // update_selections sizes the slots from the remembered selection, not the project (PresetBundle.cpp:3929-3937):
  // the never-used A350 keeps one slot, min_filament_count is 1, and its colour is "#26A69A" (3941-3947).
  const session = platerSession(probe!, artifacts, THREE_FILAMENTS, A350.printer);
  expect(session.filament_presets, "the A350 selects one filament").toHaveLength(1);
  await selectInBrowser(page, THREE_FILAMENTS, A350);
  await page.screenshot({ path: `${artifacts}prepare.png` });
  await expectSidebar(page, session);
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    oracle: "the preset-select probe's plater session: wizard with every Snapmaker model, then the A350 selected",
    printer: A350.printer,
    filaments: session.filament_presets,
    filament_colour: (JSON.parse(session.full_fff_config) as { filament_colour: string[] }).filament_colour,
  }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
