import { useEffect, useRef } from "react";
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
    <dialog ref={dialog} className="shortcuts-dialog" aria-labelledby="engine-crash-title" onCancel={(event) => event.preventDefault()}>
      <header className="shortcuts-header">
        <h2 id="engine-crash-title">Slicing engine failed</h2>
      </header>
      <p className="engine-crash-message" data-testid="engine-crash-message">{crash}</p>
      <p className="switch-diameter-text">
        {fileName
          ? `Restarting the engine reopens ${fileName} from the original file with the current presets; edits made on the canvas after it was opened, and the undo history, are lost.`
          : "Restarting the engine reloads the page."}
      </p>
      <footer className="vendor-footer">
        <button className="side-button" onClick={() => void restartEngine()}>Restart engine</button>
      </footer>
    </dialog>
  );
}
