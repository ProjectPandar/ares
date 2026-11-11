import type { GuideFilament } from "@ares-slicer/wasm";
import { useMemo, useState } from "react";
import { checkedItems, dialogModel } from "../sidebar/filamentDialog";

/**
 * SortUI's filament filter (guide/22/common.js:93-118): a filament with no tag, or whose
 * `[model++variant]` tags name a selected model and nozzle.
 */
export function listedFilaments(filaments: readonly GuideFilament[], selected: readonly string[]) {
  return filaments.filter(({ models }) => {
    const tags = models.map(([model, variant]) => `[${model}++${variant}]`).join("");
    return tags === "" || selected.some((pair) => tags.includes(pair));
  });
}

/**
 * Filament Selection, guide page 22: the listed filaments merged by vendor, type and short name,
 * checked when one of their presets is selected; Finish stores every preset of the checked items.
 */
export function FilamentPage({ filaments, selected, onBack, onFinish }: {
  filaments: readonly GuideFilament[];
  selected: ReadonlySet<string>;
  onBack: () => void;
  onFinish: (filaments: string[]) => void;
}) {
  const model = useMemo(
    () => dialogModel(filaments.map(({ name, vendor, filament_type }) => ({ name, vendor, type: filament_type }))),
    [filaments],
  );
  const [checked, setChecked] = useState<ReadonlySet<string>>(() => checkedItems(model.items, selected));
  const [notice, setNotice] = useState(false);
  const toggle = (key: string, on: boolean) => {
    const next = new Set(checked);
    if (on) next.add(key);
    else next.delete(key);
    setChecked(next);
  };
  // ResponseFilamentResult (common.js:486-520).
  const finish = () => {
    if (checked.size === 0) setNotice(true);
    else onFinish(model.items.filter(({ key }) => checked.has(key)).flatMap(({ names }) => names));
  };
  return (
    <>
      <div className="wizard-filaments" role="group" aria-label="Filaments">
        {model.items.map((item) => (
          <label key={item.key} className="vendor-item" title={item.names.join("\n")}>
            <input type="checkbox" checked={checked.has(item.key)} onChange={(event) => toggle(item.key, event.target.checked)} />
            {item.short}
            <span className="wizard-filament-meta">{item.vendor} · {item.type}</span>
          </label>
        ))}
      </div>
      <footer className="vendor-footer wizard-footer">
        <span className="filament-count" data-testid="filament-count">{checked.size} / {model.items.length}</span>
        <button className="side-button" onClick={onBack}>Back</button>
        <button className="side-button" onClick={finish}>Finish</button>
      </footer>
      {notice && (
        <div className="wizard-notice" role="alertdialog" aria-labelledby="wizard-notice-title" aria-describedby="wizard-notice-text">
          <div id="wizard-notice-title" className="wizard-notice-bar">Error</div>
          <p id="wizard-notice-text">At least one filament must be selected.</p>
          <button className="side-button" onClick={() => setNotice(false)}>OK</button>
        </div>
      )}
    </>
  );
}
