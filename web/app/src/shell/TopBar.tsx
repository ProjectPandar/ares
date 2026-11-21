import { Link } from "@tanstack/react-router";
import { type RefObject, useEffect, useRef, useState } from "react";
import { exportAllGcode, exportGcode, importFile, importFiles, setTheme, sliceAllPlates, slicePlate } from "../state/actions";
import { importPresetFiles, presetSliceBlocker } from "../state/presets";
import { useApp } from "../state/store";
import { FileMenu } from "./FileMenu";
import { MoonIcon, OrcaIcon, SunIcon } from "./icons";

/** BBLTopbar: the File menu and the project title. */
export function TitleBar({ importInput: input, projectInput, onShortcuts, onAbout }: {
  importInput: RefObject<HTMLInputElement | null>;
  projectInput: RefObject<HTMLInputElement | null>;
  onShortcuts: () => void;
  onAbout: () => void;
}) {
  const { theme, file } = useApp();
  const presetInput = useRef<HTMLInputElement>(null);
  return (
    <header className="titlebar">
      <div className="titlebar-left">
        <span className="brand" aria-label="Ares">
          <span className="brand-mark">A</span>
          Ares
        </span>
        <FileMenu importInput={input} projectInput={projectInput} presetInput={presetInput} />
        <input ref={input} type="file" accept=".3mf,.stl" multiple hidden data-testid="import-input"
          onChange={(event) => {
            importFiles([...(event.target.files ?? [])]);
            event.target.value = "";
          }} />
        {/* Plater::load_project (File > Open Project). */}
        <input ref={projectInput} type="file" accept=".3mf" hidden data-testid="project-input"
          onChange={(event) => {
            const picked = event.target.files?.[0];
            if (picked) void importFile(picked);
            event.target.value = "";
          }} />
        {/* MainFrame::load_config_file (File > Import > Import Configs). */}
        <input ref={presetInput} type="file" accept=".json,.zip,.orca_printer,.orca_bundle,.orca_filament" multiple hidden
          data-testid="preset-import-input" onChange={(event) => {
            const picked = [...(event.target.files ?? [])];
            if (picked.length) void importPresetFiles(picked);
            event.target.value = "";
          }} />
      </div>
      <span className="titlebar-title">{file ? `${file.name} - Ares` : "Ares"}</span>
      <div className="titlebar-right">
        <HelpMenu onShortcuts={onShortcuts} onAbout={onAbout} />
        <button className="bar-button" title={theme === "dark" ? "Light theme" : "Dark theme"}
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
          {theme === "dark" ? <SunIcon /> : <MoonIcon />}
        </button>
      </div>
    </header>
  );
}

/**
 * MainFrame::generate_help_menu (MainFrame.cpp:2903-2966) in the top bar's drop-down (:3683), as far
 * as ported: Keyboard Shortcuts (:2907-2909) and About (:2958-2963). Closes on a pick, Escape or a
 * click elsewhere.
 */
function HelpMenu({ onShortcuts, onAbout }: { onShortcuts: () => void; onAbout: () => void }) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (event: Event) => {
      if (event instanceof KeyboardEvent ? event.key === "Escape" : !root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("pointerdown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);
  const pick = (action: () => void) => () => {
    setOpen(false);
    action();
  };
  return (
    <div ref={root} className="help-menu">
      <button className="bar-button" aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen(!open)}>Help</button>
      {open && (
        <div className="context-menu" role="menu" aria-label="Help">
          <button type="button" role="menuitem" onClick={pick(onShortcuts)}>Keyboard Shortcuts</button>
          <button type="button" role="menuitem" onClick={pick(onAbout)}>About Ares</button>
        </div>
      )}
    </div>
  );
}

/**
 * MainFrame notebook: Prepare/Preview tabs, slice and export SideButtons. A multi-plate project
 * also offers "Slice all" (`eSliceAll`, Plater::priv::on_action_slice_all) and exports every
 * sliced plate.
 */
export function TabBar() {
  const { engine, scene, slice, presetSelection, plateSlices } = useApp();
  const slicing = slice.status === "slicing";
  const blocker = scene && presetSelection ? presetSliceBlocker(presetSelection, scene) : null;
  const multiPlate = (scene?.plates.length ?? 0) > 1;
  return (
    <nav className="tabbar">
      <div className="tabbar-tabs">
        <Link to="/prepare" className="tab" activeProps={{ className: "tab active" }}>
          <OrcaIcon name="tab_3d_active" size={20} /> Prepare
        </Link>
        <Link to="/preview" className="tab" activeProps={{ className: "tab active" }}>
          <OrcaIcon name="tab_preview_active" size={20} /> Preview
        </Link>
      </div>
      <div className="tabbar-actions">
        <button className="side-button" onClick={() => void slicePlate()}
          disabled={!scene || slicing || engine !== "ready" || blocker !== null}
          title={blocker ?? undefined}>
          {slicing ? "Slicing…" : "Slice plate"}
        </button>
        {multiPlate && (
          <button className="side-button" onClick={() => void sliceAllPlates()}
            disabled={slicing || engine !== "ready" || blocker !== null} title={blocker ?? undefined}>
            Slice all
          </button>
        )}
        <button className="side-button" onClick={exportGcode} disabled={slice.status !== "done"}>
          Export G-code file
        </button>
        {multiPlate && (
          <button className="side-button" onClick={exportAllGcode} disabled={slicing || Object.keys(plateSlices).length === 0}>
            Export all
          </button>
        )}
      </div>
    </nav>
  );
}
