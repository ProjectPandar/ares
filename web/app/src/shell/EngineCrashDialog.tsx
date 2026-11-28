import { useEffect, useRef } from "react";
import { DialogHeader } from "./DialogHeader";
import { restartEngine } from "../state/engineRecovery";
import { useApp } from "../state/store";

/**
 * The HardCrash text with an engine restart: Ares UI without an upstream counterpart, where the user
 * restarts the application (BackgroundSlicingProcess.cpp:110-112).
 */
export function EngineCrashDialog() {
  const crash = useApp((state) => state.engineCrash);
  const fileName = useApp((state) => state.file?.name);
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (crash) dialog.current!.showModal();
  }, [crash]);
  if (!crash) return null;
  return (
    <dialog
      ref={dialog}
      className="dialog"
      role="alertdialog"
      aria-labelledby="engine-crash-title"
      onCancel={(event) => event.preventDefault()}
    >
      <DialogHeader id="engine-crash-title" title="Slicing engine failed" />
      <div className="dialog-body">
        <div className="stack">
          <p className="notice error engine-crash-message" data-testid="engine-crash-message" role="alert">
            {crash}
          </p>
          <p>
            {fileName
              ? `Restarting the engine reopens ${fileName} from the original file with the current presets; edits made on the canvas after it was opened, and the undo history, are lost.`
              : "Restarting the engine reloads the page."}
          </p>
        </div>
      </div>
      <footer className="dialog-footer">
        <button className="button primary" onClick={() => void restartEngine()}>
          Restart engine
        </button>
      </footer>
    </dialog>
  );
}
