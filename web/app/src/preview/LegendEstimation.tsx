import type { Estimation, FilamentLegend } from "@ares-slicer/wasm";
import { compactWeight, printf, shortTime, timeDhms } from "./toolPositionFormat";
import "./estimation.css";

const EXCLUDED = ["Support", "Flushed", "Tower"] as const;

/**
 * `exlude_m`/`exlude_g` (GCodeViewer.cpp:4613-4614): the support (:3613-3627),
 * flushed (:3597-3611) and wipe-tower (:3581-3595) totals, each a double sum
 * over the used extruders of `get_used_filament_from_volume` (:3443-3447).
 */
function excluded({ columns, rows, filamentDiameters, filamentDensities }: FilamentLegend) {
  let [m, g] = [0, 0];
  for (const column of EXCLUDED) {
    const index = columns.indexOf(column);
    let [columnM, columnG] = [0, 0];
    if (index >= 0) for (const { extruderId, volumes } of rows) {
      const radius = 0.5 * Math.fround(filamentDiameters[extruderId]);
      columnM += 0.001 * volumes[index] / (Math.PI * (radius * radius));
      columnG += volumes[index] * Math.fround(filamentDensities[extruderId]) * 0.001;
    }
    m += columnM;
    g += columnG;
  }
  return [m, g];
}

/**
 * Estimation footer of `render_legend` (GCodeViewer.cpp:4547-4644), Normal
 * mode, metric units: filament and cost rows in the feature type view
 * (:4596-4626), then the time rows (:4627-4644).
 */
export function EstimationRows({ estimation, legend, featureType }: {
  estimation: Estimation;
  legend: FilamentLegend;
  featureType: boolean;
}) {
  const { totalUsedFilament, totalWeight, totalCost, time, prepareTime } = estimation;
  const rows: [string, string][] = [];
  if (featureType) {
    const [m, g] = excluded(legend);
    rows.push(
      // :4605-4608, `koef` 1000.0 at :3559.
      ["Total Filament", `${printf(totalUsedFilament / 1000, 2)} m  ${compactWeight(totalWeight)}`],
      // :4615-4618: total minus the support, flushed and wipe-tower usage.
      ["Model Filament", `${printf(totalUsedFilament / 1000 - m, 2)} m  ${compactWeight(totalWeight - g)}`],
      // :4624.
      ["Cost", printf(totalCost, 2)],
    );
  }
  if (prepareTime !== 0) rows.push(["Prepare time", shortTime(timeDhms(prepareTime))]);
  rows.push(
    ["Model printing time", shortTime(timeDhms(Math.fround(time - prepareTime)))],
    ["Total time", shortTime(timeDhms(time))],
  );
  return (
    <section className="legend-estimation" data-testid="legend-estimation" aria-labelledby="legend-estimation-title">
      <h3 id="legend-estimation-title">{featureType ? "Total Estimation" : "Time Estimation"}</h3>
      <dl>
        {rows.map(([label, value]) => <div key={label}><dt>{label}:</dt><dd>{value}</dd></div>)}
      </dl>
    </section>
  );
}
