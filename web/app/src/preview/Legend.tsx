import type { ExtrusionRole } from "@ares-slicer/wasm";
import { EXTRUSION_ROLES } from "@ares-slicer/wasm";
import type { PreviewView } from "../state/store";
import { ROLE_COLORS, TRAVEL_COLOR, WIPE_COLOR } from "../viewport/toolpathMesh";

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

const css = ([r, g, b]: readonly number[]) => `rgb(${r} ${g} ${b})`;

/** GCodeViewer legend in "Line Type" view: click a row to hide that feature. */
export function Legend({ roles, view, onChange }: {
  roles: number[];
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
      </ul>
      <div className="legend-title">Options</div>
      <ul className="legend-list">
        {([["travel", "Travel", TRAVEL_COLOR], ["wipe", "Wipe", WIPE_COLOR]] as const).map(([key, label, color]) => (
          <li key={key}>
            <button className={view[key] ? "legend-row" : "legend-row off"} aria-pressed={view[key]}
              onClick={() => onChange({ [key]: !view[key] })}>
              <span className="legend-swatch" style={{ background: css(color) }} />
              {label}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
