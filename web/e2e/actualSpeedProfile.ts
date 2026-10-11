import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";
import { movesSliderVertices, type Move } from "./toolpathPreview";

const SEAM = 3;
const EXTRUDE = 10;

/** printf `%.*f`: round half to even on the exact value of the float. */
const printf = (value: number, decimals: number) =>
  new Intl.NumberFormat("en-US", {
    minimumFractionDigits: decimals, maximumFractionDigits: decimals, roundingMode: "halfEven", useGrouping: false,
  }).format(value);
const float = (bits: number) => new Float32Array(new Uint32Array([bits]).buffer)[0];

interface NativeProfile {
  id: number;
  updated: boolean;
  yRangeBits?: [number, number];
  levels?: number[][];
  data?: [number, number, boolean][];
}

/**
 * GCodeViewer actual speed profile (`GCodeViewer.cpp:1681-1753` data, `:518-614`
 * window, `:152-230` plot) against the Native source-kernel oracle
 * (`tools/chain-probe/actual-speed-profile.nix`) on the same G-code, travels
 * and wipes hidden as the preview starts. Expects the line type view at the
 * last moves stop of the first layer with the properties folded.
 */
export async function checkActualSpeedProfile(page: Page, moves: Move[], artifacts: string) {
  const out = `${artifacts}actual-speed-profile/`;
  mkdirSync(out, { recursive: true });
  const oracle = process.env.ARES_ACTUAL_SPEED_PROFILE_BIN;
  if (!oracle) throw new Error("ARES_ACTUAL_SPEED_PROFILE_BIN is required for the actual speed profile oracle");
  execFileSync(oracle, ["--process-gcode", `${artifacts}oracle-input.gcode`,
    "--dump-moves", `${out}moves.csv`, "--dump-actual-speed-profile", `${out}native.json`], { stdio: "ignore" });
  expect(readFileSync(`${out}moves.csv`)).toEqual(readFileSync(`${artifacts}oracle-moves.csv`));
  const native = JSON.parse(readFileSync(`${out}native.json`, "utf8")) as {
    vertices: { id: number; gcodeId: number }[];
    hidden: NativeProfile[];
  };
  const { vertices, stops } = movesSliderVertices(moves, 0);
  expect(native.vertices.map((v) => v.gcodeId)).toEqual(vertices.map((v) => v.gcodeId));
  const rows = (profile: NativeProfile) =>
    profile.data!.map(([pos, speed, internal]) => ({ pos: printf(float(pos), 3), speed: printf(float(speed), 1), internal }));

  // An extrusion stop whose line has at least two segments, preferring one with an internal point.
  const candidates = stops.map((id, stop) => ({ stop, id, profile: native.hidden[id] }))
    .filter(({ id, profile }) => vertices[id].type === EXTRUDE && profile.updated && profile.data!.length >= 3);
  const chosen = candidates.find(({ profile }) => profile.data!.some(([, , internal]) => internal)) ?? candidates[0];
  expect(chosen, "an extrusion stop with a multi-point profile").toBeDefined();
  const expected = rows(chosen.profile);

  const slider = page.getByRole("slider", { name: "Moves" });
  const position = page.getByTestId("tool-position");
  const fold = position.getByRole("button", { name: "Properties" });
  await slider.fill(String(chosen.stop));
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  const toggle = position.getByRole("button", { name: /^(Show|Hide)$/ });
  await expect(toggle).toHaveCount(0);
  await fold.click();
  await expect(position.getByText("Actual speed profile", { exact: true })).toBeVisible();
  await expect(toggle).toHaveText("Show");
  await expect(toggle).toBeEnabled();
  const window = page.getByTestId("actual-speed-profile");
  await expect(window).toHaveCount(0);
  await toggle.click();
  await expect(toggle).toHaveText("Hide");
  await expect(window.locator("thead th")).toHaveText(["Position (mm)", "Speed (mm/s)"]);
  const body = window.locator("tbody tr");
  await expect(body).toHaveCount(expected.length);
  await expect(body.locator("td:nth-child(1)")).toHaveText(expected.map((row) => row.pos));
  await expect(body.locator("td:nth-child(2)")).toHaveText(expected.map((row) => row.speed));
  for (const [index, row] of expected.entries()) {
    if (row.internal) await expect(body.nth(index)).toHaveClass(/\bis-internal\b/);
    else await expect(body.nth(index)).not.toHaveClass(/\bis-internal\b/);
  }
  const plot = window.getByTestId("actual-speed-plot");
  await expect(plot.locator(".actual-speed-level")).toHaveCount(chosen.profile.levels!.length);
  await expect(plot.locator(".actual-speed-segment")).toHaveCount(expected.length - 1);

  // Plot hover (`:206-213`): segment n under t = (mx - min - 10) / (w - 10), rows n and n + 1 highlighted.
  const data = chosen.profile.data!.map(([pos]) => float(pos));
  const n = Math.floor((data.length - 1) / 2);
  const x0 = data[0];
  const scale = 1 / (data.at(-1)! - x0);
  const t = ((data[n] - x0) * scale + (data[n + 1] - x0) * scale) / 2;
  const box = (await plot.boundingBox())!;
  await page.mouse.move(box.x + 10 + t * (box.width - 10), box.y + box.height / 2);
  await expect(plot.locator(".actual-speed-segment.is-hovered")).toHaveCount(1);
  await expect(window.locator("tbody tr.is-hovered")).toHaveCount(2);
  await expect(body.nth(n)).toHaveClass(/\bis-hovered\b/);
  await expect(body.nth(n + 1)).toHaveClass(/\bis-hovered\b/);

  // Desktop: left of the position window, bottoms aligned, inside the canvas.
  const windowBox = (await window.boundingBox())!;
  const positionBox = (await position.boundingBox())!;
  expect(windowBox.x + windowBox.width).toBeLessThanOrEqual(positionBox.x);
  expect(Math.abs(windowBox.y + windowBox.height - (positionBox.y + positionBox.height))).toBeLessThanOrEqual(1);
  expect(windowBox.y).toBeGreaterThanOrEqual(0);
  const initialTheme = await page.locator("html").getAttribute("data-theme");
  for (const theme of ["dark", "light"] as const) {
    if (await page.locator("html").getAttribute("data-theme") !== theme)
      await page.getByTitle(theme === "dark" ? "Dark theme" : "Light theme").click();
    await page.screenshot({ path: `${out}plot-${theme}.png` });
  }
  if (initialTheme === "dark") await page.getByTitle("Dark theme").click();
  await page.mouse.move(0, 0);
  await expect(window.locator("tbody tr.is-hovered")).toHaveCount(0);

  // Phone: stacked above the position window, inside the viewport width.
  const size = page.viewportSize()!;
  await page.setViewportSize({ width: 390, height: 844 });
  const phoneWindow = (await window.boundingBox())!;
  const phonePosition = (await position.boundingBox())!;
  expect(phoneWindow.y + phoneWindow.height).toBeLessThanOrEqual(phonePosition.y);
  expect(phoneWindow.x).toBeGreaterThanOrEqual(0);
  expect(phoneWindow.x + phoneWindow.width).toBeLessThanOrEqual(390);
  await page.screenshot({ path: `${out}phone.png` });
  await page.setViewportSize(size);

  // A seam window vertex has no actual speed: disabled and hidden, still "Hide" (`:518-532`).
  const seamStop = stops.findIndex((id) => vertices[id].type === SEAM);
  await slider.fill(String(seamStop));
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  await expect(toggle).toBeDisabled();
  await expect(toggle).toHaveText("Hide");
  await expect(window).toHaveCount(0);

  // Back at the chosen stop, then at the last stop: the visible range ends at
  // the enabled range end, so the profile is not updated and stays (`:1682`).
  await slider.fill(String(chosen.stop));
  await expect(body.locator("td:nth-child(1)")).toHaveText(expected.map((row) => row.pos));
  const last = stops.length - 1;
  expect(vertices[stops[last]].type).toBe(EXTRUDE);
  await slider.fill(String(last));
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  await expect(toggle).toBeEnabled();
  await expect(body.locator("td:nth-child(1)")).toHaveText(expected.map((row) => row.pos));
  await expect(body.locator("td:nth-child(2)")).toHaveText(expected.map((row) => row.speed));

  await toggle.click();
  await expect(window).toHaveCount(0);
  await fold.click();
  await expect(toggle).toHaveCount(0);
  const result = { stop: chosen.stop, vertex: chosen.id, gcodeId: vertices[chosen.id].gcodeId, seamStop, rows: expected };
  writeFileSync(`${out}profile.json`, `${JSON.stringify(result, null, 2)}\n`);
  return result;
}
