import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, unzipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";
import { nativeSelection, presetSelectProbe } from "./presetSelectOracle";
import { projectConfig, saveProjectFixture } from "./saveProjectFixture";

const root = fileURLToPath(new URL("../../", import.meta.url));
const status = (page: Page) => page.getByTestId("status");
const fileMenu = (page: Page) => page.getByRole("button", { name: "File", exact: true });
const saveItem = (page: Page, name: string) => page.getByRole("menuitem", { name, exact: true });
const object = (page: Page) => page.getByTestId("object-list").getByRole("treeitem", { name: /Bracket/ }).filter({ hasNotText: "Bracket plate" });
const field = (page: Page, name: string) => page.getByRole("textbox", { name, exact: true });
const commit = async (page: Page, name: string, value: string) => {
  await field(page, name).fill(value);
  await field(page, name).press("Enter");
};
const selectQuality = (page: Page) => page.getByTestId("process-settings").getByRole("tab", { name: "Quality", exact: true }).click();

async function openProject(page: Page, project: string | Buffer) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(typeof project === "string" ? project : {
    name: "Brackets.3mf", mimeType: "model/3mf", buffer: project,
  });
  await expect(status(page)).toHaveText("Ready to slice", { timeout: 60_000 });
}

async function downloadProject(page: Page, label: string, destination: string) {
  await fileMenu(page).click();
  const save = saveItem(page, label);
  await expect(save).toBeEnabled({ timeout: 5_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), save.click()]);
  expect(download.suggestedFilename()).toBe("Brackets.3mf");
  await download.saveAs(destination);
}

type Geometry = {
  objects: { name: string; instances: { printable: boolean; matrix: number[] }[] }[];
  plates: number[][][];
};
async function geometry(page: Page): Promise<Geometry> {
  const { objects, plates } = JSON.parse((await page.locator(".viewport").getAttribute("data-instances"))!) as Geometry;
  return { objects, plates };
}

function comparedMembers(path: string) {
  const members = Object.entries(unzipSync(readFileSync(path))).map(([name, contents]) =>
    [name, createHash("sha256").update(contents).digest("hex")] as const);
  const objects = /^3D\/Objects\/[^/]+\.model$/;
  return {
    ordered: members.filter(([name]) => !objects.test(name)),
    objects: members.filter(([name]) => objects.test(name)).sort(([left], [right]) => left.localeCompare(right)),
  };
}

function nativeExport(project: string, destination: string, leg: "official" | "seeded") {
  const variable = leg === "official" ? "ORCA_APPDIR" : "ORCA_REPLAY_BIN";
  if (!oraclePath(variable, `${leg} OrcaSlicer 2.5.0`, "the pinned 3MF export reference")) return false;
  const work = resolve(destination, "..");
  mkdirSync(work, { recursive: true });
  const script = leg === "official" ? "orca-parity.sh" : "orca-seeded-parity.sh";
  const cpu = /^Cpus_allowed_list:\s*(\d+)/m.exec(readFileSync("/proc/self/status", "utf8"))![1];
  const argv = ["--arrange", "0", "--orient", "0", "--outputdir", work, "--export-3mf", "exported.3mf", project];
  execFileSync(join(root, "scripts", script), argv, {
    cwd: work, env: { ...process.env, ORCA_REPLAY_CPU: cpu, ORCA_REPLAY_SEED: "0" },
    stdio: ["ignore", "pipe", "pipe"],
  });
  return true;
}

test("Save Project downloads edited objects, plate assignments and settings that reopen", async ({ page, context }, info) => {
  // The headless executable checks compatibility against system presets; the GUI
  // also accepts project-local presets when this vendor is missing.
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const printers = page.getByRole("dialog", { name: "Printer Selection" });
  await printers.getByRole("checkbox", { name: "Bambu Lab X1 Carbon", exact: true }).check();
  await printers.getByRole("button", { name: "Confirm" }).click();
  await expect(printers).toHaveCount(0, { timeout: 60_000 });
  const opened = info.outputPath("opened", "Brackets.3mf");
  mkdirSync(resolve(opened, ".."), { recursive: true });
  const probe = presetSelectProbe();
  let expectedConfig: string | undefined;
  if (probe) {
    const template = info.outputPath("template", "Brackets.3mf");
    mkdirSync(resolve(template, ".."), { recursive: true });
    writeFileSync(template, saveProjectFixture());
    const { full_fff_config: config } = nativeSelection(probe, `${info.outputPath("native-unedited")}/`, template, ["BBL"]);
    writeFileSync(opened, saveProjectFixture({ config }));
    expectedConfig = nativeSelection(probe, `${info.outputPath("native-edited")}/`, opened, ["BBL"], {
      process_edit: { layer_height: "0.27" },
    }).full_fff_config;
  } else {
    writeFileSync(opened, saveProjectFixture());
  }
  await openProject(page, opened);
  await object(page).click();
  await commit(page, "Position X", "145.5");
  await expect(field(page, "Position X")).toHaveValue("145.50");
  await commit(page, "Rotate (absolute) Z", "60");
  await expect(field(page, "Rotate (absolute) Z")).toHaveValue("60.00");
  await selectQuality(page);
  await commit(page, "layer_height", "0.27");
  await expect(field(page, "layer_height")).toHaveValue("0.27");
  const before = await geometry(page);
  expect(before.objects.map(({ name, instances }) => [name, instances.length])).toEqual([["Bracket", 1], ["Shim", 2]]);
  expect(before.plates).toEqual([[[0, 0]], [[1, 0], [1, 1]]]);

  const downloaded = info.outputPath("Brackets.3mf");
  await downloadProject(page, "Save Project as…", downloaded);
  const repeat = info.outputPath("save.3mf");
  await downloadProject(page, "Save Project", repeat);
  expect(comparedMembers(repeat)).toEqual(comparedMembers(downloaded));

  const archive = unzipSync(readFileSync(downloaded));
  const saved = JSON.parse(strFromU8(archive[projectConfig]));
  expect(saved.layer_height).toBe("0.27");
  if (expectedConfig !== undefined) {
    const expected = JSON.parse(expectedConfig);
    for (const key of ["filament_colour_type", "flush_volumes_matrix", "flush_volumes_vector"]) {
      expect(expected[key], `Native ${key}`).toBeDefined();
      expect(saved[key], `saved ${key}`).toEqual(expected[key]);
    }
  }
  expect(strFromU8(archive["Metadata/model_settings.config"])).not.toContain('key="gcode_file"');
  expect(Object.keys(archive).filter((name) => /\.gcode(?:\.md5)?$/.test(name))).toEqual([]);

  const fresh = await context.newPage();
  await openProject(fresh, downloaded);
  const after = await geometry(fresh);
  expect(after.plates).toEqual(before.plates);
  expect(after.objects.map(({ name, instances }) => [name, instances.length])).toEqual([["Bracket", 1], ["Shim", 2]]);
  for (const [at, model] of after.objects.entries()) {
    for (const [index, item] of model.instances.entries()) {
      const expected = before.objects[at].instances[index];
      expect(item.printable).toBe(expected.printable);
      item.matrix.forEach((value, i) => expect(value).toBeCloseTo(expected.matrix[i], 9));
    }
  }
  await selectQuality(fresh);
  await expect(field(fresh, "layer_height")).toHaveValue("0.27");
  await fresh.close();

  if (expectedConfig === undefined) return;
  const reference = info.outputPath("reference", "Brackets.3mf");
  mkdirSync(resolve(reference, ".."), { recursive: true });
  writeFileSync(reference, saveProjectFixture({ config: expectedConfig, edited: true }));
  for (const leg of ["official", "seeded"] as const) {
    const fromWeb = info.outputPath(leg, "download", "exported.3mf");
    const fromReference = info.outputPath(leg, "reference", "exported.3mf");
    if (!nativeExport(downloaded, fromWeb, leg)) continue;
    nativeExport(reference, fromReference, leg);
    expect(comparedMembers(fromWeb), `${leg} OrcaSlicer re-export`).toEqual(comparedMembers(fromReference));
  }
});

test("saving a project with an unsupported negative volume explains why it cannot be saved", async ({ page }) => {
  await openProject(page, saveProjectFixture({ modifier: true }));
  const downloads: string[] = [];
  page.on("download", (download) => downloads.push(download.suggestedFilename()));
  await fileMenu(page).click();
  const save = saveItem(page, "Save Project as…");
  await expect(save).toBeEnabled({ timeout: 5_000 });
  await save.click();
  await expect(status(page)).toContainText(/save.*negative (part|volume)/i);
  expect(downloads).toEqual([]);
});
