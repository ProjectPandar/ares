import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";
import { oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";
import { checkToolpathPreview } from "./toolpathPreview";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/preset-slice/", import.meta.url));

async function choose(page: Page, combo: string, group: string, name: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  await page.getByRole("listbox", { name: combo, exact: true }).getByRole("group", { name: group })
    .getByRole("option", { name, exact: true }).click();
  await expect(page.getByRole("button", { name: combo, exact: true })).toHaveText(name);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

async function exportGcode(page: Page) {
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  const path = `${artifacts}${download.suggestedFilename()}`;
  await download.saveAs(path);
  return path;
}

test("a 3MF plate and a centered STL sliced with selected system presets match OrcaSlicer --load-settings bytes", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the --load-settings oracle", "an extraction of the official OrcaSlicer 2.4.2 AppImage, see scripts/orca-parity.sh");
  const procgcode = oraclePath("ARES_PROCGCODE_BIN", "the OrcaSlicer GCodeProcessor toolpath oracle", "build it with tools/chain-probe/procgcode.nix");
  if (!orca) test.skip();
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
    "test(orca_parity_profile_entry_3mf_load_settings) | test(orca_parity_profile_entry_stl_center_placement)"], {
    cwd: repositoryRoot,
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      ARES_PARITY_PROFILE_ENTRY_REFERENCE: "1",
      ARES_ORCA_BIN: `${repositoryRoot}scripts/orca-parity.sh`,
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
  const native = normalize(readFileSync(`${work}reference-first/plate_1.gcode`, "utf8"), "OrcaSlicer");
  const nativeCenter = normalize(readFileSync(`${centerWork}center-reference/plate_1.gcode`, "utf8"), "OrcaSlicer");
  const centerNames = ["machine", "process", "filament"].map((kind) => JSON.parse(readFileSync(`${centerWork}center-${kind}.json`, "utf8")).name);
  expect(centerNames, "both native runs use the same presets").toEqual([selection.machine, selection.process, ...selection.filaments]);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(`${work}profile-entry.3mf`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");

  // ConfigWizard: install the LH vendor; its printers join the "System presets" group.
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "System presets" });
  await wizard.getByRole("checkbox", { name: "LH", exact: true }).check();
  await wizard.getByRole("button", { name: "Done" }).click();

  await choose(page, "Printer preset", "System presets", selection.machine);
  await choose(page, "Process preset", "System presets", selection.process);
  for (const [slot, filament] of selection.filaments.entries()) {
    await choose(page, `Filament ${slot + 1} preset`, "System presets", filament);
  }
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  await page.screenshot({ path: `${artifacts}prepare.png` });

  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const gcodePath = await exportGcode(page);
  const browser = readFileSync(gcodePath, "utf8");
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    oracle: "official OrcaSlicer 2.4.2 --load-settings on a 3MF (orca_parity_profile_entry_3mf_load_settings)",
    evidence: "ordered_bytes_generator_only",
    selection,
    bytes: browser.length,
  }, null, 2)}\n`);
  expect(normalize(browser, "Ares") === native, "browser G-code equals native --load-settings G-code").toBe(true);

  // The preset slice carries the engine's toolpaths for the preview.
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);
  if (procgcode) await checkToolpathPreview(page, procgcode, gcodePath, artifacts, errors);

  // An STL loaded on the empty plate is centered on the bed and dropped to it
  // (Plater.cpp:7056-7058, 7111-7112, 7149-7160): the presets stay selected.
  await page.getByRole("link", { name: "Prepare" }).click();
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/cube10.stl`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  const centered = readFileSync(await exportGcode(page), "utf8");
  writeFileSync(`${artifacts}result-stl.json`, `${JSON.stringify({
    oracle: "official OrcaSlicer 2.4.2 --load-settings on cube10.stl at the bed center (orca_parity_profile_entry_stl_center_placement)",
    evidence: "ordered_bytes_generator_only",
    selection,
    bytes: centered.length,
  }, null, 2)}\n`);
  expect(normalize(centered, "Ares") === nativeCenter, "browser STL G-code equals native G-code of the bed-centered model").toBe(true);
  expect(errors).toEqual([]);
});
