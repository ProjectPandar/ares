import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";
import { checkToolpathPreview } from "./toolpathPreview";
import { oracleMoves } from "./oracleMoves";
import { checkToolPosition, expectNoToolPosition } from "./toolPosition";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/** Native reference: same project through ares-cli (built by `npm run test:e2e`). */
function nativeGcode(plate: string): Buffer {
  const output = `${artifacts}native.gcode`;
  execFileSync(`${repositoryRoot}target/release/ares`, ["slice", "--plate", plate, "-o", output, fixture]);
  return readFileSync(output);
}

/**
 * Official OrcaSlicer slicing a copy of the project whose `Metadata/project_settings.config`
 * (bbs_3mf.cpp:210, loaded at :2647) carries the edit: the PresetBundle::full_config ->
 * Print::apply path a GUI edit takes (Config.cpp:573 set_deserialize_nothrow, PrintApply.cpp:1113),
 * as the M854 oracle crates/ares-cli/tests/orca_parity/runner/user_overrides.rs does.
 */
function orcaEditedGcode(edit: Record<string, string>): string {
  const work = `${mkdtempSync(`${tmpdir()}/ares-web-project-edit-`)}/`;
  const entries = unzipSync(readFileSync(fixture));
  const config = "Metadata/project_settings.config";
  entries[config] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[config])), ...edit }));
  writeFileSync(`${work}edited.3mf`, zipSync(entries));
  execFileSync(`${repositoryRoot}scripts/orca-parity.sh`, ["--slice", "0", "--outputdir", work, `${work}edited.3mf`], {
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
  const orca = oraclePath("ORCA_APPDIR", "the project-edit oracle", "an extraction of the official OrcaSlicer 2.5.0-beta AppImage, see scripts/orca-parity.sh");
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

  await page.getByTestId("import-input").setInputFiles(fixture);
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
  expect(text).toContain("; generated by Ares");
  expect(text).toMatch(/^G1 X[\d.-]+ Y[\d.-]+ E[\d.]+/m);
  expect(text.match(/^;LAYER_CHANGE$/gm)?.length).toBe(50);
  expect(text).not.toMatch(/^G[0-3] .*\b(?:NaN|inf)\b/m);

  const native = nativeGcode("1");
  expect(sha256(gcode), "browser G-code must match native ares-cli bytes").toBe(sha256(native));

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
    const nativeEdited = orcaEditedGcode({ layer_height: "0.12" });
    writeFileSync(`${artifacts}project-edit/result.json`, `${JSON.stringify({
      oracle: "official OrcaSlicer 2.5.0-beta on the project with the edit in Metadata/project_settings.config",
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
      fixture: "tests/parity/orca_cli_ender3.3mf", bytes: gcode.length, sha256: sha256(gcode),
      engineReadyMs, firstSliceS, sliceMs, sliceMedianMs: [...sliceMs].sort((a, b) => a - b)[sliceMs.length >> 1],
    }, null, 2)}\n`,
  );
  writeFileSync(`${artifacts}gcode.sha256`, `${sha256(gcode)}  ${download.suggestedFilename()}\n`);
  expect(errors).toEqual([]);
});
