import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";
import { movesSliderVertices, type Move } from "./toolpathPreview";

/** GCodeProcessor `EMoveType` by `--dump-moves` id with its `to_string` label (`GCodeViewer.cpp:277-294`). */
const TYPE_LABELS = [
  "Noop", "Retract", "Unretract", "Seam", "Tool Change", "Color Change", "Pause Print", "Custom G-code", "Travel", "Wipe", "Extrude",
];
/**
 * GCodeProcessor `ExtrusionRole` by `--dump-moves` id (`ExtrusionEntity.hpp:20-42`)
 * with the `to_string(EGCodeExtrusionRole)` label of the role it converts to (`GCodeViewer.cpp:296-323`).
 */
const ROLE_LABELS = [
  "Unknown", "Inner wall", "Outer wall", "Overhang wall", "Sparse infill", "Internal solid infill", "Top surface",
  "Bottom surface", "Ironing", "Bridge", "Internal bridge", "Gap infill", "Skirt", "Brim", "Support",
  "Support interface", "Support transition", "Prime tower", "Custom", "Mixed",
];
const SEAM = 3;
const EXTRUDE = 10;
const f32 = Math.fround;

/** printf `%.*f`: round half to even on the exact value of the float. */
const printf = (value: number, decimals: number) =>
  new Intl.NumberFormat("en-US", {
    minimumFractionDigits: decimals, maximumFractionDigits: decimals, roundingMode: "halfEven", useGrouping: false,
  }).format(value);

/** `render_position_window` X/Y/Z: fewer decimals on big prints (`GCodeViewer.cpp:651-653`). */
function axes(move: Move) {
  const position = [move.x, move.y, move.z].map(f32);
  const max = Math.round(Math.max(...position));
  const decimals = max > 9999 ? 1 : max > 999 ? 2 : 3;
  return position.map((value) => printf(value, decimals));
}

/** The view type detail next to the speed (`GCodeViewer.cpp:358-412`) for the views it checks. */
function detail(move: Move, view: "FeatureType" | "Height") {
  const extrusion = move.type === EXTRUDE;
  if (view === "Height") return `Height: ${extrusion ? printf(f32(move.height), 2) : "N/A"}`;
  if (extrusion) return ROLE_LABELS[move.role];
  return move.type === 0 ? "N/A" : TYPE_LABELS[move.type];
}

/**
 * The properties table rows backed by engine data (`GCodeViewer.cpp:430-454`);
 * Acceleration, Jerk and Pressure Advance remain deferred.
 */
function properties(move: Move): [string, string][] {
  const extrusion = move.type === EXTRUDE;
  const na = (value: string) => (extrusion ? value : "N/A");
  // PathVertex::volumetric_rate (`PathVertex.hpp:129`): feedrate * mm3_per_mm in float.
  const flow = f32(f32(move.feedrate) * f32(move.mm3PerMm));
  return [
    ["Type", TYPE_LABELS[move.type]],
    ["Line Type", na(ROLE_LABELS[move.role])],
    ["Width", na(`${printf(f32(move.width), 3)} mm`)],
    ["Height", na(`${printf(f32(move.height), 3)} mm`)],
    ["Layer", String(move.layer + 1)],
    ["Speed", `${printf(f32(move.feedrate), 1)} mm/s`],
    ["Flow rate", na(`${printf(flow, 3)} mm³/s`)],
    ["Fan speed", `${printf(f32(move.fanSpeed), 0)} %`],
    ["Temperature", `${printf(f32(move.temperature), 0)} °C`],
  ];
}

/** Right after a load the marker and its window are hidden (`m_show_marker = false`, `GCodeViewer.cpp:1388`). */
export async function expectNoToolPosition(page: Page) {
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  await expect(page.getByTestId("tool-position")).toHaveCount(0);
}

async function expectToolPosition(page: Page, move: Move, view: "FeatureType" | "Height") {
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  const window = page.getByTestId("tool-position");
  await expect(window.locator(".tool-position-axis-value")).toHaveText(axes(move));
  await expect(window.getByTestId("tool-position-speed")).toHaveText(`Speed: ${printf(f32(move.feedrate), 0)}`);
  await expect(window.getByTestId("tool-position-detail")).toHaveText(detail(move, view));
}

/**
 * GCodeViewer tool position marker and window (`GCodeViewer.cpp:1594-1601`,
 * `SequentialView::render` `:957-963`, `Marker::render_position_window`
 * `:325-690`), against upstream GCodeProcessor moves: once the moves thumb
 * left the end, the window stays and describes the current vertex; outside the
 * line type view a seam shows the vertex before it (`:345-352`); the fold
 * button shows the properties table.
 */
export async function checkToolPosition(page: Page, moves: Move[], top: number, artifacts: string) {
  const { vertices, stops } = movesSliderVertices(moves, top);
  const oracle = process.env.ARES_MARKER_TIME_BIN;
  if (!oracle) throw new Error("ARES_MARKER_TIME_BIN is required for the marker Time oracle");
  execFileSync(oracle, ["--process-gcode", `${artifacts}oracle-input.gcode`,
    "--dump-moves", `${artifacts}marker-time-moves.csv`,
    "--dump-marker-time", `${artifacts}marker-time-native.json`], { stdio: "ignore" });
  expect(readFileSync(`${artifacts}marker-time-moves.csv`)).toEqual(readFileSync(`${artifacts}oracle-moves.csv`));
  const native = JSON.parse(readFileSync(`${artifacts}marker-time-native.json`, "utf8")) as {
    vertices: { id: number; gcodeId: number; layerId: number; type: number; role: number; text: [string, string] }[];
  };
  expect(native.vertices).toHaveLength(vertices.length);
  const time = (id: number) => {
    expect(native.vertices[id].id).toBe(id);
    expect(native.vertices[id].gcodeId).toBe(vertices[id].gcodeId);
    return native.vertices[id].text[0];
  };
  const slider = page.getByRole("slider", { name: "Moves" });
  await expect(slider).toHaveValue(String(stops.length - 1));
  await expectToolPosition(page, vertices[stops.at(-1)!], "FeatureType");

  const seamStop = stops.findIndex((i) => vertices[i].type === SEAM);
  expect(seamStop, "a stop ends at a seam").toBeGreaterThan(0);
  const seam = vertices[stops[seamStop]];
  const before = vertices[stops[seamStop] - 1];
  expect(before.feedrate, "the seam and the move before it differ").not.toBe(seam.feedrate);
  await slider.fill(String(seamStop));
  await expectToolPosition(page, seam, "FeatureType");
  await page.screenshot({ path: `${artifacts}preview-tool-marker.png` });

  const select = page.getByTestId("view-type");
  await select.selectOption({ label: "Layer Height" });
  await expectToolPosition(page, before, "Height");
  const window = page.getByTestId("tool-position");
  const fold = window.getByRole("button", { name: "Properties" });
  await expect(fold).toHaveAttribute("aria-expanded", "false");
  await expect(window.locator("table")).toHaveCount(0);
  await fold.click();
  await expect(fold).toHaveAttribute("aria-expanded", "true");
  const rows = [...properties(before), ["Time", time(stops[seamStop] - 1)]];
  await page.screenshot({ path: `${artifacts}marker-time-before-assert.png` });
  await expect(window.locator("table th")).toHaveText(rows.map(([label]) => label));
  await expect(window.locator("table td")).toHaveText(rows.map(([, value]) => value));
  await page.screenshot({ path: `${artifacts}preview-tool-properties.png` });
  const box = await window.boundingBox();
  const sliderBox = await page.getByTestId("moves-slider").boundingBox();
  expect(box!.y + box!.height).toBeLessThanOrEqual(sliderBox!.y);
  await select.selectOption({ label: "Line Type" });
  const timeCell = window.locator("tr").filter({ has: page.getByText("Time", { exact: true }) }).locator("td");
  await expect(timeCell).toHaveText(time(stops[seamStop]));
  for (const stop of [0, Math.floor(stops.length / 2), stops.length - 1, 0]) {
    await slider.fill(String(stop));
    await expect(timeCell).toHaveText(time(stops[stop]));
  }
  const outer = page.getByTestId("legend").getByRole("button", { name: "Outer wall" });
  await outer.click();
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  await slider.fill((await slider.getAttribute("max"))!);
  const filteredLast = native.vertices.findLast((v) => v.layerId <= top && (v.type === SEAM || (v.type === EXTRUDE && v.role !== 2)))!;
  await expect(timeCell).toHaveText(time(filteredLast.id));
  await outer.click();
  await slider.fill(String(stops.length - 1));
  await expect(timeCell).toHaveText(time(stops.at(-1)!));
  const initialTheme = await page.locator("html").getAttribute("data-theme");
  for (const theme of ["light", "dark"] as const) {
    if (await page.locator("html").getAttribute("data-theme") !== theme)
      await page.getByTitle(theme === "dark" ? "Dark theme" : "Light theme").click();
    await expect(timeCell).toHaveText(time(stops.at(-1)!));
    await page.screenshot({ path: `${artifacts}marker-time-${theme}.png` });
  }
  if (initialTheme === "light") await page.getByTitle("Light theme").click();
  await fold.click();
  await expect(window.locator("table")).toHaveCount(0);
  await select.selectOption({ label: "Line Type" });
  await slider.fill(String(stops.length - 1));
  await expectToolPosition(page, vertices[stops.at(-1)!], "FeatureType");
  return { seam: seam.gcodeId, stops: stops.length, seamStop };
}
