import type { FilamentLegend } from "@ares-slicer/wasm";
import { css } from "./LegendRange";

const labels = { Print: "Print", ColorChange: "Color change", Pause: "Pause" };
const seconds = (value: number) => `${Math.round(value)} s`;

/**
 * Extend the existing compact preview legend, not a separate settings panel.
 * Rust owns ordered segments, layer brackets and partial rows. This lazy
 * surface only labels those records and converts native mm³ into metric usage.
 */
export default function ColorPrintDetails({ legend }: { legend: FilamentLegend }) {
  const { colorPrint, filamentDiameters, filamentDensities } = legend;
  if (!colorPrint.visible) return null;
  const usage = (volume: number, extruderId: number) => {
    const index = extruderId - 1;
    const radius = 0.5 * Math.fround(filamentDiameters[index]);
    const m = 0.001 * volume / (Math.PI * radius * radius);
    const g = volume * Math.fround(filamentDensities[index]) * 0.001;
    return `${m.toFixed(2)} m · ${g.toFixed(2)}g`;
  };
  return <section className="color-print-details" data-testid="color-print-details" aria-label="ColorChange records">
    {colorPrint.extruders.map(({ extruderId, segments, ranges }) => <div key={extruderId}>
      <h3>Printed colors · Filament {extruderId + 1}</h3>
      <ol className="printed-colors">
        {segments.map(({ color, layerId, times }, index) => <li key={index}>
          <span className="legend-swatch" style={{ background: css(color) }} aria-hidden="true" />
          <span>Color {index + 1}</span><span>Layer {layerId + 1} · {seconds(times[0])} elapsed</span>
        </li>)}
      </ol>
      {ranges.length > 0 && <>
        <h3>Color-change layer brackets</h3>
        <ul className="color-change-ranges">
          {ranges.map(({ color, z }, index) => <li key={index}>
            <span className="legend-swatch" style={{ background: css(color) }} aria-hidden="true" />
            {z[0].toFixed(2)}–{z[1].toFixed(2)} mm
          </li>)}
        </ul>
      </>}
    </div>)}
    {colorPrint.partialUsage.status === "unavailable"
      ? <p className="partial-unavailable" role="status">Partial usage unavailable: native volume intervals are missing. No partial usage is inferred.</p>
      : colorPrint.partialUsage.rows.length > 0 && <table className="partial-print-records" aria-label="Partial print records">
        <caption>Partial print records</caption>
        <thead><tr><th scope="col">Event</th><th scope="col">Remaining</th><th scope="col">Duration</th><th scope="col">Model usage</th></tr></thead>
        <tbody>{colorPrint.partialUsage.rows.map((row, index) => <tr key={index}>
          <th scope="row">
            <span className="legend-swatch" style={{ background: css(row.color1) }} aria-hidden="true" />
            {row.kind === "ColorChange" && <span className="legend-swatch" style={{ background: css(row.color2) }} aria-hidden="true" />}
            {labels[row.kind]}
          </th>
          <td>{seconds(row.remaining)}</td>
          <td>{row.kind === "Print" ? seconds(row.duration) : "—"}</td>
          <td>{row.volume === null ? "—" : usage(row.volume, row.extruderId)}</td>
        </tr>)}</tbody>
      </table>}
  </section>;
}
