import { Link } from "@tanstack/react-router";
import type { RefObject } from "react";
import { exportGcode, importFile, setTheme, slicePlate } from "../state/actions";
import { useApp } from "../state/store";
import { MoonIcon, OrcaIcon, SunIcon } from "./icons";

/** BBLTopbar: file actions and the project title. */
export function TitleBar({ importInput: input, onShortcuts }: {
  importInput: RefObject<HTMLInputElement | null>;
  onShortcuts: () => void;
}) {
  const { theme, engine, file, loadingFile } = useApp();
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
        <input ref={input} type="file" accept=".3mf,.stl" hidden data-testid="import-input"
          onChange={(event) => {
            const picked = event.target.files?.[0];
            if (picked) void importFile(picked);
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

/** MainFrame notebook: Prepare/Preview tabs, slice and export SideButtons. */
export function TabBar() {
  const { engine, scene, slice } = useApp();
  const slicing = slice.status === "slicing";
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
          disabled={!scene || slicing || engine !== "ready"}>
          {slicing ? "Slicing…" : "Slice plate"}
        </button>
        <button className="side-button" onClick={exportGcode} disabled={slice.status !== "done"}>
          Export G-code file
        </button>
      </div>
    </nav>
  );
}
