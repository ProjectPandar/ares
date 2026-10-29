import type { ProjectSettings } from "@ares-slicer/wasm";
import { extruderState, isDual, MAP_MODES, type MapMode, moveFilament, setFilamentMapMode } from "../state/extruders";
import { useApp } from "../state/store";
import { stringList } from "./options";

// FilamentGroupPopup.cpp:91-97.
const MODE_LABELS: Record<MapMode, { label: string; detail: string }> = {
  "Auto For Flush": {
    label: "Filament-Saving Mode",
    detail: "Generates filament grouping for the left and right nozzles based on the most filament-saving principles to minimize waste.",
  },
  "Auto For Match": {
    label: "Convenience Mode",
    detail: "Generates filament grouping for the left and right nozzles based on the printer's actual filament status, reducing the need for manual filament adjustment.",
  },
  Manual: { label: "Custom Mode", detail: "Manually assign filament to the left or right nozzle" },
};
const NOZZLES = [["1", "Left Nozzle", "right"], ["2", "Right Nozzle", "left"]] as const;

/**
 * Filament grouping of a dual-nozzle printer: the FilamentGroupPopup modes, Convenience Mode
 * disabled without a connected printer (FilamentGroupPopup.cpp:236-253), and in Custom Mode the
 * left and right lists of FilamentMapManualPanel (FilamentMapPanel.cpp:21-97).
 */
export function FilamentMapPanel({ settings }: { settings: ProjectSettings | null }) {
  const panel = useApp((state) => state.printerPanel);
  useApp((state) => state.presetSelection);
  useApp((state) => state.overrides);
  useApp((state) => state.scene);
  if (!isDual(panel)) return null;
  const { mode, map } = extruderState();
  const colours = stringList(settings?.filament, "filament_colour");
  const types = stringList(settings?.filament, "filament_type");
  return (
    <div className="filament-map" data-testid="filament-map">
      <div role="radiogroup" aria-label="Filament grouping" className="filament-map-modes">
        {MAP_MODES.map((value) => (
          <label key={value} className="filament-map-mode" title={MODE_LABELS[value].detail}>
            <input type="radio" name="filament-map-mode" checked={mode === value} disabled={value === "Auto For Match"}
              onChange={() => setFilamentMapMode(value)} />
            {MODE_LABELS[value].label}
            {value === "Auto For Match" && <span className="filament-map-note">(Sync with printer)</span>}
          </label>
        ))}
      </div>
      {mode === "Manual" && (
        <div className="filament-map-nozzles">
          {NOZZLES.map(([nozzle, title, other]) => (
            <div key={nozzle} role="group" aria-label={title} className="filament-map-nozzle">
              <div className="combo-group-title">{title}</div>
              {map.map((value, index) => value === nozzle && (
                <button key={index} className="filament-map-chip" title={`Move filament ${index + 1} to the ${other} nozzle`}
                  aria-label={`Move filament ${index + 1} to the ${other} nozzle`} onClick={() => moveFilament(index)}>
                  <span className="filament-swatch" style={{ background: colours[index] ?? "#888" }}>{index + 1}</span>
                  <span>{types[index]}</span>
                </button>
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
