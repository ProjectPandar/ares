import { EXTRUSION_ROLES, MOVE_TYPES, type ExtrusionRole, type MoveType, type ToolMarker, type Toolpaths, VERTEX_VALUES, type VertexValue } from "@ares-slicer/wasm";
import { useState } from "react";
import { formatMarkerTime, printf } from "./toolPositionFormat";

/** `to_string(EMoveType)` (`GCodeViewer.cpp:277-294`). */
const TYPE_LABELS: Record<MoveType, string> = {
  Noop: "Noop", Retract: "Retract", Unretract: "Unretract", Seam: "Seam", ToolChange: "Tool Change",
  ColorChange: "Color Change", PausePrint: "Pause Print", CustomGCode: "Custom G-code", Travel: "Travel",
  Wipe: "Wipe", Extrude: "Extrude",
};

/** `to_string(EGCodeExtrusionRole)` (`GCodeViewer.cpp:296-323`). */
const ROLE_LABELS: Record<ExtrusionRole, string> = {
  None: "Unknown", Perimeter: "Inner wall", ExternalPerimeter: "Outer wall", OverhangPerimeter: "Overhang wall",
  InternalInfill: "Sparse infill", SolidInfill: "Internal solid infill", TopSolidInfill: "Top surface",
  Ironing: "Ironing", BridgeInfill: "Bridge", GapFill: "Gap infill", Skirt: "Skirt", SupportMaterial: "Support",
  SupportMaterialInterface: "Support interface", WipeTower: "Prime tower", Custom: "Custom",
  BottomSurface: "Bottom surface", InternalBridgeInfill: "Internal bridge", Brim: "Brim",
  SupportTransition: "Support transition", Mixed: "Mixed",
};

const NA = "N/A";

/** The view type detail after the speed (`GCodeViewer.cpp:360-412`); empty for Speed. */
function detail(viewType: string, value: (name: VertexValue) => number, extrusion: boolean, label: string) {
  const extrusionDetail = (name: string, amount: number) => `${name}${extrusion ? printf(amount, 2) : NA}`;
  switch (viewType) {
    case "FeatureType": return label;
    case "Height": return extrusionDetail("Height: ", value("height"));
    case "Width": return extrusionDetail("Width: ", value("width"));
    case "VolumetricFlowRate": return extrusionDetail("Flow: ", Math.fround(value("feedrate") * value("mm3PerMm")));
    case "FanSpeed": return `Fan: ${printf(value("fanSpeed"), 0)}`;
    case "Temperature": return `Temperature: ${printf(value("temperature"), 0)}`;
    case "LayerTimeLinear":
    case "LayerTimeLogarithmic": return `Layer Time: ${printf(value("layerDuration"), 1)}`;
    case "Acceleration": return `Acceleration: ${printf(value("acceleration"), 0)}`;
    case "Jerk": return `Jerk: ${printf(value("jerk"), 1)}`;
    case "PressureAdvance": return `PA: ${printf(value("pressureAdvance"), 4)}`;
    default: return "";
  }
}

const AXES = [["X", "#ff3c5b"], ["Y", "#64c818"], ["Z", "#2f88e9"]] as const;

/**
 * GCodeViewer `Marker::render_position_window` (`GCodeViewer.cpp:325-690`)
 * for vertex `vertex` of `toolpaths`: the fold button shows the properties
 * table (`:420-454`), the bottom row the colored X/Y/Z with fewer decimals on big
 * prints (`:651-674`), the speed and the view type detail (`:676-683`).
 */
export function ToolPosition({ toolpaths, vertex, viewType, time }: { toolpaths: Toolpaths; vertex: number; viewType: string; time: ToolMarker["time"] }) {
  const [shown, setShown] = useState(false);
  const value = (name: VertexValue) => toolpaths.values[vertex * VERTEX_VALUES.length + VERTEX_VALUES.indexOf(name)];
  const type = MOVE_TYPES[toolpaths.tags[4 * vertex + 1]];
  const role = ROLE_LABELS[EXTRUSION_ROLES[toolpaths.tags[4 * vertex]]];
  // PathVertex::is_extrusion.
  const extrusion = type === "Extrude";
  const label = extrusion ? role : type === "Noop" ? NA : TYPE_LABELS[type];
  const position = [value("x"), value("y"), value("z")];
  const max = Math.round(Math.max(...position));
  const decimals = max > 9999 ? 1 : max > 999 ? 2 : 3;
  const text = detail(viewType, value, extrusion, label);
  const extrusionValue = (amount: number, unit: string) => (extrusion ? `${printf(amount, 3)} ${unit}` : NA);
  const rows: [string, string][] = [
    ["Type", TYPE_LABELS[type]],
    ["Line Type", extrusion ? role : NA],
    ["Width", extrusionValue(value("width"), "mm")],
    ["Height", extrusionValue(value("height"), "mm")],
    ["Layer", String(toolpaths.ids[2 * vertex + 1] + 1)],
    ["Speed", `${printf(value("feedrate"), 1)} mm/s`],
    ["Acceleration", `${printf(value("acceleration"), 0)} mm/s²`],
    ["Jerk", `${printf(value("jerk"), 1)} mm/s`],
    ["Flow rate", extrusionValue(Math.fround(value("feedrate") * value("mm3PerMm")), "mm³/s")],
    ["Fan speed", `${printf(value("fanSpeed"), 0)} %`],
    ["Temperature", `${printf(value("temperature"), 0)} °C`],
    ["Pressure Advance", printf(value("pressureAdvance"), 4)],
    ["Time", formatMarkerTime(time)],
  ];
  return (
    <div className="tool-position" data-testid="tool-position">
      {shown && (
        <table className="tool-position-table">
          <tbody>
            {rows.map(([name, cell]) => <tr key={name}><th>{name}</th><td>{cell}</td></tr>)}
          </tbody>
        </table>
      )}
      <div className="tool-position-row">
        <button className="tool-position-fold" aria-label="Properties" aria-expanded={shown} onClick={() => setShown(!shown)}>
          {shown ? "▾" : "▸"}
        </button>
        {AXES.map(([axis, color], index) => (
          <span key={axis} className="tool-position-axis">
            <span style={{ color }}>{axis} </span>
            <span className="tool-position-axis-value">{printf(position[index], decimals)}</span>
          </span>
        ))}
        <span className="tool-position-speed" data-testid="tool-position-speed">Speed: {printf(value("feedrate"), 0)}</span>
        {text && <span data-testid="tool-position-detail">{text}</span>}
      </div>
    </div>
  );
}
