import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";

interface NativeCells {
  rows: { label: string; time: string; percent: string }[];
  travelTime: string;
  travelPercent: string;
}

/**
 * GCodeViewer Feature Type legend Time and % cells (`GCodeViewer.cpp:3151,
 * 3410-3418, 3638-3668`, headers `:3679-3681`) against the Native source-kernel
 * oracle (`tools/chain-probe/role-times.nix`: libvgcode `ExtrusionRoles` over the
 * real conversion) on the same G-code. Expects the line type view.
 */
export async function checkRoleTimes(page: Page, artifacts: string) {
  const out = `${artifacts}role-times/`;
  mkdirSync(out, { recursive: true });
  const oracle = process.env.ARES_ROLE_TIMES_BIN;
  if (!oracle) throw new Error("ARES_ROLE_TIMES_BIN is required for the role times oracle");
  execFileSync(oracle, ["--process-gcode", `${artifacts}oracle-input.gcode`,
    "--dump-moves", `${out}moves.csv`, "--dump-role-times", `${out}native.json`], { stdio: "ignore" });
  expect(readFileSync(`${out}moves.csv`)).toEqual(readFileSync(`${artifacts}oracle-moves.csv`));
  const { cells } = JSON.parse(readFileSync(`${out}native.json`, "utf8")) as { cells: NativeCells };
  expect(cells.rows.length, "roles of the sliced cube").toBeGreaterThanOrEqual(3);

  const legend = page.getByTestId("legend");
  await expect(legend.locator(".legend-header > *")).toHaveText(["Line Type", "Time", "%"]);
  const roles = legend.locator(".legend-list button:not([data-option])");
  await expect(roles.locator(".legend-label")).toHaveText(cells.rows.map((row) => row.label));
  await expect(roles.locator(".legend-time")).toHaveText(cells.rows.map((row) => row.time));
  await expect(roles.locator(".legend-percent")).toHaveText(cells.rows.map((row) => row.percent));
  const travel = legend.locator("[data-option=Travels]");
  await expect(travel.locator(".legend-time")).toHaveText(cells.travelTime);
  await expect(travel.locator(".legend-percent")).toHaveText(cells.travelPercent);
  await legend.screenshot({ path: `${out}legend.png` });
  return cells;
}
