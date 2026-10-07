import type { FilamentLegend } from "@ares-slicer/wasm";
import { css } from "./LegendRange";
import "./filament.css";

type Usage = [m: number, g: number];

/** `format_compact_weight` (GCodeViewer.cpp:128-150), metric display. */
function weight(grams: number) {
  const abs = Math.abs(grams);
  const [value, unit] = abs >= 1e6 ? [abs / 1e6, "t"] : abs >= 1000 ? [abs / 1000, "kg"] : [abs, "g"];
  return `${grams < 0 ? "-" : ""}${value.toFixed(2)}${unit}`;
}

/**
 * ColorPrint's dense table inherits the preview's tokens and swatches. Engine
 * columns/rows are authoritative; only metric units and text are composed here.
 * GCodeViewer.cpp:3443-3448, 4044-4141; metadata loaded at :2622-2627.
 */
export default function FilamentRows({ legend }: { legend: FilamentLegend }) {
  const { columns, rows, filamentDiameters, filamentDensities } = legend;
  const summed = columns.includes("Total");
  const totals: Usage[] = columns.filter((column) => column !== "Total").map(() => [0, 0]);
  const usageRows = rows.map(({ extruderId, volumes }) => {
    const radius = 0.5 * Math.fround(filamentDiameters[extruderId]);
    const cells: Usage[] = volumes.map((volume, index) => {
      const value: Usage = [0.001 * volume / (Math.PI * radius * radius), volume * Math.fround(filamentDensities[extruderId]) * 0.001];
      totals[index][0] += value[0];
      totals[index][1] += value[1];
      return value;
    });
    // The upstream row accumulator is float, unlike the double column totals.
    if (summed) cells.push(cells.reduce<Usage>(([m, g], value) => [Math.fround(m + value[0]), Math.fround(g + value[1])], [0, 0]));
    return cells;
  });
  if (summed) totals.push(totals.reduce<Usage>(([m, g], value) => [m + value[0], g + value[1]], [0, 0]));
  const cell = ([m, g]: Usage) => `${m.toFixed(2)} m${summed ? "\n" : "    "}${weight(g)}`;
  return (
    <table className="filament-legend" data-testid="filament-legend" aria-label="Filament usage">
      <thead><tr>
        {columns.includes("Model") && <th scope="col">Filament</th>}
        {columns.map((column) => <th scope="col" key={column}>{column}</th>)}
        <th scope="col">Display</th>
      </tr></thead>
      <tbody>
        {rows.map(({ extruderId, color }, index) => <tr className="legend-filament" key={extruderId}>
          <td><span className="legend-swatch" style={{ background: css(color) }} />{extruderId + 1}</td>
          {usageRows[index].map((value, i) => <td key={columns[i]}>{cell(value)}</td>)}
        </tr>)}
        {rows.length > 1 && <tr className="legend-filament-total">
          <td>Total</td>{totals.map((value, i) => <td key={columns[i]}>{cell(value)}</td>)}
        </tr>}
      </tbody>
    </table>
  );
}
