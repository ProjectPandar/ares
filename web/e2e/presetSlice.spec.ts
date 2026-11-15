import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { nativeEnv, oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";
import { withPlaterBedType } from "./platerExport";
import { nativeSelection, presetSelectProbe } from "./presetSelectOracle";
import { setConfigLine, withSelectedPresets } from "./selectedPresetLines";
import { checkToolpathPreview } from "./toolpathPreview";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;
const artifacts = fileURLToPath(new URL("../../target/web-e2e/preset-slice/", import.meta.url));
// A sidebar process edit that differs from the selected process preset's value.
const EDIT = { key: "layer_height", value: "0.3" };

async function choose(page: Page, combo: string, group: string, name: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  await page.getByRole("listbox", { name: combo, exact: true }).getByRole("group", { name: group })
    .getByRole("option", { name, exact: true }).click();
  await expect(page.getByRole("button", { name: combo, exact: true })).toHaveText(name);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

/**
 * Centroid of the model-coloured (filament teal) viewport pixels relative to the canvas centre, in
 * canvas-width fractions, with their count. The bed axes (ColorRGB::X/Y/Z) are excluded by hue.
 */
function modelOffset(page: Page) {
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const probe = document.createElement("canvas");
    [probe.width, probe.height] = [400, Math.round((400 * canvas.height) / canvas.width)];
    const context = probe.getContext("2d")!;
    context.drawImage(canvas, 0, 0, probe.width, probe.height);
    const pixels = context.getImageData(0, 0, probe.width, probe.height).data;
    let [count, sumX, sumY] = [0, 0, 0];
    for (let index = 0; index < pixels.length; index += 4) {
      const [r, g, b] = [pixels[index], pixels[index + 1], pixels[index + 2]];
      if (g - r > 60 && b - r > 60 && Math.abs(g - b) < 50) {
        const pixel = index / 4;
        count++;
        sumX += pixel % probe.width;
        sumY += Math.floor(pixel / probe.width);
      }
    }
    return { count, x: (sumX / count - probe.width / 2) / probe.width, y: (sumY / count - probe.height / 2) / probe.width };
  });
}

/** `BuildVolume::bed_center` (BuildVolume.hpp:83): the printable_area bbox centre of a raw vendor machine, through `inherits`. */
function bedCenter(vendor: string, machine: string): [number, number] {
  const dir = `${repositoryRoot}OrcaSlicer/resources/profiles/${vendor}/machine/`;
  const field = (name: string): string[] => {
    const json = JSON.parse(readFileSync(`${dir}${name}.json`, "utf8")) as { printable_area?: string[]; inherits: string };
    return json.printable_area ?? field(json.inherits);
  };
  const points = field(machine).map((point) => point.split("x").map(Number));
  const [xs, ys] = [points.map(([x]) => x), points.map(([, y]) => y)];
  return [(Math.min(...xs) + Math.max(...xs)) / 2, (Math.min(...ys) + Math.max(...ys)) / 2];
}

/** GCode.cpp:8272 writes filament_multi_colour only when the loaded config carries it. */
function multiColour(gcode: string) {
  return gcode.split("\n").filter((line) => line.startsWith("; filament_multi_colour ="));
}

/**
 * The GuideFrame "All LH models" check: every LH model with each nozzle variant, the model's `nozzle_diameter`
 * entries (PresetBundle.cpp:8319-8327).
 */
function lhModels() {
  const vendor = JSON.parse(readFileSync(`${profiles}LH.json`, "utf8")) as { machine_model_list: { name: string; sub_path: string }[] };
  return Object.fromEntries(vendor.machine_model_list.map(({ name, sub_path }) =>
    [name, (JSON.parse(readFileSync(`${profiles}LH/${sub_path}`, "utf8")) as { nozzle_diameter: string }).nozzle_diameter.split(";")]));
}

/**
 * The STL reference with the plater's `filament_multi_colour`: with no project open, load_selections copies the
 * filament colours into it (PresetBundle.cpp:4116-4117).
 */
function withLoadedMultiColour(gcode: string) {
  const lines = gcode.split("\n");
  const colour = lines.find((line) => line.startsWith("; filament_colour = "))!.slice("; filament_colour = ".length);
  setConfigLine(lines, "filament_multi_colour", colour);
  return lines.join("\n");
}

async function exportGcode(page: Page, directory: string) {
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  const path = `${directory}${download.suggestedFilename()}`;
  await download.saveAs(path);
  return path;
}

test("a 3MF plate with and without a sidebar edit and a centered STL sliced with selected system presets match OrcaSlicer --load-settings bytes", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the --load-settings oracle", "an extraction of the official OrcaSlicer 2.5.0-beta AppImage, see scripts/orca-parity.sh");
  const procgcode = oraclePath("ARES_PROCGCODE_BIN", "the OrcaSlicer GCodeProcessor toolpath oracle", "build it with tools/chain-probe/procgcode.nix");
  if (!orca) test.skip();
  const seeded = process.env.ARES_PARITY_SEEDED_REFERENCE === "1";
  const launcher = seeded ? process.env.ARES_ORCA_SEEDED_BIN : `${repositoryRoot}scripts/orca-parity.sh`;
  if (!launcher) throw new Error("Explicit seeded mode requires ARES_ORCA_SEEDED_BIN");
  const probe = presetSelectProbe();
  if (!probe) throw new Error("ARES_ORCA_PRESET_SELECT_BIN is required for the plater session's selected-preset lines");
  test.setTimeout(600_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });

  // Oracle: the public 3MF profile-entry parity test runs official OrcaSlicer on a 3MF with
  // `--load-settings machine;process --load-filaments filament` (OrcaSlicer.cpp:1624-1640) and
  // writes the project, the selection and the native plate G-code. The STL center-placement test
  // slices cube10.stl pre-placed at the center of the rectangular LH Stinger bed (printable_area
  // 18x38..218x238), where Plater::priv::load_model_objects puts a model loaded on an empty plate.
  const nativeRoot = `${mkdtempSync(`${tmpdir()}/ares-web-preset-oracle-`)}/`;
  execFileSync("cargo", ["nextest", "run", "-p", "ares-cli", "-E",
    "test(profile_entry_3mf_load_settings) | test(profile_entry_stl_center_placement)"], {
    cwd: repositoryRoot,
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      ARES_PARITY_PROFILE_ENTRY_REFERENCE: "1",
      ARES_ORCA_BIN: launcher,
      ...(seeded ? {} : nativeEnv(orca!)),
      ARES_PARITY_ARTIFACT_ROOT: nativeRoot,
    },
  });
  const runs = readdirSync(nativeRoot).filter((name) => name.startsWith("orca-runner-")).map((name) => `${nativeRoot}${name}/`);
  const work = runs.find((dir) => existsSync(`${dir}selection.json`))!;
  const centerWork = runs.find((dir) => existsSync(`${dir}center-reference`))!;
  cpSync(work, `${artifacts}native/`, { recursive: true });
  cpSync(centerWork, `${artifacts}native-center/`, { recursive: true });
  expect(JSON.parse(readFileSync(`${work}result.json`, "utf8"))).toMatchObject({ native_repeat: "PASS" });
  const selection = JSON.parse(readFileSync(`${work}selection.json`, "utf8")) as { machine: string; process: string; filaments: string[] };
  // The plater session the browser replays: the first-run app opens the 3MF, the GuideFrame wizard installs every
  // LH model and the sidebar picks the selected LH system presets. The browser slices it on the explicit preset
  // route, the --load-settings model, so each reference is the CLI slice with this session's selected-preset lines.
  const session = nativeSelection(probe, `${artifacts}native/plater-session/`, `${work}profile-entry.3mf`, [], {
    wizard: { LH: lhModels() },
    select: { printer: selection.machine, process: selection.process, filaments: selection.filaments },
  });
  const inputs = JSON.parse(readFileSync(`${work}native-profile-inputs.json`, "utf8")) as { datadir: string; settings: string[]; filaments: string[] };
  const loadPresets = ["--datadir", inputs.datadir, "--load-settings", inputs.settings.join(";"), "--load-filaments", inputs.filaments.join(";")];
  // The session's picks run load_selections, which replaces the 3MF's curr_bed_type with the selected printer's
  // (PresetBundle.cpp:4046-4055, Preset.cpp:1140-1171); the 3MF references slice with the session's.
  const platerBed = withPlaterBedType(session, `${work}profile-entry.3mf`, `${work}profile-entry-plater-bed.3mf`);
  mkdirSync(`${work}selected-system-reference`);
  execFileSync(launcher, [
    ...loadPresets, "--slice", "0", "--outputdir", `${work}selected-system-reference`, platerBed,
  ], { cwd: work, stdio: ["ignore", "pipe", "pipe"] });
  cpSync(`${work}selected-system-reference`, `${artifacts}native/selected-system-reference`, { recursive: true });
  const native = withSelectedPresets(normalize(readFileSync(`${work}selected-system-reference/plate_1.gcode`, "utf8"), "OrcaSlicer"), session);
  mkdirSync(`${centerWork}selected-system-reference`);
  execFileSync(launcher, [
    ...loadPresets, "--arrange", "0", "--slice", "0", "--outputdir", `${centerWork}selected-system-reference`, `${centerWork}cube10.stl`,
  ], { cwd: centerWork, stdio: ["ignore", "pipe", "pipe"] });
  cpSync(`${centerWork}selected-system-reference`, `${artifacts}native-center/selected-system-reference`, { recursive: true });
  const nativeCenter = withLoadedMultiColour(normalize(readFileSync(`${centerWork}selected-system-reference/plate_1.gcode`, "utf8"), "OrcaSlicer"));
  const centerNames = ["machine", "process", "filament"].map((kind) => JSON.parse(readFileSync(`${centerWork}center-${kind}.json`, "utf8")).name);
  expect(centerNames, "both native runs use the same presets").toEqual([selection.machine, selection.process, ...selection.filaments]);

  // Preserve the edit snapshot, but apply the Native CLI edit after pristine vendor loading
  // (OrcaSlicer.cpp:2118-2157; Config.cpp:461 apply_only: later values win).
  const processPreset = JSON.parse(readFileSync(`${work}process.json`, "utf8")) as Record<string, unknown>;
  writeFileSync(`${work}process-override.json`, JSON.stringify({ ...processPreset, [EDIT.key]: EDIT.value }, null, 2));
  mkdirSync(`${work}override-reference`);
  execFileSync(launcher, [
    ...loadPresets, "--layer-height", String(EDIT.value),
    "--slice", "0", "--outputdir", `${work}override-reference`, platerBed,
  ], { cwd: work, stdio: ["ignore", "pipe", "pipe"] });
  cpSync(`${work}override-reference`, `${artifacts}native/override-reference`, { recursive: true });
  const nativeEdited = withSelectedPresets(normalize(readFileSync(`${work}override-reference/plate_1.gcode`, "utf8"), "OrcaSlicer"), session);
  expect(nativeEdited === native, "the edit changes the native output").toBe(false);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(`${work}profile-entry.3mf`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  // GuideFrame (GUI_App.cpp:9997): install the LH vendor; its printers join the "System presets" group.
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All LH models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });

  // The plater combo lists the printer model (PresetComboBoxes.cpp:1238-1251); selecting it
  // resolves the preset with PresetBundle::get_similar_printer_preset (Plater.cpp:9565-9572),
  // here the 3MF's own printer, which the byte comparison below checks.
  const machine = JSON.parse(readFileSync(`${work}machine.json`, "utf8")) as { printer_model: string };
  await choose(page, "Printer preset", "System presets", machine.printer_model);
  await expect(page.getByRole("button", { name: "Printer preset", exact: true })).toHaveAttribute("title", selection.machine);
  await choose(page, "Process preset", "System presets", selection.process);
  for (const [slot, filament] of selection.filaments.entries()) {
    await choose(page, `Filament ${slot + 1} preset`, "System presets", filament);
  }
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  await page.screenshot({ path: `${artifacts}prepare.png` });

  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const gcodePath = await exportGcode(page, artifacts);
  const browser = readFileSync(gcodePath, "utf8");
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    oracle: "official OrcaSlicer 2.5.0-beta --load-settings on a 3MF (profile_entry_3mf_load_settings)",
    evidence: "ordered_bytes_generator_only",
    selection,
    bytes: browser.length,
  }, null, 2)}\n`);
  expect(multiColour(browser), "filament_multi_colour config line").toEqual(multiColour(native));
  expect(normalize(browser, "Ares") === native, "browser G-code equals native --load-settings G-code").toBe(true);

  // The preset slice carries the engine's toolpaths for the preview.
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);
  if (procgcode) await checkToolpathPreview(page, procgcode, gcodePath, artifacts, errors, false);

  // Sidebar edits ride on the preset slice as overrides applied after the selected presets.
  await page.getByRole("link", { name: "Prepare" }).click();
  const field = page.getByRole("textbox", { name: EDIT.key, exact: true });
  await expect(field, "process settings stay editable with presets selected").toBeVisible();
  await expect(field, "the field shows the selected process preset value").toHaveValue(String(processPreset[EDIT.key]));
  await field.fill(EDIT.value);
  await field.press("Enter");
  await expect(page.getByTestId("process-edited")).toContainText("1 modified option");
  await page.getByTestId("sidebar").screenshot({ path: `${artifacts}edited/sidebar.png` });
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const editedPath = await exportGcode(page, `${artifacts}edited/`);
  const browserEdited = readFileSync(editedPath, "utf8");
  writeFileSync(`${artifacts}edited/result.json`, `${JSON.stringify({
    oracle: "OrcaSlicer selected vendor presets followed by the CLI layer-height edit",
    evidence: "ordered_bytes_generator_only",
    edit: EDIT,
    bytes: browserEdited.length,
  }, null, 2)}\n`);
  expect(multiColour(browserEdited), "edited filament_multi_colour config line").toEqual(multiColour(nativeEdited));
  expect(normalize(browserEdited, "Ares") === nativeEdited, "edited browser G-code equals native --load-settings G-code").toBe(true);

  // An STL loaded on the empty plate is centered on the bed and dropped to it
  // (Plater.cpp:7056-7058, 7111-7112, 7149-7160): the presets stay selected.
  await page.getByRole("link", { name: "Prepare" }).click();
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/cube10.stl`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  // The sidebar always has the first filament combo (Plater.cpp:2188-2191); without project filaments it
  // keeps the selected filament (PresetBundle.cpp:435-436), the one the STL slices with.
  await expect(page.getByRole("button", { name: "Filament 1 preset", exact: true })).toHaveText(selection.filaments[0]);
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  // The Prepare viewport shows the model where the slice places it: framed on the selected printer's
  // plate and seen from the top, the plate centre - where --center puts the model
  // (OrcaSlicer.cpp:4376-4392, Model.cpp:704-722 around BuildVolume::bed_center) - is the canvas centre.
  const viewToolbar = page.getByTestId("view-toolbar");
  await viewToolbar.getByRole("button", { name: "Fit to plate" }).click();
  await viewToolbar.getByRole("button", { name: "Top", exact: true }).click();
  await expect.poll(async () => (await modelOffset(page)).count, "the model is drawn").toBeGreaterThan(50);
  const offset = await modelOffset(page);
  await page.screenshot({ path: `${artifacts}prepare-stl-top.png` });
  writeFileSync(`${artifacts}prepare-stl.json`, `${JSON.stringify({ bedCenter: bedCenter("LH", selection.machine), offset }, null, 2)}\n`);
  expect(Math.hypot(offset.x, offset.y), "model centred on the selected printer's plate").toBeLessThan(0.01);
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const centered = readFileSync(await exportGcode(page, artifacts), "utf8");
  writeFileSync(`${artifacts}result-stl.json`, `${JSON.stringify({
    oracle: "official OrcaSlicer 2.5.0-beta --load-settings on cube10.stl at the bed center (profile_entry_stl_center_placement)",
    evidence: "ordered_bytes_generator_only",
    selection,
    bytes: centered.length,
  }, null, 2)}\n`);
  expect(multiColour(centered), "STL filament_multi_colour config line").toEqual(multiColour(nativeCenter));
  expect(normalize(centered, "Ares") === nativeCenter, "browser STL G-code equals native G-code of the bed-centered model").toBe(true);
  expect(errors).toEqual([]);
});
