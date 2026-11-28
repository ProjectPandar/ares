import type { GuideFilament } from "@ares-slicer/wasm";
import { useEffect, useMemo, useRef, useState } from "react";
import { DialogHeader } from "../shell/DialogHeader";
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
export function FilamentPage({
  filaments,
  selected,
  onBack,
  onFinish,
}: {
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
  const notice = useRef<HTMLDialogElement>(null);
  const [showNotice, setShowNotice] = useState(false);
  useEffect(() => {
    if (showNotice) notice.current!.showModal();
  }, [showNotice]);
  const toggle = (key: string, on: boolean) => {
    const next = new Set(checked);
    if (on) next.add(key);
    else next.delete(key);
    setChecked(next);
  };
  // ResponseFilamentResult (common.js:486-520).
  const finish = () => {
    if (checked.size === 0) setShowNotice(true);
    else onFinish(model.items.filter(({ key }) => checked.has(key)).flatMap(({ names }) => names));
  };
  return (
    <>
      <div className="dialog-body">
        <section className="frame" role="group" aria-label="Filaments">
          <header className="frame-head">
            <h2>Filaments</h2>
          </header>
          <div className="frame-content">
            {model.items.map((item) => (
              <label key={item.key} className="check" title={item.names.join("\n")}>
                <input
                  type="checkbox"
                  aria-label={item.short}
                  checked={checked.has(item.key)}
                  onChange={(event) => toggle(item.key, event.target.checked)}
                />
                <span>{item.short}</span>
                <span className="check-detail">
                  {item.vendor} · {item.type}
                </span>
              </label>
            ))}
          </div>
        </section>
      </div>
      <footer className="dialog-footer">
        <span className="summary" data-testid="filament-count">
          {checked.size} / {model.items.length}
        </span>
        <button className="button" onClick={onBack}>
          Back
        </button>
        <button className="button primary" onClick={finish}>
          Finish
        </button>
      </footer>
      {showNotice && (
        <dialog
          ref={notice}
          className="dialog"
          role="alertdialog"
          aria-labelledby="wizard-notice-title"
          // React bubbles a dialog's close and cancel through its ancestors: the wizard under it stays open.
          onClose={(event) => {
            event.stopPropagation();
            setShowNotice(false);
          }}
          onCancel={(event) => event.stopPropagation()}
        >
          <DialogHeader id="wizard-notice-title" title="Error" />
          <div className="dialog-body">
            <p className="notice error" role="alert">
              At least one filament must be selected.
            </p>
          </div>
          <footer className="dialog-footer">
            <button className="button primary" onClick={() => notice.current!.close()}>
              OK
            </button>
          </footer>
        </dialog>
      )}
    </>
  );
}
