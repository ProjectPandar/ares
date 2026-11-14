import { Link } from "@tanstack/react-router";
import { type RefObject, useRef } from "react";
import { exportAllGcode, exportGcode, importFiles, setTheme, sliceAllPlates, slicePlate } from "../state/actions";
import { importPresetFiles, presetSliceBlocker } from "../state/presets";
import { useApp } from "../state/store";
import { MoonIcon, OrcaIcon, SunIcon } from "./icons";

/** BBLTopbar: file actions and the project title. */
export function TitleBar({ importInput: input, onShortcuts }: {
  importInput: RefObject<HTMLInputElement | null>;
  onShortcuts: () => void;
}) {
  const { theme, engine, file, loadingFile } = useApp();
  const presetInput = useRef<HTMLInputElement>(null);
  return (
    <header className="titlebar">
      <div className="titlebar-left">
        <span className="brand" aria-label="Ares">
          <span className="brand-mark">A</span>
          Ares
        </span>
        <button className="bar-button" title="Import 3MF/STL" onClick={() => input.current?.click()}
          disabled={engine !== "ready" || loadingFile}>
          <OrcaIcon name="topbar_open" /> <span>Import</span>
        </button>
        <input ref={input} type="file" accept=".3mf,.stl" multiple hidden data-testid="import-input"
          onChange={(event) => {
            importFiles([...(event.target.files ?? [])]);
            event.target.value = "";
          }} />
        {/* MainFrame::load_config_file (File > Import > Import Configs). */}
        <button className="bar-button" title="Import Configs" onClick={() => presetInput.current?.click()}
          disabled={engine !== "ready"}>
          <span>Import Configs</span>
        </button>
        <input ref={presetInput} type="file" accept=".json,.zip,.orca_printer,.orca_bundle,.orca_filament" multiple hidden
          data-testid="preset-import-input" onChange={(event) => {
            const picked = [...(event.target.files ?? [])];
            if (picked.length) void importPresetFiles(picked);
            event.target.value = "";
          }} />
      </div>
      <span className="titlebar-title">{file ? `${file.name} - Ares` : "Ares"}</span>
      <div className="titlebar-right">
        <button className="bar-button" title="Keyboard Shortcuts (?)" aria-label="Keyboard Shortcuts"
          onClick={onShortcuts}>?</button>
        <button className="bar-button" title={theme === "dark" ? "Light theme" : "Dark theme"}
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
          {theme === "dark" ? <SunIcon /> : <MoonIcon />}
        </button>
      </div>
    </header>
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
