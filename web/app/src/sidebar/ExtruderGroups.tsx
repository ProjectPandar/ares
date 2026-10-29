import { useEffect, useRef, useState } from "react";
import { extruderState, isDual, setVolumeType, volumeTypeOptions } from "../state/extruders";
import { selectDiameter } from "../state/printer";
import { nozzleOptions, type PrinterPanel } from "../state/printerPanel";
import { useApp } from "../state/store";

// Plater.cpp:2044-2045: config extruder 0 is the left nozzle, 1 the right one.
const TITLES = ["Left Nozzle", "Right Nozzle"];

/** Sidebar::priv::switch_diameter's "Switch diameter" question (Plater.cpp:1249-1272). */
function SwitchDiameter({ panel, diameters, onClose }: { panel: PrinterPanel; diameters: [string, string]; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => dialog.current!.showModal(), []);
  const choose = (diameter: string) => {
    dialog.current!.close();
    void selectDiameter(panel, diameter);
  };
  return (
    <dialog ref={dialog} className="shortcuts-dialog switch-diameter" aria-labelledby="switch-diameter-title" onClose={onClose}>
      <header className="shortcuts-header"><h2 id="switch-diameter-title">Switch diameter</h2></header>
      <p className="switch-diameter-text">
        The software does not support using different diameter of nozzles for one print. If the left and right nozzles are
        inconsistent, we can only proceed with single-head printing. Please confirm which nozzle you would like to use for
        this project.
      </p>
      <footer className="vendor-footer">
        <button className="side-button" onClick={() => choose(diameters[0])}>{`Left nozzle: ${diameters[0]}mm`}</button>
        <button className="side-button" onClick={() => choose(diameters[1])}>{`Right nozzle: ${diameters[1]}mm`}</button>
        <button className="side-button" onClick={() => dialog.current!.close()}>Cancel</button>
      </footer>
    </dialog>
  );
}

/** ExtruderGroup (Plater.cpp:418-460, 1023-1140): each nozzle's diameter and, when it has a choice, its flow. */
export function ExtruderGroups() {
  const panel = useApp((state) => state.printerPanel);
  const volumeTypeDef = useApp((state) => state.configDefs.get("nozzle_volume_type"));
  // Re-render on the values extruderState reads.
  useApp((state) => state.presetSelection);
  useApp((state) => state.overrides);
  useApp((state) => state.scene);
  const [mismatch, setMismatch] = useState<[string, string] | null>(null);
  if (!isDual(panel) || !volumeTypeDef) return null;
  const { volumeTypes } = extruderState();
  const nozzles = [nozzleOptions(panel, 0), nozzleOptions(panel, 1)];
  const pickDiameter = (extruder: number, diameter: string) => {
    const pair: [string, string] = extruder === 0 ? [diameter, nozzles[1].selected] : [nozzles[0].selected, diameter];
    if (pair[0] === pair[1]) void selectDiameter(panel, diameter);
    else setMismatch(pair);
  };
  return (
    <div className="extruder-groups">
      {TITLES.map((title, extruder) => {
        const flows = volumeTypeOptions(panel, extruder, volumeTypeDef.enum_values);
        return (
          <fieldset key={title} className="extruder-group" aria-label={title}>
            <legend>{title}</legend>
            <label className="kv-row">
              <span>Diameter</span>
              <select className="option-select printer-select" aria-label={`${title} diameter`} value={nozzles[extruder].selected}
                onChange={(event) => pickDiameter(extruder, event.target.value)}>
                {nozzles[extruder].options.map((diameter) => <option key={diameter} value={diameter}>{`${diameter} mm`}</option>)}
              </select>
            </label>
            {flows.length > 1 && (
              <label className="kv-row">
                <span>Flow</span>
                <select className="option-select printer-select" aria-label={`${title} flow`} value={volumeTypes[extruder]}
                  onChange={(event) => setVolumeType(extruder, event.target.value)}>
                  {flows.map((flow) => <option key={flow} value={flow}>{flow}</option>)}
                </select>
              </label>
            )}
          </fieldset>
        );
      })}
      {mismatch && <SwitchDiameter panel={panel} diameters={mismatch} onClose={() => setMismatch(null)} />}
    </div>
  );
}
