import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";
import { oraclePath } from "./oracleEnv";

const root = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = `${root}target/web-e2e/custom-gcode-overview`;
const overview = `${root}tests/parity/custom-gcode-overview/overview.3mf`;

/**
 * Cell texts of the legend Custom G-code overview (GCodeViewer.cpp:4477-4546)
 * from the live Native kernel of tools/chain-probe/custom-gcode-overview.nix for
 * the browser's own G-code: the header (:4483-4485), then label, `%d` layer and
 * `short_time(get_time_dhms(t))` per item (:4517-4544). Native outputs stay
 * outside the repository.
 */
function native(gcode: string) {
  const directory = mkdtempSync(`${tmpdir()}/custom-gcode-overview-web-`);
  execFileSync(process.execPath, [`${root}tools/chain-probe/custom-gcode-overview-fixtures.mjs`, `${directory}/oracle`, overview, gcode, "0"], { cwd: root });
  const { rows } = JSON.parse(readFileSync(`${directory}/oracle/native.json`, "utf8")) as {
    rows: { label: string; layer: string; time: string }[];
  };
  return { directory, table: [["Custom G-code", "Layer", "Time"], ...rows.map(({ label, layer, time }) => [label, layer, time])] };
}

/** Non-waiting read: a missing block is an assertion diff, not a timeout. */
const table = (page: Page) => page.getByTestId("legend-custom-gcode").locator("tr").evaluateAll((rows) =>
  rows.map((row) => [...row.querySelectorAll("th, td")].map((cell) => cell.textContent)));

async function slice(page: Page, project: string) {
  await page.getByRole("link", { name: "Prepare", exact: true }).click();
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);
  await expect(page.getByTestId("view-type")).toBeVisible();
}

test("legend Custom G-code overview matches GCodeViewer::render_legend over the live Native kernel", async ({ page }) => {
  test.setTimeout(900_000);
  const bins = [
    oraclePath("ARES_PLATE_CUSTOM_GCODES_BIN", "the plate custom G-code probe", "tools/chain-probe/plate-custom-gcodes.nix"),
    oraclePath("ARES_CUSTOM_GCODE_OVERVIEW_BIN", "the custom G-code overview kernel", "tools/chain-probe/custom-gcode-overview.nix"),
  ];
  if (bins.includes(null)) test.skip();
  mkdirSync(artifacts, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await slice(page, overview);
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export G-code" }).click()]);
  const gcode = `${artifacts}/browser.gcode`;
  await download.saveAs(gcode);
  const oracle = native(gcode);
  // Every item of the fixture's plate 1 (tests/parity/custom-gcode-overview/README.md).
  expect(oracle.table, "Native header and rows").toHaveLength(10);
  const report: Record<string, unknown> = { oracle: oracle.directory, expected: oracle.table };
  for (const view of ["FeatureType", "Speed"]) {
    await page.getByTestId("view-type").selectOption(view);
    await expect.poll(async () => {
      report[view] = await table(page);
      writeFileSync(`${artifacts}/rows.json`, `${JSON.stringify(report, null, 2)}\n`);
      return report[view];
    }, { message: `${view} overview table` }).toEqual(oracle.table);
  }
  await page.getByTestId("view-type").selectOption("FeatureType");
  for (const theme of ["dark", "light"] as const) {
    if (await page.locator("html").getAttribute("data-theme") !== theme)
      await page.getByTitle(theme === "dark" ? "Dark theme" : "Light theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
    await page.getByTestId("legend").screenshot({ path: `${artifacts}/legend-custom-gcode-${theme}.png` });
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({ path: `${artifacts}/legend-custom-gcode-phone.png` });
  await page.setViewportSize({ width: 1600, height: 900 });
  // `custom_gcode_per_print_z.size() != 0` (:4481): no block without items.
  await slice(page, `${root}tests/parity/orca_cli_ender3.3mf`);
  await expect(page.getByTestId("legend-estimation")).toBeVisible();
  await expect(page.getByTestId("legend-custom-gcode")).toHaveCount(0);
});
