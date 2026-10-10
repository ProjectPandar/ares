import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";
import { oraclePath } from "./oracleEnv";

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
 * `plate_1.print_statistics.json` of tools/chain-probe/print-statistics.nix,
 * pinned to one CPU with the seeded reference's seed like
 * `OrcaRunner::print_statistics_reference`. Written to target/, never the repo.
 */
function native(bin: string, name: string): Native {
  const directory = `${artifacts}/${name}`;
  mkdirSync(directory, { recursive: true });
  const run = spawnSync("taskset", ["-c", "0", bin, "--slice", "0", "--outputdir", directory, `${root}tests/parity/summary/${name}.3mf`], {
    env: { ...process.env, ORCA_REPLAY_SEED: "0" }, encoding: "utf8", timeout: 290_000, killSignal: "SIGKILL",
  });
  writeFileSync(`${directory}/native.log`, `${run.stdout}\n${run.stderr}`);
  if (run.error) throw run.error;
  expect(run.status, `native slice of ${name}`).toBe(0);
  return JSON.parse(readFileSync(`${directory}/plate_1.print_statistics.json`, "utf8")) as Native;
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

/** Non-waiting reads: a missing footer is an assertion diff, not a timeout. */
async function footer(page: Page) {
  const section = page.getByTestId("legend-estimation");
  const [title = null] = await section.getByRole("heading").allTextContents();
  const rows = await section.locator("dl > div").evaluateAll((items) =>
    items.map((item) => [item.querySelector("dt")!.textContent!, item.querySelector("dd")!.textContent!]));
  return { title, rows };
}

test("legend estimation footer matches GCodeViewer::render_legend over Native statistics", async ({ page }) => {
  test.setTimeout(900_000);
  const bin = oraclePath("ARES_PRINT_STATISTICS_BIN", "the PrintStatistics probe", "tools/chain-probe/print-statistics.nix");
  if (!bin) test.skip();
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  const report: Record<string, unknown> = {};
  let excluded = false;
  for (const name of FIXTURES) {
    const stats = native(bin!, name);
    // Normal mode only: the mode suffix and button (:4566-4570, 4646-4670) are deferred.
    expect(stats.stealth_time, `${name} stealth_time`).toBe(0);
    await page.getByRole("link", { name: "Prepare", exact: true }).click();
    await page.getByTestId("import-input").setInputFiles(`${root}tests/parity/summary/${name}.3mf`);
    await expect(page.getByTestId("status")).toHaveText("Ready to slice");
    await page.getByRole("button", { name: "Slice plate", exact: true }).click();
    await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
    await page.getByRole("link", { name: "Preview", exact: true }).click();
    await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);
    const views: Record<string, unknown> = {};
    for (const [view, title] of [["FeatureType", "Total Estimation"], ["Speed", "Time Estimation"]] as const) {
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
    }
    report[name] = { native: stats, views };
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
  writeFileSync(`${artifacts}/result.json`, `${JSON.stringify(report, null, 2)}\n`);
});
