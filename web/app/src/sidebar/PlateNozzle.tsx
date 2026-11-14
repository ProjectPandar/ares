import type { ProjectSettings } from "@ares-slicer/wasm";
import { selectBedType, selectDiameter } from "../state/printer";
import { accordBedType, bedTypeOptions, nozzleOptions, showsBedTypes } from "../state/printerPanel";
import { useApp } from "../state/store";
import { firstString } from "./options";

/**
 * The plate type and machine variant selectors of the plater printer panel
 * (Plater.cpp:1120-1140, 3015-3048). Bambu dual printers use their left/right controls.
 */
export function PlateNozzle({ settings }: { settings: ProjectSettings | null }) {
  const panel = useApp((state) => state.printerPanel);
  const selection = useApp((state) => state.presetSelection);
  const edited = useApp((state) => state.overrides.curr_bed_type);
  const def = useApp((state) => state.configDefs.get("curr_bed_type"));
  if (!panel || !def) return null;
  const options = bedTypeOptions(def, panel);
  const bedType = selection ? selection.bedType : (edited as string | undefined) ?? firstString(settings?.project, "curr_bed_type");
  const nozzle = nozzleOptions(panel, 0);
  const multiExtruderVariant = !panel.bbl && panel.nozzleDiameters.length > 1;
  return (
    <>
      {showsBedTypes(panel) && (
        <label className="kv-row">
          <span>Plate type</span>
          <select className="option-select printer-select" aria-label="Plate type" value={accordBedType(options, bedType ?? "")}
            onChange={(event) => selectBedType(event.target.value)}>
            {options.map(({ value, label }) => <option key={value} value={value}>{label}</option>)}
          </select>
        </label>
      )}
      {(panel.nozzleDiameters.length < 2 || multiExtruderVariant) && (
        <label className="kv-row">
          <span>{multiExtruderVariant ? "Variant" : "Nozzle"}</span>
          <select className="option-select printer-select" aria-label={multiExtruderVariant ? "Variant" : "Nozzle diameter"} value={nozzle.selected}
            onChange={(event) => void selectDiameter(panel, event.target.value)}>
            {nozzle.options.map((diameter) => <option key={diameter} value={diameter}>{`${diameter} mm`}</option>)}
          </select>
        </label>
      )}
    </>
  );
}
