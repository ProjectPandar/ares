import { useEffect, useRef, useState } from "react";
import { importStlFiles } from "../state/stlImport";

export function MultipartDialog({ files, onClose }: { files: File[]; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [autoDrop, setAutoDrop] = useState(true);
  useEffect(() => dialog.current!.showModal(), []);
  const choose = (asMultipart: boolean) => {
    dialog.current!.close();
    onClose();
    void importStlFiles(files, { asMultipart, autoDrop });
  };
  return (
    <dialog ref={dialog} className="shortcuts-dialog" aria-labelledby="multipart-title" aria-describedby="multipart-question" onClose={onClose}
      onKeyDown={(event) => {
        if (event.key !== "Tab") return;
        const controls = event.currentTarget.querySelectorAll<HTMLElement>("button, input");
        const first = controls[0];
        const last = controls[controls.length - 1];
        if (event.target === (event.shiftKey ? first : last)) {
          event.preventDefault();
          (event.shiftKey ? last : first).focus();
        }
      }}>
      <header className="shortcuts-header">
        <h2 id="multipart-title">An object with multiple parts was detected</h2>
        <button className="bar-button" aria-label="Close" onClick={() => dialog.current!.close()}>{"\u2715"}</button>
      </header>
      <p id="multipart-question" className="switch-diameter-text">Load these files as a single object with multiple parts?</p>
      <label className="vendor-item">
        <input type="checkbox" checked={autoDrop} onChange={(event) => setAutoDrop(event.target.checked)} />
        Auto-Drop
      </label>
      <footer className="vendor-footer">
        <button className="side-button" onClick={() => choose(true)}>Yes</button>
        <button className="side-button" onClick={() => choose(false)}>No</button>
      </footer>
    </dialog>
  );
}
