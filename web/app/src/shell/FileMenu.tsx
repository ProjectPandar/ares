import { useRouter } from "@tanstack/react-router";
import { type KeyboardEvent, type MouseEvent, type RefObject, useEffect, useId, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { exportGcode, newProject } from "../state/actions";
import { useApp } from "../state/store";
import { OrcaIcon } from "./icons";
import { ctrlLabel, mac } from "./keys";

/** A row: runs `run` when enabled, or shows why the web cannot perform it yet. */
interface Item {
  label: string;
  run?: () => void;
  reason?: string;
  /** The Ctrl key bound to `run`. */
  key?: string;
  submenu?: Row[];
}
type Row = Item | "-";

const WRITER = "Needs the 3MF project writer.";
const STL = "STL export is not available yet.";
const DRC = "DRC export is not available yet.";

const items = (menu: HTMLElement) =>
  [...menu.querySelectorAll<HTMLElement>(":scope > [role=menuitem], :scope > [role=none] > [role=menuitem]")];
const focusFirst = (menu: HTMLElement) => {
  const all = items(menu);
  (all.find((item) => item.getAttribute("aria-disabled") !== "true") ?? all[0]).focus();
};

/**
 * MainFrame::init_menubar_as_editor's File menu (MainFrame.cpp:3030-3183, :3741-3779 on macOS), shown by
 * BBLTopbar's File tool (BBLTopbar.cpp:290-292, :647-659). Items the web cannot perform yet are disabled
 * with their reason.
 */
export function FileMenu({ importInput, projectInput, presetInput }: {
  importInput: RefObject<HTMLInputElement | null>;
  projectInput: RefObject<HTMLInputElement | null>;
  presetInput: RefObject<HTMLInputElement | null>;
}) {
  const { engine, file, loadingFile, slice } = useApp();
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [submenu, setSubmenu] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const menuId = useId();
  const ready = engine === "ready";
  const slicing = slice.status === "slicing";

  const close = () => {
    setOpen(false);
    setSubmenu(null);
  };
  const startNewProject = () => {
    void newProject();
    void router.navigate({ to: "/prepare" });
  };
  const rows: Row[] = [
    { label: "New Window", run: () => window.open(location.origin + location.pathname, "_blank", "noopener") },
    { label: "New Project", run: slicing ? undefined : () => (file ? setConfirming(true) : startNewProject()) },
    { label: "Open Project…", key: "O", run: ready && !loadingFile && !slicing ? () => projectInput.current!.click() : undefined },
    { label: "Recent files", reason: "Recent files are not kept in the browser yet.", submenu: [] },
    { label: "Save Project", reason: WRITER },
    { label: "Save Project as…", reason: WRITER },
    "-",
    { label: "Import", submenu: [
      { label: "Import 3MF/STL…", key: "I", run: ready && !loadingFile ? () => importInput.current!.click() : undefined },
      { label: "Import ZIP Archive…", reason: "ZIP archives cannot be imported yet." },
      { label: "Import Configs…", run: ready ? () => presetInput.current!.click() : undefined },
    ] },
    { label: "Export", submenu: [
      { label: "Export all objects as one STL…", reason: STL },
      { label: "Export all objects as STLs…", reason: STL },
      { label: "Export all objects as one DRC…", reason: DRC },
      { label: "Export all objects as DRCs…", reason: DRC },
      { label: "Export Generic 3MF…", reason: WRITER },
      { label: "Export plate sliced file…", reason: WRITER },
      { label: "Export all plate sliced file…", reason: WRITER },
      // Ctrl+G runs the G-code export until the sliced file export takes it (Shortcuts.cpp:70).
      { label: "Export G-code…", key: "G", run: slice.status === "done" ? exportGcode : undefined },
      { label: "Export toolpaths as OBJ…", reason: "Toolpath OBJ export is not available yet." },
      { label: "Export Preset Bundle…", reason: "Preset bundle export is not available yet." },
    ] },
    "-",
    { label: "Sync Presets", reason: "Cloud preset sync is not available in the browser." },
    "-",
    { label: "Preset Bundle", reason: "The preset bundle dialog is not available yet." },
  ];

  useEffect(() => {
    if (!open) return;
    const shut = () => {
      setOpen(false);
      setSubmenu(null);
    };
    const outside = (event: PointerEvent) => {
      if (!root.current!.contains(event.target as Node)) shut();
    };
    // Escape leaves the open submenu first, then the menu.
    const escape = (event: globalThis.KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      const parent = submenu && root.current!.querySelector<HTMLElement>(`[aria-expanded=true][role=menuitem]`);
      if (parent) {
        setSubmenu(null);
        if (root.current!.contains(document.activeElement)) parent.focus();
      } else {
        shut();
        button.current!.focus();
      }
    };
    document.addEventListener("pointerdown", outside);
    document.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("pointerdown", outside);
      document.removeEventListener("keydown", escape);
    };
  }, [open, submenu]);

  const openMenu = () => {
    flushSync(() => setOpen(true));
    focusFirst(root.current!.querySelector<HTMLElement>("[role=menu]")!);
  };
  const openSubmenu = (label: string, parent: HTMLElement) => {
    flushSync(() => setSubmenu(label));
    focusFirst(parent.nextElementSibling as HTMLElement);
  };
  // Arrow keys move within the focused item's own menu; ArrowRight enters a submenu, ArrowLeft leaves it.
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const menu = event.currentTarget;
    const current = event.target as HTMLElement;
    const all = items(menu);
    const index = all.indexOf(current);
    if (index < 0) return;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      all[(index + (event.key === "ArrowDown" ? 1 : all.length - 1)) % all.length].focus();
    } else if (event.key === "ArrowRight" && current.getAttribute("aria-haspopup") && current.getAttribute("aria-disabled") !== "true") {
      openSubmenu(current.getAttribute("aria-label")!, current);
    } else if (event.key === "ArrowLeft" && menu.dataset.submenu) {
      setSubmenu(null);
      (menu.previousElementSibling as HTMLElement).focus();
    } else {
      if (event.key === "Tab") close();
      return;
    }
    event.preventDefault();
    event.stopPropagation();
  };

  const renderRows = (list: Row[], nested: boolean) => list.map((row, index) => {
    if (row === "-") return <div key={`-${index}`} role="separator" className="menu-separator" />;
    const enabled = row.run !== undefined || (row.submenu !== undefined && row.reason === undefined);
    const expanded = submenu === row.label;
    const reasonId = row.reason && `${menuId}-${row.label.replace(/\W/g, "")}`;
    const onClick = (event: MouseEvent<HTMLButtonElement>) => {
      if (!enabled) return;
      if (row.submenu) {
        // Opens, never toggles: a mouse hover has already opened it. A keyboard click (Enter or Space)
        // focuses its first item.
        if (event.detail === 0) openSubmenu(row.label, event.currentTarget);
        else setSubmenu(row.label);
        return;
      }
      close();
      // A keyboard pick returns the focus to the File button.
      if (event.detail === 0) button.current!.focus();
      row.run!();
    };
    const control = (
      <button key={row.label} type="button" role="menuitem" aria-label={row.label}
        aria-disabled={enabled ? undefined : true} aria-describedby={reasonId}
        aria-haspopup={row.submenu ? "menu" : undefined} aria-expanded={row.submenu ? expanded : undefined}
        aria-keyshortcuts={row.key && `${mac ? "Meta" : "Control"}+${row.key}`}
        onClick={onClick}
        onPointerEnter={(event) => {
          if (!nested && event.pointerType === "mouse") setSubmenu(row.submenu && enabled ? row.label : null);
        }}>
        <span className="menu-label">{row.label}</span>
        {row.key && <span className="menu-shortcut">{`${ctrlLabel}${row.key}`}</span>}
        {row.submenu && <span className="menu-arrow" aria-hidden>›</span>}
        {row.reason && <span className="menu-reason" id={reasonId}>{row.reason}</span>}
      </button>
    );
    if (!row.submenu) return control;
    return (
      <div key={row.label} role="none" className="file-submenu">
        {control}
        {expanded && (
          <div className="context-menu" role="menu" aria-label={row.label} data-submenu="true" onKeyDown={onKeyDown}>
            {renderRows(row.submenu, true)}
          </div>
        )}
      </div>
    );
  });

  return (
    <div ref={root} className="file-menu">
      <button ref={button} className="bar-button" aria-haspopup="menu" aria-expanded={open}
        onClick={(event) => {
          if (open) close();
          else if (event.detail === 0) openMenu();
          else setOpen(true);
        }}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" || open) return;
          event.preventDefault();
          openMenu();
        }}>
        <OrcaIcon name="topbar_file" /> <span>File</span>
      </button>
      {open && (
        <div className="context-menu" role="menu" aria-label="File" onKeyDown={onKeyDown}>
          {renderRows(rows, false)}
        </div>
      )}
      {confirming && <NewProjectDialog onClose={(discard) => {
        setConfirming(false);
        if (discard) startNewProject();
      }} />}
    </div>
  );
}

/** Plater::close_with_confirm before `new_project` (Plater.cpp:16064), without Save until there is a project writer. */
function NewProjectDialog({ onClose }: { onClose: (discard: boolean) => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const discard = useRef(false);
  useEffect(() => dialog.current!.showModal(), []);
  const answer = (value: boolean) => {
    discard.current = value;
    dialog.current!.close();
  };
  return (
    <dialog ref={dialog} className="shortcuts-dialog" aria-labelledby="new-project-title" onClose={() => onClose(discard.current)}>
      <header className="shortcuts-header">
        <h2 id="new-project-title">Discard the current project?</h2>
      </header>
      <p className="switch-diameter-text">The model, its plates and slice results are closed. The selected presets stay.</p>
      <footer className="vendor-footer">
        <button className="side-button" onClick={() => answer(true)}>Discard</button>
        <button className="side-button" onClick={() => answer(false)}>Cancel</button>
      </footer>
    </dialog>
  );
}
