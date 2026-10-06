import type { ExtrusionRole, MoveType } from "@ares-slicer/wasm";
import { EXTRUSION_ROLES, MOVE_TYPES } from "@ares-slicer/wasm";
import type { PreviewView } from "../state/store";
import { OPTION_COLORS, ROLE_COLORS } from "../viewport/roles";

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

const css = ([r, g, b]: readonly number[]) => `rgb(${r} ${g} ${b})`;

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
 * GCodeViewer legend in the feature type view (`GCodeViewer.cpp:3870-3911`):
 * one row per used extrusion role, then one per used option in `EOptionType`
 * order; clicking a row toggles it. Option rows but Travel show the move
 * count (`GCodeViewer.cpp:3808`; Travel's comes from print statistics).
 */
export function Legend({ roles, options, optionTypes, moveTypeCounts, view, onChange }: {
  roles: number[];
  options: number[];
  optionTypes: string[];
  moveTypeCounts: Uint32Array;
  view: PreviewView;
  onChange: (patch: Partial<PreviewView>) => void;
}) {
  return (
    <div className="legend" data-testid="legend">
      <div className="legend-title">Line Type</div>
      <ul className="legend-list">
        {roles.map((id) => {
          const hidden = (view.hiddenRoles >> id) & 1;
          return (
            <li key={id}>
              <button className={hidden ? "legend-row off" : "legend-row"} aria-pressed={!hidden}
                onClick={() => onChange({ hiddenRoles: view.hiddenRoles ^ (1 << id) })}>
                <span className="legend-swatch" style={{ background: css(ROLE_COLORS[id]) }} />
                {ROLE_LABELS[EXTRUSION_ROLES[id]]}
              </button>
            </li>
          );
        })}
        {options.flatMap((id) => {
          const name = optionTypes[id];
          const row = OPTION_ROWS[name];
          if (!row) return [];
          const visible = ((view.visibleOptions >> id) & 1) === 1;
          return [
            <li key={name}>
              <button className={visible ? "legend-row" : "legend-row off"} aria-pressed={visible} data-option={name}
                onClick={() => onChange({ visibleOptions: view.visibleOptions ^ (1 << id) })}>
                <span className="legend-swatch" style={{ background: css(OPTION_COLORS[name]) }} />
                <span className="legend-label">{row.label}</span>
                <span className="legend-count">{row.moves && compactCount(moveTypeCounts[MOVE_TYPES.indexOf(row.moves)])}</span>
              </button>
            </li>,
          ];
        })}
      </ul>
    </div>
  );
}
