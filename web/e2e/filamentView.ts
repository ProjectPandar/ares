import { expect, type Page } from "@playwright/test";
import { canvasDigest } from "./toolpathPreview";
import type { Move } from "./oracleMoves";

/** GCodeProcessor `EMoveType::Extrude` in `--dump-moves` CSV ids. */
const EXTRUDE = 10;
/** `ExtrusionRole` ids (`ExtrusionEntity.hpp:20-43`) `process_G1` buckets apart (`GCodeProcessor.cpp:3902-3911`). */
const SUPPORT_ROLES = new Set([14, 15, 16]);
const WIPE_TOWER = 17;
const f32 = Math.fround;

/**
 * Values of a config key in the G-code's config block, which upstream's G-code
 * preview loads (`GCodeProcessor::process_file`, `GCodeProcessor.cpp:2626`) and
 * `apply_config` reads (`:2246-2256`; floats are stored as float, `:2083`).
 */
function configValues(gcode: string, key: string) {
  const match = new RegExp(`^; ${key} = (.*)$`, "m").exec(gcode);
  if (!match) throw new Error(`G-code config lacks ${key}`);
  return match[1].split(/[;,]/).map((value) => value.trim());
}

/** `format_compact_weight` (`GCodeViewer.cpp:128-150`). */
function compactWeight(grams: number) {
  const abs = Math.abs(grams);
  const [scaled, unit] = abs >= 1e6 ? [abs / 1e6, "t"] : abs >= 1000 ? [abs / 1000, "kg"] : [abs, "g"];
  return `${grams < 0 ? "-" : ""}${scaled.toFixed(2)}${unit}`;
}

/**
 * GCodeViewer legend in the Filament (`EViewType::ColorPrint`) view, from the
 * upstream GCodeProcessor moves of the browser's G-code:
 * - used extruders: the ids of extrude vertices in id order (libvgcode
 *   `ViewerImpl.cpp:1017-1022`, `get_used_extruders_ids` `:1363-1370`);
 * - volumes: `process_G1` adds `area * dE` of each extrude move to the model,
 *   support or wipe tower cache by role (`GCodeProcessor.cpp:3832, 3898-3911`);
 *   `UsedFilaments::process_*_cache` keeps nonzero caches only (`:1558-1600`);
 * - columns, `displayed_columns` (`GCodeViewer.cpp:3563-3625`) and the
 *   headers (`:3743-3770`); each cell is `get_used_filament_from_volume`
 *   (`:3443-3448`) as `"%.2f m    %s"` with only the Model column, else
 *   `"%.2f m\n%s"`, the row Total a float sum (`:4044-4094`), and the Total row
 *   with more than one extruder (`:4101-4141`).
 */
function expectedLegend(moves: Move[], gcode: string) {
  // Actual-speed moves split processed moves (`internal_only`, `GCodeProcessor.cpp:5966-5983`);
  // arcs would add internal moves that do count (`:4698-4777`), and flush regions the Flushed
  // column (`:3170-3178, 3949-3961`): this oracle covers G-code without either.
  expect(gcode, "oracle covers G-code without arcs").not.toMatch(/^G[23] /m);
  expect(gcode, "oracle covers G-code without flush regions").not.toContain("; FLUSH_START");
  const diameters = configValues(gcode, "filament_diameter").map(Number).map(f32);
  const densities = configValues(gcode, "filament_density").map(Number).map(f32);
  const colours = configValues(gcode, "filament_colour");
  const volumes = new Map<number, [number, number, number]>();
  for (const move of moves) {
    if (move.type !== EXTRUDE || move.internalOnly) continue;
    const radius = f32(0.5 * diameters[move.extruder]);
    const volume = f32(f32(f32(Math.PI) * f32(radius * radius)) * move.deltaE);
    const bucket = SUPPORT_ROLES.has(move.role) ? 1 : move.role === WIPE_TOWER ? 2 : 0;
    const entry = volumes.get(move.extruder) ?? [0, 0, 0];
    entry[bucket] += volume;
    volumes.set(move.extruder, entry);
  }
  const used = [...volumes.keys()].sort((a, b) => a - b);
  // Bucket index of each column in display order: Model, Support, Tower.
  const columns = [["Model", 0], ["Support", 1], ["Tower", 2]] as const;
  const shown = columns.filter(([, bucket]) => used.some((id) => volumes.get(id)![bucket] !== 0));
  const total = shown.some(([name]) => name !== "Model");
  const usage = (id: number, volume: number) => [
    0.001 * volume / (Math.PI * (0.5 * diameters[id]) ** 2),
    volume * densities[id] * 0.001,
  ];
  const cell = ([m, g]: number[]) => `${m.toFixed(2)} m${total ? "\n" : "    "}${compactWeight(g)}`;
  const totals = shown.map(() => [0, 0]);
  const rows = used.map((id) => {
    const cells = shown.map(([, bucket], index) => {
      const value = usage(id, volumes.get(id)![bucket]);
      totals[index][0] += value[0];
      totals[index][1] += value[1];
      return value;
    });
    const sum = cells.reduce(([m, g], value) => [f32(m + value[0]), f32(g + value[1])], [0, 0]);
    return [String(id + 1), ...[...cells, ...(total ? [sum] : [])].map(cell)];
  });
  const grand = totals.reduce(([m, g], value) => [m + value[0], g + value[1]], [0, 0]);
  return {
    headers: [...(shown.some(([name]) => name === "Model") ? ["Filament"] : []), ...shown.map(([name]) => name), ...(total ? ["Total"] : []), "Display"],
    rows,
    total: used.length > 1 ? ["Total", ...[...totals, ...(total ? [grand] : [])].map(cell)] : null,
    colours: used.map((id) => colours[id]),
  };
}

// LibVGCodeWrapper.cpp:58-61: ORCA's minimum brightness clamp, before display.
const rgb = (hex: string) => `rgb(${[1, 3, 5].map((at) => Math.max(48, Math.trunc(f32(f32(parseInt(hex.slice(at, at + 2), 16) * f32(1 / 255)) * 255)))).join(", ")})`;

/**
 * The Filament view lists the used filaments (`GCodeViewer.cpp:4026-4141`) and
 * colors the toolpaths by filament color (`ViewerImpl::get_vertex_color`,
 * tool colors from `filament_colour`, `GUI_Preview.cpp:680-690`).
 */
export async function checkFilamentView(page: Page, moves: Move[], gcode: string, artifacts: string) {
  const expected = expectedLegend(moves, gcode);
  const select = page.getByTestId("view-type");
  const featureType = await canvasDigest(page);
  await select.selectOption({ label: "Filament" });
  const legend = page.getByTestId("filament-legend");
  await expect.poll(() => legend.locator("thead th").allTextContents()).toEqual(expected.headers);
  const rows = legend.locator("tbody tr.legend-filament");
  await expect(rows).toHaveCount(expected.rows.length);
  for (const [index, row] of expected.rows.entries()) {
    expect(await rows.nth(index).locator("td").allTextContents(), `filament row ${index}`).toEqual(row);
    await expect(rows.nth(index).locator(".legend-swatch")).toHaveCSS("background-color", rgb(expected.colours[index]));
  }
  const total = legend.locator("tbody tr.legend-filament-total");
  if (expected.total) expect(await total.locator("td").allTextContents()).toEqual(expected.total);
  else await expect(total).toHaveCount(0);
  expect(await canvasDigest(page), "Filament must recolor the toolpaths").not.toBe(featureType);
  await page.screenshot({ path: `${artifacts}preview-view-ColorPrint.png` });
  await select.selectOption({ label: "Line Type" });
  await expect(legend).toHaveCount(0);
  expect(await canvasDigest(page), "Line Type restores the feature colors").toBe(featureType);
  return expected;
}
