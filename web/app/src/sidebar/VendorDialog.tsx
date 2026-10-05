import { vendors } from "@ares-slicer/profiles";
import { useEffect, useRef, useState } from "react";
import { FILAMENT_LIBRARY, setVendors } from "../state/presets";
import { useApp } from "../state/store";

/**
 * ConfigWizard printer page, reduced to vendor selection: the checked
 * vendors' system presets join the catalog (PresetBundle::load_system_presets).
 */
export function VendorDialog({ onClose }: { onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [checked, setChecked] = useState(() => new Set(useApp.getState().vendors));
  useEffect(() => dialog.current!.showModal(), []);
  const toggle = (id: string, on: boolean) => {
    const next = new Set(checked);
    if (on) next.add(id);
    else next.delete(id);
    setChecked(next);
  };
  return (
    <dialog ref={dialog} className="shortcuts-dialog" aria-labelledby="vendors-title" onClose={onClose}>
      <header className="shortcuts-header">
        <h2 id="vendors-title">System presets</h2>
        <button className="bar-button" aria-label="Close" onClick={() => dialog.current!.close()}>{"\u2715"}</button>
      </header>
      <div className="vendor-list">
        {vendors.filter((vendor) => vendor.id !== FILAMENT_LIBRARY).map((vendor) => (
          <label key={vendor.id} className="vendor-item" title={vendor.description}>
            <input type="checkbox" checked={checked.has(vendor.id)}
              onChange={(event) => toggle(vendor.id, event.target.checked)} />
            {vendor.name ?? vendor.id}
          </label>
        ))}
      </div>
      <footer className="vendor-footer">
        <button className="side-button" onClick={() => {
          void setVendors(vendors.map((vendor) => vendor.id).filter((id) => checked.has(id)));
          dialog.current!.close();
        }}>Done</button>
      </footer>
    </dialog>
  );
}
