import { useEffect, useRef } from "react";

const mac = /Mac|iPhone|iPad/.test(navigator.userAgent);
const ctrl = mac ? "\u2318+" : "Ctrl+";

/**
 * KBShortcutsDialog::fill_shortcuts, limited to the shortcuts this app
 * implements; descriptions keep the upstream wording.
 */
const GROUPS: [string, [string, string][]][] = [
  ["Global shortcuts", [
    [`${ctrl}I`, "Import geometry data from STL/3MF files"],
    [`${ctrl}G`, "Export plate sliced file"],
    [`${ctrl}R`, "Slice plate"],
    ["?", "Show keyboard shortcuts list"],
  ]],
  ["Plater", [
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
    ["A", "Arrange all objects"],
    ["Shift+A", "Arrange objects on selected plates"],
    // Shortcuts.cpp:33-41, 79-85.
    [mac ? "Backspace" : "Delete", "Delete Selected"],
    [`${ctrl}D`, "Delete All"],
    ["+", "Add instance"],
    ["-", "Remove instance"],
    ["Tab", "Switch between Prepare/Preview"],
  ]],
  ["Preview", [
    ["Arrow Up", "Vertical slider - Move active thumb Up"],
    ["Arrow Down", "Vertical slider - Move active thumb Down"],
    ["L", "On/Off one layer mode of the vertical slider"],
    ["C", "On/Off G-code window"],
    ["Tab", "Switch between Prepare/Preview"],
    ["Shift+Arrow Up/Down", "Move slider 5x faster"],
    ["Shift+Mouse wheel", "Move slider 5x faster"],
    [`${ctrl}Arrow Up/Down`, "Move slider 5x faster"],
    [`${ctrl}Mouse wheel`, "Move slider 5x faster"],
  ]],
];

export function ShortcutsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (open) dialog.current!.showModal();
  }, [open]);
  return (
    <dialog ref={dialog} className="shortcuts-dialog" aria-labelledby="shortcuts-title" onClose={onClose}>
      <header className="shortcuts-header">
        <h2 id="shortcuts-title">Keyboard Shortcuts</h2>
        <button className="bar-button" aria-label="Close" onClick={() => dialog.current!.close()}>{"\u2715"}</button>
      </header>
      <div className="shortcuts-groups">
        {GROUPS.map(([title, rows]) => (
          <section key={title}>
            <h3>{title}</h3>
            <dl>
              {rows.map(([keys, description]) => (
                <div className="shortcut-row" key={keys + description}>
                  <dt><kbd>{keys}</kbd></dt>
                  <dd>{description}</dd>
                </div>
              ))}
            </dl>
          </section>
        ))}
      </div>
    </dialog>
  );
}
