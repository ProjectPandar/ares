import { useEffect, useRef } from "react";
import { DialogHeader } from "./DialogHeader";
import { ctrlLabel as ctrl, mac } from "./keys";

/**
 * KBShortcutsDialog::fill_shortcuts, limited to the shortcuts this app
 * implements; descriptions keep the upstream wording.
 */
const GROUPS: [string, [string, string][]][] = [
  [
    "Global shortcuts",
    [
      [`${ctrl}O`, "Open Project"],
      [`${ctrl}I`, "Import geometry data from STL/3MF files"],
      [`${ctrl}G`, "Export plate sliced file"],
      [`${ctrl}R`, "Slice plate"],
      ["?", "Show keyboard shortcuts list"],
    ],
  ],
  [
    "Plater",
    [
      ["Mouse wheel", "Zoom View"],
      ["Esc", "Deselect all"],
      [`${ctrl}0`, "Camera view - Default"],
      [`${ctrl}1`, "Camera view - Top"],
      [`${ctrl}2`, "Camera view - Bottom"],
      [`${ctrl}3`, "Camera view - Front"],
      [`${ctrl}4`, "Camera view - Behind"],
      [`${ctrl}5`, "Camera Angle - Left side"],
      [`${ctrl}6`, "Camera Angle - Right side"],
      ["I", "Zoom in"],
      ["O", "Zoom out"],
      ["M", "Gizmo move"],
      ["R", "Gizmo rotate"],
      ["S", "Gizmo scale"],
      ["A", "Arrange all objects"],
      ["Shift+A", "Arrange objects on selected plates"],
      // Shortcuts.cpp:74-75; Ctrl+Shift+Z redoes too.
      [`${ctrl}Z`, "Undo"],
      [`${ctrl}Y`, "Redo"],
      [`${ctrl}Shift+Z`, "Redo"],
      // Shortcuts.cpp:33-41, 79-85.
      [mac ? "Backspace" : "Delete", "Delete Selected"],
      [`${ctrl}D`, "Delete All"],
      ["+", "Add instance"],
      ["-", "Remove instance"],
      ["Tab", "Switch between Prepare/Preview"],
    ],
  ],
  [
    "Preview",
    [
      ["Arrow Up", "Vertical slider - Move active thumb Up"],
      ["Arrow Down", "Vertical slider - Move active thumb Down"],
      ["L", "On/Off one layer mode of the vertical slider"],
      ["C", "On/Off G-code window"],
      ["Tab", "Switch between Prepare/Preview"],
      ["Shift+Arrow Up/Down", "Move slider 5x faster"],
      ["Shift+Mouse wheel", "Move slider 5x faster"],
      [`${ctrl}Arrow Up/Down`, "Move slider 5x faster"],
      [`${ctrl}Mouse wheel`, "Move slider 5x faster"],
    ],
  ],
];

export function ShortcutsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (open) dialog.current!.showModal();
  }, [open]);
  return (
    <dialog ref={dialog} className="dialog" aria-labelledby="shortcuts-title" onClose={onClose}>
      <DialogHeader id="shortcuts-title" title="Keyboard Shortcuts" close={() => dialog.current!.close()} />
      <div className="dialog-body">
        <div className="stack">
          {GROUPS.map(([title, rows]) => (
            <section className="frame" key={title}>
              <header className="frame-head">
                <h2>{title}</h2>
              </header>
              <div className="frame-content">
                <dl className="shortcuts-list">
                  {rows.map(([keys, description]) => (
                    <div key={keys + description}>
                      <dt>{description}</dt>
                      <dd>
                        <kbd>{keys}</kbd>
                      </dd>
                    </div>
                  ))}
                </dl>
              </div>
            </section>
          ))}
        </div>
      </div>
    </dialog>
  );
}
