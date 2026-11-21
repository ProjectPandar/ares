import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";
import { presetSelectProbe } from "./presetSelectOracle";
import { cstyle, openedSelection, withGuiMissingKeys } from "./selectedPresetLines";

const root = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = `${root}target/web-e2e/estimation`;
const FIXTURES = ["time-cost", "unused-filament"];

type Volumes = Record<string, number>;
interface Native {
  total_used_filament: number; total_weight: number; total_cost: number;
  normal_time: number; normal_prepare_time: number; stealth_time: number;
  footer_times: [string, string, string];
  total_volumes_per_extruder: Volumes; support_volumes_per_extruder: Volumes;
  flush_per_filament: Volumes; wipe_tower_volumes_per_extruder: Volumes;
  filament_diameters: number[]; filament_densities: number[];
}

/**
 * `plate_1.print_statistics.json` of tools/chain-probe/print-statistics.nix slicing `input`,
 * pinned to the one CPU `ORCA_REPLAY_CPU` names with the seeded reference's seed like
 * `OrcaRunner::print_statistics_reference` (scripts/orca-seeded-parity.sh). Written to target/, never the repo.
 */
function native(bin: string, name: string, input: string): Native {
  const directory = `${artifacts}/${name}`;
  const cpu = process.env.ORCA_REPLAY_CPU;
  if (!cpu || !/^\d+$/.test(cpu)) throw new Error(`ORCA_REPLAY_CPU=${cpu ?? "(unset)"} must name the one CPU the Native probe runs on`);
  mkdirSync(directory, { recursive: true });
  const run = spawnSync("taskset", ["-c", cpu, bin, "--slice", "0", "--outputdir", directory, input], {
    env: { ...process.env, ORCA_REPLAY_SEED: "0" }, encoding: "utf8", timeout: 290_000, killSignal: "SIGKILL",
  });
  writeFileSync(`${directory}/native.log`, `${run.stdout}\n${run.stderr}`);
  if (run.error) throw run.error;
  expect(run.status, `native slice of ${name}`).toBe(0);
  return JSON.parse(readFileSync(`${directory}/plate_1.print_statistics.json`, "utf8")) as Native;
}

/**
 * The reference config block prints each added GUI value: a scalar as is, a vector comma- or cstyle-joined, or with
 * the same elements where slicing resizes it to the printer's extruders (PresetBundle.cpp:253-306). filament_map_2
 * is derived by the slice, never read (PrintApply.cpp:1505-1511, GCode.cpp:8300-8304).
 */
function expectAddedValues(gcode: string, added: Record<string, string | string[]>) {
  const lines = gcode.split("\n");
  const block = new Map(lines.slice(lines.indexOf("; CONFIG_BLOCK_START") + 1, lines.indexOf("; CONFIG_BLOCK_END"))
    .map((line) => /^; (\S+) = (.*)$/.exec(line)!).map(([, key, value]) => [key, value]));
  for (const [key, value] of Object.entries(added)) {
    const printed = block.get(key);
    if (printed === undefined || key === "filament_map_2") continue;
    if (!Array.isArray(value)) expect(printed, `reference ${key}`).toBe(value);
    else if (printed !== value.join(",") && printed !== cstyle(value))
      expect(new Set(printed.split(",")), `reference ${key} elements`).toEqual(new Set(value));
  }
}

/** glibc `%.Nf`: half to even on the exact double (toFixed rounds ties up). */
const printf = (value: number, digits: number) => new Intl.NumberFormat("en-US", {
  minimumFractionDigits: digits, maximumFractionDigits: digits, roundingMode: "halfEven", useGrouping: false,
}).format(value);
/** `format_compact_weight` (GCodeViewer.cpp:128-150), metric. */
function weight(grams: number) {
  const abs = Math.abs(grams);
  const [value, unit] = abs >= 1e6 ? [abs / 1e6, "t"] : abs >= 1000 ? [abs / 1000, "kg"] : [abs, "g"];
  return `${grams < 0 ? "-" : ""}${printf(value, 2)}${unit}`;
}

/**
 * Footer rows from the Native statistics: the filament rows by
 * GCodeViewer.cpp:3443-3447, 3581-3627 and 4605-4624 over the Native volume
 * maps (used extruders: the ids with a total volume), the time texts as the
 * probe's own `short_time(get_time_dhms(...))` (:4633, 4639, 4644).
 */
function expected(stats: Native, featureType: boolean) {
  const used = Object.keys(stats.total_volumes_per_extruder).map(Number).sort((a, b) => a - b);
  const usage = (volumes: Volumes) => {
    let [m, g] = [0, 0];
    for (const id of used) if (String(id) in volumes) {
      const volume = volumes[id];
      const radius = 0.5 * stats.filament_diameters[id];
      m += 0.001 * volume / (Math.PI * (radius * radius));
      g += volume * stats.filament_densities[id] * 0.001;
    }
    return [m, g];
  };
  const [supportM, supportG] = usage(stats.support_volumes_per_extruder);
  const [flushM, flushG] = usage(stats.flush_per_filament);
  const [towerM, towerG] = usage(stats.wipe_tower_volumes_per_extruder);
  const [excludeM, excludeG] = [supportM + flushM + towerM, supportG + flushG + towerG];
  const [prepare, model, total] = stats.footer_times;
  const rows: [string, string][] = featureType ? [
    ["Total Filament:", `${printf(stats.total_used_filament / 1000, 2)} m  ${weight(stats.total_weight)}`],
    ["Model Filament:", `${printf(stats.total_used_filament / 1000 - excludeM, 2)} m  ${weight(stats.total_weight - excludeG)}`],
    ["Cost:", printf(stats.total_cost, 2)],
  ] : [];
  if (stats.normal_prepare_time !== 0) rows.push(["Prepare time:", prepare]);
  rows.push(["Model printing time:", model], ["Total time:", total]);
  return { rows, excludeM };
}

/** `view_type_items` (GCodeViewer.cpp:1076-1093): Summary first. */
const VIEW_TYPE_ORDER = [
  "Summary", "FeatureType", "ColorPrint", "Speed", "ActualSpeed", "Acceleration", "Jerk", "Height", "Width",
  "VolumetricFlowRate", "ActualVolumetricFlowRate", "LayerTimeLinear", "LayerTimeLogarithmic", "FanSpeed",
  "Temperature", "PressureAdvance",
];

/** Summary legend rows (GCodeViewer.cpp:4003-4024), `koef` 1000 (:3559), Normal mode. */
const summaryRows = (stats: Native) => [
  ["Total:", `${printf(stats.total_used_filament / 1000, 2)} m / ${weight(stats.total_weight)}`],
  ["Cost:", printf(stats.total_cost, 2)],
  ["Total time:", stats.footer_times[2]],
];

/** Non-waiting reads: missing rows are an assertion diff, not a timeout. */
const dlRows = (page: Page, testId: string) => page.getByTestId(testId).locator("dl > div").evaluateAll((items) =>
  items.map((item) => [item.querySelector("dt")!.textContent!, item.querySelector("dd")!.textContent!]));

async function footer(page: Page) {
  const section = page.getByTestId("legend-estimation");
  const [title = null] = await section.getByRole("heading").allTextContents();
  return { title, rows: await dlRows(page, "legend-estimation") };
}

test("legend estimation footer matches GCodeViewer::render_legend over Native statistics", async ({ page }) => {
  test.setTimeout(900_000);
  const bin = oraclePath("ARES_PRINT_STATISTICS_BIN", "the PrintStatistics probe", "tools/chain-probe/print-statistics.nix");
  if (!bin) test.skip();
  if (!presetSelectProbe()) throw new Error("ARES_ORCA_PRESET_SELECT_BIN is unset while the PrintStatistics probe is set: the reference needs the GUI route");
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  const report: Record<string, unknown> = {};
  let excluded = false;
  for (const name of FIXTURES) {
    // The web opens the 3MF on the GUI route; the reference is the 3MF with the GUI's values for the keys it lacks.
    const fixture = `${root}tests/parity/summary/${name}.3mf`;
    mkdirSync(`${artifacts}/${name}`, { recursive: true });
    const added = withGuiMissingKeys(fixture, openedSelection(fixture, `${artifacts}/${name}/selection/`)!, `${artifacts}/${name}/gui-route.3mf`);
    expect(Object.keys(added), `${name} lacks a key the CLI fills from the system preset`).toContain("enable_pre_heating");
    const stats = native(bin!, name, `${artifacts}/${name}/gui-route.3mf`);
    expectAddedValues(readFileSync(`${artifacts}/${name}/plate_1.gcode`, "utf8"), added);
    // Normal mode only: the mode suffix and button (:4566-4570, 4646-4670) are deferred.
    expect(stats.stealth_time, `${name} stealth_time`).toBe(0);
    await page.getByRole("link", { name: "Prepare", exact: true }).click();
    await page.getByTestId("import-input").setInputFiles(fixture);
    await expect(page.getByTestId("status")).toHaveText("Ready to slice");
    await page.getByRole("button", { name: "Slice plate", exact: true }).click();
    await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
    await page.getByRole("link", { name: "Preview", exact: true }).click();
    await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);
    await expect(page.getByTestId("view-type")).toBeVisible();
    expect(await page.getByTestId("view-type").locator("option").evaluateAll((options) =>
      options.map((option) => (option as HTMLOptionElement).value)), "view type combo").toEqual(VIEW_TYPE_ORDER);
    const views: Record<string, unknown> = {};
    const summary = { expected: summaryRows(stats), actual: null as string[][] | null };
    for (const [view, title] of [
      ["FeatureType", "Total Estimation"], ["Speed", "Time Estimation"], ["Summary", "Time Estimation"],
    ] as const) {
      await page.getByTestId("view-type").selectOption(view);
      const want = expected(stats, view === "FeatureType");
      excluded ||= want.excludeM > 0;
      views[view] = { expected: { title, rows: want.rows } };
      await expect.poll(async () => {
        const actual = await footer(page);
        views[view] = { expected: { title, rows: want.rows }, actual };
        writeFileSync(`${artifacts}/${name}/rows.json`, `${JSON.stringify(views, null, 2)}\n`);
        return actual;
      }).toEqual({ title, rows: want.rows });
      if (view === "Summary")
        await expect.poll(async () => {
          summary.actual = await dlRows(page, "legend-summary");
          writeFileSync(`${artifacts}/${name}/summary.json`, `${JSON.stringify(summary, null, 2)}\n`);
          return summary.actual;
        }).toEqual(summary.expected);
    }
    report[name] = { native: stats, added, views, summary };
  }
  // Model Filament must differ from Total Filament on some fixture.
  expect(excluded, "a fixture with support, flushed or wipe-tower filament").toBe(true);
  await page.getByTestId("view-type").selectOption("FeatureType");
  for (const theme of ["dark", "light"] as const) {
    if (await page.locator("html").getAttribute("data-theme") !== theme)
      await page.getByTitle(theme === "dark" ? "Dark theme" : "Light theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
    await page.getByTestId("legend").screenshot({ path: `${artifacts}/legend-estimation-${theme}.png` });
  }
  await page.getByTestId("view-type").selectOption("Summary");
  for (const theme of ["light", "dark"] as const) {
    if (await page.locator("html").getAttribute("data-theme") !== theme)
      await page.getByTitle(theme === "dark" ? "Dark theme" : "Light theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
    await page.getByTestId("legend").screenshot({ path: `${artifacts}/legend-summary-${theme}.png` });
  }
  writeFileSync(`${artifacts}/result.json`, `${JSON.stringify(report, null, 2)}\n`);
});
