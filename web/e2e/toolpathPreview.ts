import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";
import { checkColorRanges } from "./colorRanges";
import { checkFilamentView } from "./filamentView";
import { checkToolPosition, expectNoToolPosition } from "./toolPosition";
import { checkActualSpeedProfile } from "./actualSpeedProfile";
import { oracleMoves, type Move } from "./oracleMoves";
import { checkRoleTimes } from "./roleTimes";
import { checkSegmentColors } from "./segmentColors";
import { checkSegmentGeometry } from "./segmentGeometry";
import { checkSegmentSizes } from "./segmentSizes";

/** Hash of the downscaled viewport canvas, to detect view changes, once the engine's moves view is current and drawn. */
export async function canvasDigest(page: Page) {
  await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
  await expect(page.locator("[data-testid=viewport][data-drawing]")).toHaveCount(0);
  return page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => {
    const probe = document.createElement("canvas");
    probe.width = 256;
    probe.height = 144;
    const context = probe.getContext("2d")!;
    context.drawImage(canvas, 0, 0, 256, 144);
    return Array.from(context.getImageData(0, 0, 256, 144).data).join(",");
  }).then((pixels) => createHash("sha256").update(pixels).digest("hex"));
}

/** GCodeProcessor `EMoveType::Extrude` / `Seam` and `ExtrusionRole::erCustom` in `--dump-moves` CSV ids. */
const EXTRUDE = 10;
const SEAM = 3;
const CUSTOM = 18;

/**
 * Legend option rows in libvgcode `EOptionType` order (`Types.hpp`) with the
 * GCodeProcessor `EMoveType` they show (`move_type_to_option`, `Types.cpp:12-27`)
 * and their `append_option_item` label (`GCodeViewer.cpp:3834-3861`); color
 * changes, pauses and custom G-codes get no row there. All but Travel show the
 * move count (`GCodeViewer.cpp:3808`).
 */
const OPTION_ROWS = [
  { option: "Travels", type: 8, label: "Travel" },
  { option: "Wipes", type: 9, label: "Wipe" },
  { option: "Retractions", type: 1, label: "Retract" },
  { option: "Unretractions", type: 2, label: "Unretract" },
  { option: "Seams", type: 3, label: "Seams" },
  { option: "ToolChanges", type: 4, label: "Filament Changes" },
] as const;
/** Option markers (libvgcode `PathVertex::is_option`, `PathVertex.cpp:26-41`) by `EOptionType` name. */
const MARKER_TYPES: Record<string, number> = {
  Retractions: 1, Unretractions: 2, Seams: 3, ToolChanges: 4, ColorChanges: 5, PausePrints: 6, CustomGCodes: 7,
};

/** GCodeProcessor `EMoveType::Noop` / `Travel` / `Wipe`: with `Extrude`, the moves libvgcode draws as paths. */
const PATH_TYPES = new Set([0, 8, 9, EXTRUDE]);

/**
 * G-code line the G-code window marks: `GCodeViewer::render` passes
 * `get_current_vertex().gcode_id`, the last vertex of the view range. With
 * every extrusion role and the default options shown (only seams,
 * `Settings.hpp:33-42`) and `ViewerImpl::update_view_full_range`, that is the
 * last extrusion or seam up to the top layer.
 */
function currentLine(moves: Move[], top: number) {
  let id = 0;
  for (const move of moves) {
    if (move.layer > top) break;
    if (move.type === EXTRUDE || move.type === SEAM) id = move.gcodeId;
  }
  return id;
}

/**
 * libvgcode vertices and, per moves slider stop, the vertex it ends the view
 * at, with layers `[0, top]` shown, every extrusion role and only seams among
 * the options (`Settings.hpp:33-42`), and the `seq_top_layer_only` default
 * (`AppConfig.cpp:202-203`, `GCodeViewer.cpp:1128-1131`).
 */
export function movesSliderVertices(moves: Move[], top: number) {
  // libvgcode::convert, starts_path (LibVGCodeWrapper.cpp:227-235): a path move whose type, role,
  // mm3/mm, acceleration or jerk differs from the previous move gets a phantom vertex first, with its own ids.
  const vertices: Move[] = [];
  for (let i = 1; i < moves.length; i++) {
    const [prev, curr] = [moves[i - 1], moves[i]];
    const startsPath = prev.type !== curr.type || prev.role !== curr.role || prev.mm3PerMm !== curr.mm3PerMm
      || prev.acceleration !== curr.acceleration || prev.jerk !== curr.jerk;
    if (PATH_TYPES.has(curr.type) && (vertices.length === 0 || startsPath)) vertices.push(curr);
    vertices.push(curr);
  }
  // ViewerImpl::is_visible (ViewerImpl.cpp:1713-1726) with only extrusions and seams shown.
  const visible = (v: Move) => v.type === EXTRUDE || v.type === SEAM;
  // update_view_full_range (ViewerImpl.cpp:1758-1789): last vertex up to the top layer, trailing
  // hidden ones dropped (hidden travels are not extended over).
  let last = vertices.findIndex((v) => v.layer > top);
  last = (last < 0 ? vertices.length : last) - 1;
  while (!visible(vertices[last])) last--;
  // update_view_full_range (ViewerImpl.cpp:1734-1745): the first visible vertex, one step back for an extrusion.
  let first = vertices.findIndex(visible);
  if (first > 0 && vertices[first].type === EXTRUDE) first--;
  // Top layer only (ViewerImpl.cpp:1794-1803): one step before the first visible vertex of the top layer.
  let shortened = false;
  while (vertices[first].layer < top || !visible(vertices[first])) {
    first++;
    shortened = true;
  }
  if (shortened) first--;
  // GCodeViewer::update_moves_slider (GCodeViewer.cpp:1790-1807): one stop per run of equal gcode ids,
  // at its last vertex (`values.back() = i + 1`; update_sequential_view_current takes value - 1).
  const stops: number[] = [];
  for (let i = first; i <= last; i++) {
    if (i > first && vertices[i - 1].gcodeId === vertices[i].gcodeId) stops[stops.length - 1] = i;
    else stops.push(i);
  }
  return { vertices, stops };
}

/** Moves slider stops as G-code line ids. */
function movesSliderStops(moves: Move[], top: number) {
  const { vertices, stops } = movesSliderVertices(moves, top);
  return stops.map((i) => vertices[i].gcodeId);
}

/** `GCodeWindow::render`: the framed line is `id`, cut to 52 characters plus "..." past 55. */
async function expectMarkedLine(page: Page, gcode: string[], id: number) {
  const line = gcode[id - 1];
  const current = page.getByTestId("gcode-panel").locator(".gcode-line.current");
  await expect(current.locator(".gcode-ln")).toHaveText(String(id));
  await expect(current).toContainText(line.length > 55 ? `${line.slice(0, 52)}...` : line);
}

/**
 * Horizontal moves slider (GCodeViewer `m_moves_slider`): one stop per G-code
 * line of the top layer's enabled range. A stop shows the moves up to it and
 * frames its line in the G-code window (`GLCanvas3D.cpp:8263-8266`,
 * `GCodeViewer::update_sequential_view_current`); the layers below turn grey
 * (`ViewerImpl::update_colors_texture`).
 */
async function checkMovesSlider(page: Page, gcode: string[], moves: Move[], top: number, artifacts: string, latched: boolean) {
  const stops = movesSliderStops(moves, top);
  const slider = page.getByRole("slider", { name: "Moves" });
  await expect(slider).toHaveAttribute("aria-valuemax", String(stops.length - 1));
  await expect(slider).toHaveValue(String(stops.length - 1));
  await expectMarkedLine(page, gcode, stops.at(-1)!);
  const full = await canvasDigest(page);
  await expect(page.getByTestId("tool-position")).toHaveCount(latched ? 1 : 0);
  const middle = Math.floor(stops.length / 2);
  await slider.fill(String(middle));
  await expectMarkedLine(page, gcode, stops[middle]);
  expect(await canvasDigest(page), "a middle stop must hide later moves").not.toBe(full);
  await page.screenshot({ path: `${artifacts}preview-moves-middle.png` });
  await slider.fill("0");
  await expectMarkedLine(page, gcode, stops[0]);
  await slider.fill(String(stops.length - 1));
  await expectMarkedLine(page, gcode, stops.at(-1)!);
  // The marker latches once the thumb leaves the end and stays (`GCodeViewer.cpp:1594-1596`): back at the
  // end it is drawn, which changes the view only if it was not drawn at the start.
  await expect(page.getByTestId("tool-position")).toHaveCount(1);
  if (latched) expect(await canvasDigest(page)).toBe(full);
  else expect(await canvasDigest(page), "the latched tool marker draws at the end").not.toBe(full);
  return { stops: stops.length, first: stops[0], middle: stops[middle], last: stops.at(-1) };
}

/**
 * Canvas keys on the moves slider (GLCanvas3D::on_key, `GLCanvas3D.cpp:3844-3885`)
 * with the top layer thumb at `top`: Left/Right step the higher thumb, x5 with
 * Shift, clamped to the stops (IMSlider::correct_higher_value,
 * `IMSlider.cpp:1675-1683`); Home/End jump to the ends. Left at the first stop
 * moves the top layer down and the thumb to the max, Right at the last stop the
 * top layer up and the thumb to the min, which the rebuilt slider keeps
 * (`keep_min`, `GCodeViewer.cpp:1813-1820`); Up/Down set the thumb to the max
 * first (`:3857`). Orca applies every key against the thumbs the previous key
 * left, so keys pressed back to back, without waiting for the engine, add up
 * within a layer and across a layer step.
 */
async function checkMovesKeys(page: Page, gcode: string[], moves: Move[], top: number) {
  const slider = page.getByRole("slider", { name: "Moves" });
  const layer = page.getByRole("slider", { name: "Top layer" });
  const at = async (layerIndex: number, stop: number) => {
    const stops = movesSliderStops(moves, layerIndex);
    const index = stop < 0 ? stops.length + stop : stop;
    await expect(page.getByTestId("moves-slider")).toHaveAttribute("aria-busy", "false");
    await expect(layer).toHaveAttribute("aria-valuenow", String(layerIndex + 1));
    await expect(slider).toHaveAttribute("aria-valuemax", String(stops.length - 1));
    await expect(slider).toHaveValue(String(index));
    await expectMarkedLine(page, gcode, stops[index]);
  };
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const steps: [string, number, number][] = [
    ["Home", top, 0],
    ["ArrowRight", top, 1],
    ["Shift+ArrowRight", top, 6],
    ["ArrowLeft", top, 5],
    ["Shift+ArrowLeft", top, 0],
    ["ArrowLeft", top - 1, -1],
    ["ArrowLeft", top - 1, -2],
    ["Shift+ArrowRight", top - 1, -1],
    ["ArrowRight", top, 0],
    ["ArrowUp", top + 1, -1],
    ["Home", top + 1, 0],
    ["ArrowDown", top, -1],
  ];
  await at(top, -1);
  for (const [key, layerIndex, stop] of steps) {
    await page.keyboard.press(key);
    await at(layerIndex, stop);
  }
  const burst = async (keys: string[], layerIndex: number, stop: number) => {
    for (const key of keys) await page.keyboard.press(key);
    await at(layerIndex, stop);
  };
  await page.keyboard.press("Home");
  await at(top, 0);
  await burst(["ArrowRight", "ArrowRight", "ArrowRight"], top, 3);
  await burst(["Home", "ArrowLeft", "ArrowLeft"], top - 1, -2);
}

/** libvgcode `Layers::update`: a layer's z is that of its last non-custom extrusion. */
function layerZ(moves: Move[], layer: number) {
  let z = 0;
  for (const move of moves) if (move.layer === layer && move.type === EXTRUDE && move.role !== CUSTOM) z = move.z;
  return z;
}

/**
 * `GCodeWindow::render`: the marked line is the current vertex's G-code line,
 * shown as command, parameters and comment and cut to 52 characters plus
 * "..." past 55.
 */
async function expectCurrentLine(page: Page, gcode: string[], moves: Move[], top: number) {
  const id = currentLine(moves, top);
  const line = gcode[id - 1];
  expect(line, `line ${id} is a move`).toMatch(/^G1 /);
  const current = page.getByTestId("gcode-panel").locator(".gcode-line.current");
  await expect(current.locator(".gcode-ln")).toHaveText(String(id));
  await expect(current).toContainText(line.length > 55 ? `${line.slice(0, 52)}...` : line);
  const shown = await page.getByTestId("gcode-panel").locator(".gcode-text").allTextContents();
  expect(Math.max(...shown.map((text) => text.length)), "lines are cut at 55 characters").toBeLessThanOrEqual(55);
}

/**
 * Legend option rows and option markers (GCodeViewer::render_legend feature
 * type view, `GCodeViewer.cpp:3870-3911`; libvgcode option markers) against the
 * upstream move counts: each row shows its move count and the viewport draws
 * one marker per option move. Seams are shown by default (`Settings.hpp:33-42`).
 */
async function checkOptions(page: Page, moves: Move[], roles: number, artifacts: string) {
  // GCodeViewer::load_as_preview counts moves but internal split ones (`GCodeViewer.cpp:1318-1324`).
  const counts = new Map<number, number>();
  for (const move of moves) if (!move.internalOnly) counts.set(move.type, (counts.get(move.type) ?? 0) + 1);
  const rows = OPTION_ROWS.filter(({ type }) => counts.has(type));
  expect(rows.map(({ option }) => option), "options of the sliced cube").toEqual(["Travels", "Wipes", "Retractions", "Unretractions", "Seams"]);
  const legend = page.getByTestId("legend");
  const buttons = legend.getByRole("button");
  await expect(buttons).toHaveCount(roles + rows.length);
  for (const [index, { option, type, label }] of rows.entries()) {
    const row = buttons.nth(roles + index);
    await expect(row).toHaveAttribute("data-option", option);
    await expect(row.locator(".legend-label")).toHaveText(label);
    await expect(row.locator(".legend-count")).toHaveText(option === "Travels" ? "" : String(counts.get(type)));
    await expect(row).toHaveAttribute("aria-pressed", String(option === "Seams"));
  }
  const markers = JSON.parse((await page.getByTestId("viewport").getAttribute("data-option-markers"))!) as Record<string, number>;
  const expected = Object.fromEntries(Object.entries(MARKER_TYPES).flatMap(([option, type]) => (counts.has(type) ? [[option, counts.get(type)!]] : [])));
  expect(markers, "one marker per option move").toEqual(expected);

  const shown = await canvasDigest(page);
  const seams = legend.locator("[data-option=Seams]");
  await seams.click();
  await expect(seams).toHaveAttribute("aria-pressed", "false");
  const hidden = await canvasDigest(page);
  expect(hidden, "hiding seams must change the view").not.toBe(shown);
  const retract = legend.locator("[data-option=Retractions]");
  await retract.click();
  await expect(retract).toHaveAttribute("aria-pressed", "true");
  expect(await canvasDigest(page), "showing retractions must change the view").not.toBe(hidden);
  await page.screenshot({ path: `${artifacts}preview-retractions.png` });
  await retract.click();
  await seams.click();
  await expect(seams).toHaveAttribute("aria-pressed", "true");
  expect(await canvasDigest(page)).toBe(shown);
  return { counts: Object.fromEntries(rows.map(({ option, type }) => [option, counts.get(type)])), markers };
}

/**
 * Drives the preview of the engine's own toolpaths (ares-wasm `VertexBuffers`)
 * with the layer slider and legend, checked against upstream GCodeProcessor
 * moves of the same G-code.
 */
export async function checkToolpathPreview(page: Page, oracle: string, gcodePath: string, artifacts: string, errors: string[], speedTravelEnds: boolean) {
  const moves = oracleMoves(oracle, gcodePath, artifacts);
  const layers = moves.at(-1)!.layer + 1;
  const extrudeRoles = new Set(moves.filter((move) => move.type === EXTRUDE).map((move) => move.role));
  expect(layers).toBe(50);
  const gcode = readFileSync(gcodePath, "utf8").split("\n");

  await expect(page.getByTestId("layer-count")).toHaveText(String(layers));
  await expectCurrentLine(page, gcode, moves, layers - 1);
  const legend = page.getByTestId("legend");
  for (const label of ["Outer wall", "Inner wall", "Sparse infill", "Travel"]) {
    await expect(legend).toContainText(label);
  }
  const options = await checkOptions(page, moves, extrudeRoles.size, artifacts);
  const roleTimes = await checkRoleTimes(page, artifacts);
  const segmentSizes = await checkSegmentSizes(page, artifacts);
  const segmentGeometry = await checkSegmentGeometry(page, artifacts);
  const segmentColors = await checkSegmentColors(page, artifacts, speedTravelEnds);
  const top = page.getByRole("slider", { name: "Top layer" });
  await expect(top).toHaveAttribute("aria-valuenow", String(layers));
  await expect(page.getByTestId("layer-slider")).toContainText(layerZ(moves, layers - 1).toFixed(2));
  const full = await canvasDigest(page);
  await page.screenshot({ path: `${artifacts}preview-toolpaths.png` });

  expect(errors, "page errors after loading toolpaths").toEqual([]);
  await top.focus();
  await page.keyboard.press("Home");
  await expect(top).toHaveAttribute("aria-valuenow", "1");
  await expect(page.getByTestId("layer-slider")).toContainText(layerZ(moves, 0).toFixed(2));
  await expectCurrentLine(page, gcode, moves, 0);
  expect(await canvasDigest(page), "first layer only must differ from all layers").not.toBe(full);
  await expectNoToolPosition(page);
  // Fresh load: the marker is not latched yet (`GCodeViewer.cpp:1388`).
  const movesSlider = { 1: await checkMovesSlider(page, gcode, moves, 0, artifacts, false) };
  const toolPosition = await checkToolPosition(page, moves, 0, artifacts);
  const actualSpeedProfile = await checkActualSpeedProfile(page, moves, artifacts);
  await top.focus();
  await page.keyboard.press("PageUp");
  await page.keyboard.press("PageUp");
  await expect(top).toHaveAttribute("aria-valuenow", "21");
  await expectCurrentLine(page, gcode, moves, 20);
  await page.screenshot({ path: `${artifacts}preview-layer-21.png` });
  await checkMovesKeys(page, gcode, moves, 20);
  await top.focus();
  await page.keyboard.press("End");
  await expect(top).toHaveAttribute("aria-valuenow", String(layers));
  // GCodeViewer::render (`GCodeViewer.cpp:1594-1601`): the marker stays shown once the moves thumb left the end.
  // The top layer commit sets `aria-busy` until the latched-marker frame is drawn, which `canvasDigest` waits on.
  const marked = await canvasDigest(page);
  expect(marked, "the latched tool marker draws at the last move").not.toBe(full);
  Object.assign(movesSlider, { [layers]: await checkMovesSlider(page, gcode, moves, layers - 1, artifacts, true) });

  const outerWall = legend.getByRole("button", { name: "Outer wall" });
  await outerWall.click();
  await expect(outerWall).toHaveAttribute("aria-pressed", "false");
  expect(await canvasDigest(page), "hiding outer walls must change the view").not.toBe(full);
  await page.screenshot({ path: `${artifacts}preview-no-outer-wall.png` });
  await outerWall.click();
  await expect(outerWall).toHaveAttribute("aria-pressed", "true");
  expect(await canvasDigest(page)).toBe(marked);
  const colorRanges = await checkColorRanges(page, moves, artifacts);
  const filamentView = await checkFilamentView(page, moves, gcode.join("\n"), artifacts);

  writeFileSync(
    `${artifacts}toolpaths.json`,
    `${JSON.stringify({
      moves: moves.length, layers, movesSlider, toolPosition, actualSpeedProfile, roles: [...extrudeRoles].sort((a, b) => a - b), roleTimes, segmentSizes, segmentGeometry, segmentColors, options, colorRanges, filamentView,
      layerZs: { 1: layerZ(moves, 0), [layers]: layerZ(moves, layers - 1) },
      currentLines: { 1: currentLine(moves, 0), 21: currentLine(moves, 20), [layers]: currentLine(moves, layers - 1) },
    }, null, 2)}\n`,
  );
}
