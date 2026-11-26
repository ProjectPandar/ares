import { expect, type Page } from "@playwright/test";
import type { Move } from "./oracleMoves";

/** GCodeProcessor `EMoveType::Extrude` / `Seam` in `--dump-moves` CSV ids. */
const EXTRUDE = 10;
const SEAM = 3;
/** GCodeProcessor `EMoveType::Noop` / `Travel` / `Wipe`: with `Extrude`, the moves libvgcode draws as paths. */
const PATH_TYPES = new Set([0, 8, 9, EXTRUDE]);

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
    const startsPath =
      prev.type !== curr.type ||
      prev.role !== curr.role ||
      prev.mm3PerMm !== curr.mm3PerMm ||
      prev.acceleration !== curr.acceleration ||
      prev.jerk !== curr.jerk;
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
export async function checkMovesSlider(
  page: Page,
  gcode: string[],
  moves: Move[],
  top: number,
  artifacts: string,
  latched: boolean,
  digest: (page: Page) => Promise<string>,
) {
  const stops = movesSliderStops(moves, top);
  const slider = page.getByRole("slider", { name: "Moves" });
  await expect(slider).toHaveAttribute("aria-valuemax", String(stops.length - 1));
  await expect(slider).toHaveValue(String(stops.length - 1));
  await expectMarkedLine(page, gcode, stops.at(-1)!);
  const full = await digest(page);
  await expect(page.getByTestId("tool-position")).toHaveCount(latched ? 1 : 0);
  const middle = Math.floor(stops.length / 2);
  await slider.fill(String(middle));
  await expectMarkedLine(page, gcode, stops[middle]);
  expect(await digest(page), "a middle stop must hide later moves").not.toBe(full);
  await page.screenshot({ path: `${artifacts}preview-moves-middle.png` });
  await slider.fill("0");
  await expectMarkedLine(page, gcode, stops[0]);
  await slider.fill(String(stops.length - 1));
  await expectMarkedLine(page, gcode, stops.at(-1)!);
  // The marker latches once the thumb leaves the end and stays (`GCodeViewer.cpp:1594-1596`): back at the
  // end it is drawn, which changes the view only if it was not drawn at the start.
  await expect(page.getByTestId("tool-position")).toHaveCount(1);
  if (latched) expect(await digest(page)).toBe(full);
  else expect(await digest(page), "the latched tool marker draws at the end").not.toBe(full);
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
export async function checkMovesKeys(page: Page, gcode: string[], moves: Move[], top: number) {
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
