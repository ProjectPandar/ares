import { useEffect, useRef, useState } from "react";
import { DialogHeader } from "./DialogHeader";
import { slicer } from "../state/slicer";

/**
 * AboutDialog (AboutDialog.cpp:230-360) in text only: the mark with the version (:259-279), the
 * attribution (:289-292) and the licence (CopyrightsDialog::get_html_text, :145-175).
 */
export function AboutDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [orca, setOrca] = useState<string>();
  useEffect(() => {
    if (!open) return;
    dialog.current!.showModal();
    void slicer.orcaSlicerVersion().then(setOrca);
  }, [open]);
  return (
    <dialog ref={dialog} className="dialog" aria-labelledby="about-title" onClose={onClose}>
      <DialogHeader id="about-title" title="About Ares" close={() => dialog.current!.close()} />
      <div className="dialog-body">
        <div className="stack">
          <div className="about-identity">
            <span className="about-mark" aria-hidden="true">
              A
            </span>
            <div>
              <strong>Ares</strong>
              <div className="muted" data-testid="about-version">
                {ARES_VERSION}
              </div>
            </div>
          </div>
          {orca && <p>Ares is a Rust port of the OrcaSlicer {orca} slicing core.</p>}
          <p className="muted">
            Open-source slicing stands on a tradition of collaboration and attribution. Slic3r, created by Alessandro
            Ranellucci and the RepRap community, laid the foundation. PrusaSlicer by Prusa Research built on that work,
            Bambu Studio forked from PrusaSlicer, and OrcaSlicer drew from both and grew far beyond them. Ares carries
            that work forward, crediting those who came before.
          </p>
          <p>
            Ares is licensed under the{" "}
            <a href="https://www.gnu.org/licenses/agpl-3.0.html" target="_blank" rel="noreferrer">
              GNU Affero General Public License, version 3
            </a>
            .
          </p>
        </div>
      </div>
    </dialog>
  );
}
