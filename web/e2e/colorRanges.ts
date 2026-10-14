import { expect, type Page } from "@playwright/test";
import { canvasDigest } from "./toolpathPreview";

/** The `--dump-moves` fields the libvgcode color ranges read. */
export type RangeMove = {
  type: number;
  role: number;
  layer: number;
  width: number;
  height: number;
  feedrate: number;
  mm3PerMm: number;
  fanSpeed: number;
  temperature: number;
  time: number;
  pressureAdvance: number;
  acceleration: number;
  jerk: number;
  actualFeedrate: number;
};

const EXTRUDE = 10;
const TRAVEL = 8;
/** GCodeProcessor `EMoveType::Noop` / `Travel` / `Wipe` / `Extrude`: the moves libvgcode draws as paths. */
const PATH_TYPES = new Set([0, TRAVEL, 9, EXTRUDE]);
const f32 = Math.fround;

/**
 * libvgcode `round_to_bin` (`ViewerImpl.cpp:47-63`) in float arithmetic:
 * the scale is the first one whose threshold the value reaches, and
 * `fast_round_up` is `floor(a + 0.5)` (`ViewerImpl.cpp:37-43`).
 */
function roundToBin(value: number) {
  const scale = [100, 1000, 10000, 100000, 1000000];
  const invscale = [0.01, 0.001, 0.0001, 0.00001, 0.000001].map(f32);
  const threshold = [0.095, 0.0095, 0.00095, 0.000095, 0.0000095].map(f32);
  let i = 0;
  while (value < threshold[i] && i < 4) i++;
  return f32(Math.floor(f32(value * scale[i]) + 0.5) * invscale[i]);
}

/**
 * Legend values of one `ColorRange` fed with `values` in order, top row first:
 * `update` counts values off the current bounds (`ColorRange.cpp:123-130`),
 * `get_values` gives one / two values or one per palette color with the
 * linear or logarithmic step (`ColorRange.cpp:17-33, 87-115`), and
 * `append_range` lists them from the last palette color down
 * (`GCodeViewer.cpp:3293-3313`).
 */
function legendValues(values: number[], logarithmic: boolean) {
  let [min, max, count] = [f32(3.4028234663852886e38), -f32(3.4028234663852886e38), 0];
  for (const value of values) {
    if (value !== min && value !== max) count++;
    min = Math.min(min, value);
    max = Math.max(max, value);
  }
  if (count === 1) return [min];
  if (count === 2) return [max, min];
  const colors = 11;
  const step = logarithmic
    ? min !== 0 ? f32(f32(Math.log(f32(max / min))) / (colors - 1)) : 0
    : f32(f32(max - min) / (colors - 1));
  const out = [];
  for (let i = colors - 1; i >= 0; i--) {
    out.push(logarithmic ? f32(Math.exp(Math.log(min) + f32(i * step))) : f32(min + f32(i * step)));
  }
  return out;
}

/**
 * Values each range sees, from `ViewerImpl::update_color_ranges`
 * (`ViewerImpl.cpp:1817-1878`) over the libvgcode vertices, which carry the
 * values of moves 1.. (`LibVGCodeWrapper.cpp:191-268`; the extra path-start
 * vertices repeat their move's values with zero times, except the previous
 * move's `actual_feedrate`, `:215-240`). Every extrusion role is shown, so
 * Custom extrusions count for width and flow (`:1847`).
 */
function rangeInputs(moves: RangeMove[], travels: boolean) {
  const vertices: RangeMove[] = [];
  for (let i = 1; i < moves.length; i++) {
    const [prev, curr] = [moves[i - 1], moves[i]];
    if (PATH_TYPES.has(curr.type) && (vertices.length === 0 || prev.type !== curr.type || prev.role !== curr.role || prev.mm3PerMm !== curr.mm3PerMm))
      vertices.push({ ...curr, time: 0, actualFeedrate: prev.actualFeedrate });
    vertices.push(curr);
  }
  const extrusions = vertices.filter((move) => move.type === EXTRUDE);
  const moving = vertices.filter((move) => move.type === EXTRUDE || (travels && move.type === TRAVEL));
  // `Layers::update` sums vertex times per layer (`Layers.cpp`); `get_times` (`:1870-1875`).
  const layerTimes: number[] = [];
  for (const move of vertices) layerTimes[move.layer] = f32((layerTimes[move.layer] ?? 0) + move.time);
  return {
    // `:1846`
    Height: extrusions.map((move) => roundToBin(move.height)),
    // `:1848`
    Width: extrusions.map((move) => roundToBin(move.width)),
    // `:1849`, `PathVertex::volumetric_rate` (`PathVertex.hpp:129`)
    VolumetricFlowRate: extrusions.map((move) => roundToBin(f32(move.feedrate * move.mm3PerMm))),
    // `:1850-1851`, `PathVertex::actual_volumetric_rate` (`PathVertex.hpp:133`)
    ActualVolumetricFlowRate: extrusions.map((move) => roundToBin(f32(move.actualFeedrate * move.mm3PerMm))),
    // `:1852-1853`
    FanSpeed: extrusions.map((move) => roundToBin(move.fanSpeed)),
    Temperature: extrusions.map((move) => roundToBin(move.temperature)),
    // `:1855-1856`: ORCA pressure advance of extrusions, unbinned.
    PressureAdvance: extrusions.filter((move) => move.pressureAdvance >= 0).map((move) => move.pressureAdvance),
    // `:1858-1866`: extrusions, plus travels while shown (wipes stay hidden here).
    Speed: moving.map((move) => move.feedrate),
    ActualSpeed: moving.map((move) => move.actualFeedrate),
    Acceleration: moving.map((move) => move.acceleration),
    Jerk: moving.map((move) => move.jerk),
    LayerTimeLinear: layerTimes,
    LayerTimeLogarithmic: layerTimes,
  };
}

/**
 * View types the legend offers, in `view_type_items` order
 * (`GCodeViewer.cpp:1077-1093`) with their `get_view_type_string` labels
 * (`GCodeViewer.cpp:66-101`) and `append_range` decimals
 * (`GCodeViewer.cpp:3911-3986`; layer times pass `true`, i.e. 1).
 */
const VIEWS = [
  { view: "Summary", label: "Summary" },
  { view: "FeatureType", label: "Line Type" },
  { view: "ColorPrint", label: "Filament" },
  { view: "Speed", label: "Speed", decimals: 0 },
  { view: "ActualSpeed", label: "Actual Speed", decimals: 0 },
  { view: "Acceleration", label: "Acceleration", decimals: 0 },
  { view: "Jerk", label: "Jerk", decimals: 1 },
  { view: "Height", label: "Layer Height", decimals: 2 },
  { view: "Width", label: "Line Width", decimals: 2 },
  { view: "VolumetricFlowRate", label: "Flow", decimals: 2 },
  { view: "ActualVolumetricFlowRate", label: "Actual Flow", decimals: 2 },
  { view: "LayerTimeLinear", label: "Layer Time", decimals: 1 },
  { view: "LayerTimeLogarithmic", label: "Layer Time (log)", decimals: 1 },
  { view: "FanSpeed", label: "Fan Speed", decimals: 0 },
  { view: "Temperature", label: "Temperature", decimals: 0 },
  { view: "PressureAdvance", label: "Pressure Advance", decimals: 3 },
] as const;

/** printf `%.*f`, as `append_range_item` formats the values. */
const printf = (value: number, decimals: number) => value.toFixed(decimals);

async function expectRows(page: Page, values: number[], decimals: number, view: string) {
  const rows = page.getByTestId("legend").locator(".legend-range li");
  await expect(rows, `${view} legend values`).toHaveText(values.map((value) => printf(value, decimals)));
}

/**
 * GCodeViewer legend view types: each range view lists the values of its
 * libvgcode `ColorRange`, recomputed from the upstream GCodeProcessor moves,
 * and recolors the toolpaths; Speed also offers the Travel row, whose
 * visibility feeds the speed range (`GCodeViewer.cpp:3913-3929`); Actual
 * Speed, Acceleration and Jerk do the same (`:3930-3978`).
 */
export async function checkColorRanges(page: Page, moves: RangeMove[], artifacts: string) {
  const select = page.getByTestId("view-type");
  await expect(select.locator("option")).toHaveText(VIEWS.map(({ label }) => label));
  await expect(select).toHaveValue("FeatureType");
  const featureType = await canvasDigest(page);
  const inputs = rangeInputs(moves, false);
  const expected: Record<string, number[]> = {};
  for (const entry of VIEWS) {
    if (!("decimals" in entry)) continue;
    const { view, label, decimals } = entry;
    await select.selectOption({ label });
    expected[view] = legendValues(inputs[view], view === "LayerTimeLogarithmic");
    await expectRows(page, expected[view], decimals, view);
    expect(await canvasDigest(page), `${label} must recolor the toolpaths`).not.toBe(featureType);
    if (["Height", "Speed", "ActualSpeed", "ActualVolumetricFlowRate", "LayerTimeLogarithmic", "Acceleration", "Jerk", "PressureAdvance"].includes(view)) {
      await page.screenshot({ path: `${artifacts}preview-view-${view}.png` });
    }
  }

  await select.selectOption({ label: "Speed" });
  const travel = page.getByTestId("legend").locator("[data-option=Travels]");
  await travel.click();
  await expect(travel).toHaveAttribute("aria-pressed", "true");
  const withTravels = legendValues(rangeInputs(moves, true).Speed, false);
  expect(withTravels[0], "travels are faster than extrusions").toBeGreaterThan(expected.Speed[0]);
  await expectRows(page, withTravels, 0, "Speed with travels");
  await travel.click();
  await expectRows(page, expected.Speed, 0, "Speed");
  // Actual Speed, Acceleration and Jerk offer the same Travel row (`GCodeViewer.cpp:3930-3978`).
  const travelInputs = rangeInputs(moves, true);
  for (const [view, label, decimals] of [["ActualSpeed", "Actual Speed", 0], ["Acceleration", "Acceleration", 0], ["Jerk", "Jerk", 1]] as const) {
    await select.selectOption({ label });
    await expect(travel).toHaveAttribute("aria-pressed", "false");
    await travel.click();
    await expectRows(page, legendValues(travelInputs[view], false), decimals, `${view} with travels`);
    await travel.click();
    await expectRows(page, expected[view], decimals, view);
  }

  await select.selectOption({ label: "Line Type" });
  await expect(page.getByTestId("legend").locator(".legend-range li")).toHaveCount(0);
  expect(await canvasDigest(page), "Line Type restores the feature colors").toBe(featureType);
  return Object.fromEntries(Object.entries(expected).map(([view, values]) => [view, [values.at(-1), values[0], values.length]]));
}
