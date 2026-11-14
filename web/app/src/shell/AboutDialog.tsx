import { useEffect, useRef, useState } from "react";
import { slicer } from "../state/slicer";

/**
 * AboutDialog (AboutDialog.cpp:230-360) in text only, as Ares has no logo: the version, large and
 * right-aligned (:259-279), the attribution (:289-292) and the licence
 * (CopyrightsDialog::get_html_text, :145-175).
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
    <dialog ref={dialog} className="about-dialog" aria-labelledby="about-title" onClose={onClose}>
      <header className="shortcuts-header">
        <h2 id="about-title">About Ares</h2>
        <button className="bar-button" aria-label="Close" onClick={() => dialog.current!.close()}>{"✕"}</button>
      </header>
      <div className="about-body">
        <p className="about-name">Ares</p>
        <p className="about-version" data-testid="about-version">{ARES_VERSION}</p>
        {orca && <p>Ares is a Rust port of the OrcaSlicer {orca} slicing core.</p>}
        <p className="about-credits">
          Open-source slicing stands on a tradition of collaboration and attribution. Slic3r, created by Alessandro
          Ranellucci and the RepRap community, laid the foundation. PrusaSlicer by Prusa Research built on that work,
          Bambu Studio forked from PrusaSlicer, and OrcaSlicer drew from both and grew far beyond them. Ares carries
          that work forward, crediting those who came before.
        </p>
        <p>
          Ares is licensed under the{" "}
          <a href="https://www.gnu.org/licenses/agpl-3.0.html" target="_blank" rel="noreferrer">
            GNU Affero General Public License, version 3
          </a>.
        </p>
      </div>
    </dialog>
  );
}
