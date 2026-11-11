import type { GuideFilament, GuideModel, InstalledPrinters } from "@ares-slicer/wasm";
import { type KeyboardEvent, type SyntheticEvent, useEffect, useRef, useState } from "react";
import { installedVendors, useApp } from "../state/store";
import { defaultMaterials, finishWizard, guideFilaments, guideModels, installedFromChecked } from "../state/wizard";
import { FilamentPage, listedFilaments } from "./FilamentPage";
import { ModelPage, sortedModels } from "./ModelPage";

/** HandleModelList's pre-checked models (common.js:118-125): those with installed nozzles. */
const installedModels = (installed: InstalledPrinters) =>
  new Set(Object.values(installed).flatMap((models) => Object.keys(models).filter((model) => models[model].length > 0)));

/** The filament page's data: the listed filaments and the presets checked at open. */
interface FilamentStep {
  installed: InstalledPrinters;
  filaments: GuideFilament[];
  selected: ReadonlySet<string>;
}

/**
 * GuideFrame (WebGuideDialog.cpp): on first run, Printer Selection (page 21) then Filament
 * Selection (page 22) with no way to close; from the printer combo, Printer Selection alone
 * (page 24) with Confirm and Cancel, closed by Escape (`:271`).
 */
export function SetupWizard({ firstRun, onClose }: { firstRun: boolean; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [models, setModels] = useState<GuideModel[] | null>(null);
  const [checked, setChecked] = useState<ReadonlySet<string>>(() => installedModels(useApp.getState().installedPrinters));
  const [filamentStep, setFilamentStep] = useState<FilamentStep | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    dialog.current!.showModal();
    void guideModels().then((loaded) => setModels(sortedModels(loaded)));
  }, []);

  // save_userguide_models (WebGuideDialog.cpp:607-660): the checked models with all their nozzles
  // and their default materials selected among the listed filaments.
  const saveModels = async (all: GuideModel[]) => {
    const installed = installedFromChecked(all, checked);
    const filaments = await guideFilaments(installedVendors(installed));
    const selected = new Set([...useApp.getState().installedFilaments, ...defaultMaterials(all, checked, filaments)]);
    return { installed, filaments, selected };
  };
  const run = async (task: () => Promise<void>) => {
    setBusy(true);
    try {
      await task();
    } finally {
      setBusy(false);
    }
  };
  // OnExitFilter (common.js:416-444): nothing happens while no model is checked.
  const next = (all: GuideModel[]) => run(async () => {
    const step = await saveModels(all);
    const pairs = all.filter(({ model }) => checked.has(model)).flatMap(({ model, variants }) => variants.map((variant) => `${model}++${variant}`));
    setFilamentStep({ ...step, filaments: listedFilaments(step.filaments, pairs) });
  });
  const confirm = (all: GuideModel[]) => run(async () => {
    const { installed, selected } = await saveModels(all);
    await finishWizard(installed, [...selected], all);
  });
  const blockClose = (event: SyntheticEvent | KeyboardEvent) => {
    if (firstRun || busy) event.preventDefault();
  };

  return (
    <dialog ref={dialog} className="shortcuts-dialog wizard-dialog" aria-labelledby="wizard-title" onClose={onClose}
      onCancel={blockClose} onKeyDown={(event) => event.key === "Escape" && blockClose(event)}>
      <header className="shortcuts-header">
        <h2 id="wizard-title">{filamentStep ? "Filament Selection" : "Printer Selection"}</h2>
        {!firstRun && <button className="bar-button" aria-label="Close" disabled={busy} onClick={() => dialog.current!.close()}>{"✕"}</button>}
      </header>
      {!models && <div className="side-empty">Loading printers…</div>}
      {models && filamentStep && (
        <FilamentPage filaments={filamentStep.filaments} selected={filamentStep.selected}
          onBack={() => setFilamentStep(null)}
          onFinish={(filaments) => void run(() => finishWizard(filamentStep.installed, filaments, models))} />
      )}
      {models && !filamentStep && (
        <>
          <ModelPage models={models} checked={checked} onChange={setChecked} />
          <footer className="vendor-footer wizard-footer">
            {firstRun ? (
              <button className="side-button" disabled={busy} onClick={() => checked.size > 0 && void next(models)}>Next</button>
            ) : (
              <>
                <button className="side-button" disabled={busy} onClick={() => checked.size > 0 && void confirm(models)}>Confirm</button>
                <button className="side-button" disabled={busy} onClick={() => dialog.current!.close()}>Cancel</button>
              </>
            )}
          </footer>
        </>
      )}
    </dialog>
  );
}
