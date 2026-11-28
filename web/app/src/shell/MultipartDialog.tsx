import { useEffect, useRef, useState } from "react";
import { DialogHeader } from "./DialogHeader";
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
    <dialog
      ref={dialog}
      className="dialog"
      aria-labelledby="multipart-title"
      aria-describedby="multipart-question"
      onClose={onClose}
      onKeyDown={(event) => {
        if (event.key !== "Tab") return;
        const controls = event.currentTarget.querySelectorAll<HTMLElement>("button, input");
        const first = controls[0];
        const last = controls[controls.length - 1];
        if (event.target === (event.shiftKey ? first : last)) {
          event.preventDefault();
          (event.shiftKey ? last : first).focus();
        }
      }}
    >
      <DialogHeader
        id="multipart-title"
        title="An object with multiple parts was detected"
        description="Load these files as a single object with multiple parts?"
        descriptionId="multipart-question"
        close={() => dialog.current!.close()}
      />
      <div className="dialog-body">
        <label className="check">
          <input type="checkbox" checked={autoDrop} onChange={(event) => setAutoDrop(event.target.checked)} />
          <span>Auto-Drop</span>
        </label>
      </div>
      <footer className="dialog-footer">
        <button className="button" onClick={() => choose(false)}>
          No
        </button>
        <button className="button primary" onClick={() => choose(true)}>
          Yes
        </button>
      </footer>
    </dialog>
  );
}
