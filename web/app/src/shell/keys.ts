/**
 * Shortcuts act on the canvas and main frame (GLCanvas3D::on_char,
 * MainFrame key handler), so they stay off while a form field has focus, a
 * control already handled the key, or a dialog is open.
 */
export function isShortcut(event: KeyboardEvent): boolean {
  if (event.defaultPrevented || document.querySelector("dialog[open]")) return false;
  return !(event.target as Element).closest("input, textarea, select, [contenteditable]");
}

/** Ctrl on Windows/Linux, Cmd on macOS (wxWidgets `CmdDown`). */
export const cmdDown = (event: KeyboardEvent) => event.ctrlKey || event.metaKey;

export const mac = /Mac|iPhone|iPad/.test(navigator.userAgent);
/** The Ctrl key's label in shortcuts: ⌘ on macOS. */
export const ctrlLabel = mac ? "\u2318+" : "Ctrl+";
