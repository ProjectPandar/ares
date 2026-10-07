import { lazy, Suspense } from "react";
import type { ColorRange, ExtrusionRole, MoveType, Toolpaths } from "@ares-slicer/wasm";
import { EXTRUSION_ROLES, MOVE_TYPES } from "@ares-slicer/wasm";
import type { PreviewView } from "../state/store";
import { css, RangeRows, VIEW_TYPE_ITEMS } from "./LegendRange";

const FilamentRows = lazy(() => import("./LegendFilament"));

/** `ExtrusionEntity::role_to_string` (`ExtrusionEntity.cpp:583`). */
const ROLE_LABELS: Record<ExtrusionRole, string> = {
  None: "Undefined",
  Perimeter: "Inner wall",
  ExternalPerimeter: "Outer wall",
  OverhangPerimeter: "Overhang wall",
  InternalInfill: "Sparse infill",
  SolidInfill: "Internal solid infill",
  TopSolidInfill: "Top surface",
  Ironing: "Ironing",
  BridgeInfill: "Bridge",
  GapFill: "Gap infill",
  Skirt: "Skirt",
  SupportMaterial: "Support",
  SupportMaterialInterface: "Support interface",
  WipeTower: "Prime tower",
  Custom: "Custom",
  BottomSurface: "Bottom surface",
  InternalBridgeInfill: "Internal Bridge",
  Brim: "Brim",
  SupportTransition: "Support transition",
  Mixed: "Multiple",
};

/**
 * `append_option_item` rows by `EOptionType` name: label (`GCodeViewer.cpp:3834-3861`)
 * and the move type whose count it shows (`option_stats`, `GCodeViewer.cpp:3783-3791`;
 * Travel shows print statistics instead). Other options get no row.
 */
const OPTION_ROWS: Record<string, { label: string; moves?: MoveType }> = {
  Travels: { label: "Travel" },
  Wipes: { label: "Wipe", moves: "Wipe" },
  Retractions: { label: "Retract", moves: "Retract" },
  Unretractions: { label: "Unretract", moves: "Unretract" },
  Seams: { label: "Seams", moves: "Seam" },
  ToolChanges: { label: "Filament Changes", moves: "ToolChange" },
};

const TRAVEL_ROW_VIEWS = new Set(["Speed", "ActualSpeed", "Acceleration", "Jerk"]);

/** `format_compact_count` (`GCodeViewer.cpp:3166-3188`). */
function compactCount(value: number) {
  if (value < 1000) return String(value);
  const suffixes = ["", "K", "M", "B", "T", "P", "E"];
  let index = 0;
  let divisor = 1;
  while (index + 1 < suffixes.length && Math.floor(value / divisor) >= 1000) {
    divisor *= 1000;
    index++;
  }
  const tenths = Math.floor(((value % divisor) * 10) / divisor);
  return `${Math.floor(value / divisor)}${tenths ? `.${tenths}` : ""}${suffixes[index]}`;
}

/**
 * GCodeViewer legend (`GCodeViewer::render_legend`): the view type combo, then
 * in the feature type view (`GCodeViewer.cpp:3870-3911`) one row per used
 * extrusion role and one per used option in `EOptionType` order; clicking a
 * row toggles it. Option rows but Travel show the move count
 * (`GCodeViewer.cpp:3808`; Travel's comes from print statistics). Range views
 * list their color range; Speed, Acceleration and Jerk add the Travel row
 * (`GCodeViewer.cpp:3913-3929, 3947-3978`).
 */
export function Legend({ roles, options, toolpaths, range, view, onChange }: {
  roles: number[];
  options: number[];
  toolpaths: Toolpaths;
  /** Color range of `view.viewType`, once the engine has computed it. */
  range: ColorRange | null;
  view: PreviewView;
  onChange: (patch: Partial<PreviewView>) => void;
}) {
  const { optionTypes, moveTypeCounts, roleColors, optionColors, viewTypes } = toolpaths;
  const optionRow = (id: number, count: boolean) => {
    const name = optionTypes[id];
    const row = OPTION_ROWS[name];
    if (!row) return [];
    const visible = ((view.visibleOptions >> id) & 1) === 1;
    return [
      <li key={name}>
        <button className={visible ? "legend-row" : "legend-row off"} aria-pressed={visible} data-option={name}
          onClick={() => onChange({ visibleOptions: view.visibleOptions ^ (1 << id) })}>
          <span className="legend-swatch" style={{ background: css(optionColors.subarray(3 * id, 3 * id + 3)) }} />
          <span className="legend-label">{row.label}</span>
          {count && <span className="legend-count">{row.moves && compactCount(moveTypeCounts[MOVE_TYPES.indexOf(row.moves)])}</span>}
        </button>
      </li>,
    ];
  };
  const item = VIEW_TYPE_ITEMS.find(({ view: type }) => type === view.viewType);
  return (
    <div className="legend" data-testid="legend">
      <select className="legend-view" data-testid="view-type" aria-label="View type" value={view.viewType}
        onChange={(event) => onChange({ viewType: event.target.value })}>
        {VIEW_TYPE_ITEMS.filter(({ view: type }) => viewTypes.includes(type)).map(({ view: type, label }) => (
          <option key={type} value={type}>{label}</option>
        ))}
      </select>
      {view.viewType === "ColorPrint" ? (
        <Suspense fallback={<p role="status">Loading filament legend…</p>}>
          <FilamentRows legend={toolpaths.filamentLegend} />
        </Suspense>
      ) : view.viewType !== "FeatureType" ? (
        <>
          {range && item?.decimals !== undefined && <RangeRows range={range} decimals={item.decimals} />}
          {TRAVEL_ROW_VIEWS.has(view.viewType) && <ul className="legend-list">{optionRow(optionTypes.indexOf("Travels"), false)}</ul>}
        </>
      ) : (
      <ul className="legend-list">
        {roles.map((id) => {
          const hidden = (view.hiddenRoles >> id) & 1;
          return (
            <li key={id}>
              <button className={hidden ? "legend-row off" : "legend-row"} aria-pressed={!hidden}
                onClick={() => onChange({ hiddenRoles: view.hiddenRoles ^ (1 << id) })}>
                <span className="legend-swatch" style={{ background: css(roleColors.subarray(3 * id, 3 * id + 3)) }} />
                {ROLE_LABELS[EXTRUSION_ROLES[id]]}
              </button>
            </li>
          );
        })}
        {options.flatMap((id) => optionRow(id, true))}
      </ul>
      )}
    </div>
  );
}
