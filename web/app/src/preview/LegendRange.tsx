import type { ColorRange } from "@ares-slicer/wasm";

/**
 * Legend view type combo items in `view_type_items` order (`GCodeViewer.cpp:1077-1093`)
 * with their `get_view_type_string` labels (`GCodeViewer.cpp:66-101`) and the
 * decimals `render_legend` passes to `append_range` (`GCodeViewer.cpp:3911-3986`;
 * layer times pass `true`, i.e. 1), for the view types the engine's `Toolpaths.viewTypes`
 * offers; the others have no data source yet.
 */
export const VIEW_TYPE_ITEMS: { view: string; label: string; decimals?: number }[] = [
  { view: "FeatureType", label: "Line Type" },
  { view: "ColorPrint", label: "Filament" },
  { view: "Speed", label: "Speed", decimals: 0 },
  { view: "Height", label: "Layer Height", decimals: 2 },
  { view: "Width", label: "Line Width", decimals: 2 },
  { view: "VolumetricFlowRate", label: "Flow", decimals: 2 },
  { view: "LayerTimeLinear", label: "Layer Time", decimals: 1 },
  { view: "LayerTimeLogarithmic", label: "Layer Time (log)", decimals: 1 },
  { view: "FanSpeed", label: "Fan Speed", decimals: 0 },
  { view: "Temperature", label: "Temperature", decimals: 0 },
];

export const css = ([r, g, b]: Iterable<number>) => `rgb(${r} ${g} ${b})`;

/**
 * `append_range` (`GCodeViewer.cpp:3293-3313`): one row per value from the top
 * palette color down, a single value with the first color, two values with
 * the last and first colors; each value as `%.*f`.
 */
export function RangeRows({ range, decimals }: { range: ColorRange; decimals: number }) {
  const { values, palette } = range;
  const rows: [number, number][] =
    values.length === 1 ? [[0, values[0]]]
    : values.length === 2 ? [[palette.length - 1, values[1]], [0, values[0]]]
    : palette.map((_, i) => [i, values[i]] as [number, number]).reverse();
  return (
    <ul className="legend-list legend-range">
      {rows.map(([color, value], index) => (
        <li key={index} className="legend-row">
          <span className="legend-swatch" style={{ background: css(palette[color]) }} />
          {value.toFixed(decimals)}
        </li>
      ))}
    </ul>
  );
}
