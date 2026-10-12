import type { CustomGCodeOverviewRow } from "@ares-slicer/wasm";
import { shortTime, timeDhms } from "./toolPositionFormat";
import "./estimation.css";

/**
 * Custom G-code overview of `render_legend` (GCodeViewer.cpp:4477-4546): the
 * rows come from ares-vgcode; this only prints the layer (`%d`, :4528) and
 * `short_time(get_time_dhms(time))` (:4543). Absent without items (:4481).
 */
export function CustomGCodeRows({ rows }: { rows: CustomGCodeOverviewRow[] }) {
  if (rows.length === 0) return null;
  return (
    <section className="legend-custom-gcode" data-testid="legend-custom-gcode">
      <table aria-label="Custom G-code overview">
        <thead><tr><th scope="col">Custom G-code</th><th scope="col">Layer</th><th scope="col">Time</th></tr></thead>
        <tbody>
          {rows.map(({ label, layer, time }, index) => (
            <tr key={index}><td>{label}</td><td>{layer}</td><td>{shortTime(timeDhms(time))}</td></tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
