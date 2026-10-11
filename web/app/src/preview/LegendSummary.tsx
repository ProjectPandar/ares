import type { Estimation } from "@ares-slicer/wasm";
import { compactWeight, printf, shortTime, timeDhms } from "./toolPositionFormat";
import "./estimation.css";

/** `case EViewType::Summary` of `render_legend` (GCodeViewer.cpp:4003-4024), Normal mode, metric. */
export function SummaryRows({ estimation: { totalUsedFilament, totalWeight, totalCost, time } }: { estimation: Estimation }) {
  const rows = [
    // `koef` 1000.0 at :3559.
    ["Total", `${printf(totalUsedFilament / 1000, 2)} m / ${compactWeight(totalWeight)}`],
    ["Cost", printf(totalCost, 2)],
    ["Total time", shortTime(timeDhms(time))],
  ];
  return (
    <section className="legend-summary" data-testid="legend-summary" aria-label="Summary">
      <dl>
        {rows.map(([label, value]) => <div key={label}><dt>{label}:</dt><dd>{value}</dd></div>)}
      </dl>
    </section>
  );
}
