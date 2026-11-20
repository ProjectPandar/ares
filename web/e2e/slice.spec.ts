import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { oraclePath } from "./oracleEnv";
import { aresGeneratorLine, normalize } from "./orcaGcode";
import { openedSelection, withSelectedPresets } from "./selectedPresetLines";
import { checkToolpathPreview } from "./toolpathPreview";
import { oracleMoves } from "./oracleMoves";
import { checkToolPosition, expectNoToolPosition } from "./toolPosition";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;
const endTravel = "G1 X50 Y{print_bed_max[1]*0.5} F3000";

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/**
 * The fixture with one XY travel at 50 mm/s before the end G-code's present-print travel at
 * `travel_speed`. 2.5.0 `convert` starts a path when acceleration or jerk changes
 * (LibVGCodeWrapper.cpp:227-235), so only consecutive XY travels at different feedrates keep
 * differing Speed + Travel ends (`segmentColors.ts`); the plain fixture has none.
 */
function travelSpeedsProject(): string {
  const directory = `${artifacts}slice-input/`;
  mkdirSync(directory, { recursive: true });
  const entries = unzipSync(readFileSync(fixture));
  const name = "Metadata/project_settings.config";
  const config = JSON.parse(strFromU8(entries[name]));
  const present = "G1 X5 Y{print_bed_max[1]*0.8} F{travel_speed*60} ; present print\n";
  expect(config.machine_end_gcode).toContain(present);
  entries[name] = strToU8(JSON.stringify({ ...config, machine_end_gcode: config.machine_end_gcode.replace(present, `${endTravel}\n${present}`) }));
  const project = `${directory}orca_cli_ender3.3mf`;
  writeFileSync(project, zipSync(entries));
  return project;
}

/** Native reference: same project through ares-cli (built by `npm run test:e2e`). */
function nativeGcode(project: string, plate: string): Buffer {
  const output = `${artifacts}native.gcode`;
  execFileSync(`${repositoryRoot}target/release/ares`, ["slice", "--plate", plate, "-o", output, project]);
  return readFileSync(output);
}

/**
 * Official OrcaSlicer slicing the project with the edit as typed CLI options, as the M854 oracle
 * crates/ares-cli/tests/orca_parity/runner/user_overrides.rs does. OrcaSlicer.cpp:4138-4223 records
 * the changed keys in different_settings_to_system like a GUI edit (PresetBundle.cpp:5586-5595);
 * an edit inside the project config keeps the project's column (OrcaSlicer.cpp:1896, :3202-3207).
 */
function orcaEditedGcode(project: string, edit: Record<string, string>): string {
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-project-edit-`)}/`;
  const options = Object.entries(edit).flatMap(([key, value]) => [`--${key.replaceAll("_", "-")}`, value]);
  execFileSync(`${repositoryRoot}scripts/orca-parity.sh`, [...options, "--slice", "0", "--outputdir", work, project], {
    cwd: work, stdio: ["ignore", "pipe", "pipe"],
  });
  const gcode = readFileSync(`${work}plate_1.gcode`, "utf8");
  writeFileSync(`${artifacts}project-edit/native.gcode`, gcode);
  return normalize(gcode, "OrcaSlicer");
}

/** True when the WebGL canvas shows more than a flat background. */
async function canvasHasContent(page: Page) {
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const probe = document.createElement("canvas");
    probe.width = 64;
    probe.height = 64;
    const context = probe.getContext("2d")!;
    context.drawImage(canvas, 0, 0, 64, 64);
    const pixels = context.getImageData(0, 0, 64, 64).data;
    const colours = new Set<number>();
    for (let index = 0; index < pixels.length; index += 4) {
      colours.add((pixels[index] << 16) | (pixels[index + 1] << 8) | pixels[index + 2]);
    }
    return colours.size;
  });
}

/** Re-slices the plate `runs` times; each sample is Slice plate click to "Slice complete" in ms. */
async function sliceTimings(page: Page, runs = 20) {
  const samples: number[] = [];
  for (let run = 0; run < runs; run++) {
    samples.push(await page.evaluate(() => new Promise<number>((resolve) => {
      const status = document.querySelector("[data-testid=status]")!;
      const observer = new MutationObserver(() => {
        if (!status.textContent!.startsWith("Slice complete")) return;
        observer.disconnect();
        resolve(Math.round((performance.now() - started) * 10) / 10);
      });
      observer.observe(status, { childList: true, subtree: true, characterData: true });
      const started = performance.now();
      [...document.querySelectorAll("button")].find((button) => button.textContent === "Slice plate")!.click();
    })));
  }
  return samples;
}

test("import 3MF, slice in the WASM worker, export and preview G-code", async ({ page }) => {
  // GCodeProcessor oracle (tools/chain-probe/procgcode.nix) of the toolpath preview.
  const orca = oraclePath("ORCA_APPDIR", "the project-edit oracle", "an extraction of the official OrcaSlicer 2.5.0 AppImage, see scripts/orca-parity.sh");
  const oracle = oraclePath("ARES_PROCGCODE_BIN", "the OrcaSlicer GCodeProcessor toolpath oracle", "build it with tools/chain-probe/procgcode.nix");
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`${message.text()} (${message.location().url})`);
  });

  const loadStarted = Date.now();
  await page.goto("/");
  await expect(page).toHaveURL(/#\/prepare$/);
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  const engineReadyMs = Date.now() - loadStarted;

  const project = travelSpeedsProject();
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect(page.getByTestId("object-list").getByRole("treeitem")).not.toHaveCount(0);
  await expect(page.getByTestId("sidebar")).toContainText("Layer height");
  expect(await canvasHasContent(page)).toBeGreaterThan(8);
  await page.screenshot({ path: `${artifacts}prepare.png` });

  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  // First slice as the status bar reports it (worker round trip, cold), then warm re-slices.
  const firstSliceS = Number(/in ([\d.]+) s$/.exec((await page.getByTestId("status").textContent())!)![1]);
  const sliceMs = await sliceTimings(page);

  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  expect(download.suggestedFilename()).toBe("orca_cli_ender3_plate_1.gcode");
  const gcodePath = `${artifacts}${download.suggestedFilename()}`;
  await download.saveAs(gcodePath);
  const gcode = readFileSync(gcodePath);
  const text = gcode.toString("utf8");
  expect(text.split("\n")).toContain(aresGeneratorLine);
  expect(text).toMatch(/^G1 X[\d.-]+ Y[\d.-]+ E[\d.]+/m);
  expect(text.match(/^;LAYER_CHANGE$/gm)?.length).toBe(50);
  expect(text).not.toMatch(/^G[0-3] .*\b(?:NaN|inf)\b/m);

  const selection = openedSelection(project, `${artifacts}slice-selection/`);
  if (selection) {
    const expected = withSelectedPresets(nativeGcode(project, "1").toString("utf8"), selection);
    writeFileSync(`${artifacts}expected.gcode`, expected);
    expect(sha256(gcode), "browser G-code must match native ares-cli bytes with the plater's selected presets").toBe(sha256(Buffer.from(expected)));
  } else {
    test.info().annotations.push({ type: "skip", description: "ARES_ORCA_PRESET_SELECT_BIN is unset: the ares-cli byte comparison needs the plater's selected presets and did not run" });
  }

  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page).toHaveURL(/#\/preview$/);
  await expect(page.getByTestId("gcode-bytes")).toHaveText(`${gcode.length.toLocaleString("en-US")} bytes`);
  // The engine returns move vertices, so the preview has toolpaths and a current G-code line.
  await expect(page.getByTestId("gcode-panel")).toBeVisible();
  expect(await canvasHasContent(page)).toBeGreaterThan(8);
  await page.screenshot({ path: `${artifacts}preview.png` });
  await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);

  if (oracle) await checkToolpathPreview(page, oracle, gcodePath, artifacts, errors, true);

  // A process edit slices the project with the edit applied over its config (M854): the G-code is
  // byte-equal to OrcaSlicer's for the project carrying the same value.
  const layerHeight = page.getByRole("textbox", { name: "layer_height", exact: true });
  await layerHeight.fill("0.12");
  await layerHeight.press("Enter");
  await expect(page.getByTestId("process-edited")).toContainText("1 modified option");
  await expect(layerHeight).toHaveValue("0.12");
  await expect(page.getByTestId("slice-summary")).toContainText("Plate not sliced");
  await page.screenshot({ path: `${artifacts}process-edited.png` });
  await page.getByRole("button", { name: "Slice plate" }).first().click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  mkdirSync(`${artifacts}project-edit/`, { recursive: true });
  const [editedDownload] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  const editedPath = `${artifacts}project-edit/${editedDownload.suggestedFilename()}`;
  await editedDownload.saveAs(editedPath);
  const edited = readFileSync(editedPath, "utf8");
  expect(edited === gcode.toString("utf8"), "the edit changes the browser output").toBe(false);
  await page.getByRole("link", { name: "Preview" }).click();
  await expectNoToolPosition(page);
  if (oracle) {
    const moves = oracleMoves(oracle, editedPath, `${artifacts}project-edit/`);
    await page.getByRole("slider", { name: "Top layer" }).focus();
    await page.keyboard.press("Home");
    const slider = page.getByRole("slider", { name: "Moves" });
    await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
    await slider.fill("0");
    await slider.fill((await slider.getAttribute("max"))!);
    await checkToolPosition(page, moves, 0, `${artifacts}project-edit/`);
  }
  if (orca) {
    if (!selection) throw new Error("ARES_ORCA_PRESET_SELECT_BIN is unset while ORCA_APPDIR is set: the project-edit comparison needs the plater's selected presets");
    const nativeEdited = withSelectedPresets(orcaEditedGcode(project, { layer_height: "0.12" }), selection);
    writeFileSync(`${artifacts}project-edit/result.json`, `${JSON.stringify({
      oracle: "official OrcaSlicer 2.5.0 on the project with the edit as typed CLI options",
      evidence: "ordered_bytes_generator_only",
      edit: { layer_height: "0.12" },
      bytes: edited.length,
    }, null, 2)}\n`);
    expect(normalize(edited, "Ares") === nativeEdited, "edited browser G-code equals OrcaSlicer's for the edited project").toBe(true);
  }
  await page.getByRole("button", { name: "Reset all" }).click();
  await expect(page.getByTestId("process-edited")).toHaveCount(0);
  await expect(layerHeight).not.toHaveValue("0.12");

  writeFileSync(
    `${artifacts}result.json`,
    `${JSON.stringify({
      fixture: "tests/parity/orca_cli_ender3.3mf", machineEndGcodeTravel: endTravel, bytes: gcode.length, sha256: sha256(gcode),
      engineReadyMs, firstSliceS, sliceMs, sliceMedianMs: [...sliceMs].sort((a, b) => a - b)[sliceMs.length >> 1],
    }, null, 2)}\n`,
  );
  writeFileSync(`${artifacts}gcode.sha256`, `${sha256(gcode)}  ${download.suggestedFilename()}\n`);
  expect(errors).toEqual([]);
});
