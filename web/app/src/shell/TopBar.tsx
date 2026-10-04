import { Link } from "@tanstack/react-router";
import { useRef } from "react";
import { exportGcode, importFile, setTheme, slicePlate } from "../state/actions";
import { useApp } from "../state/store";
import { ExportIcon, ImportIcon, MoonIcon, SliceIcon, SunIcon } from "./icons";

export function TopBar() {
  const input = useRef<HTMLInputElement>(null);
  const { theme, engine, scene, slice, loadingFile } = useApp();
  const slicing = slice.status === "slicing";
  return (
    <header className="topbar">
      <div className="topbar-left">
        <span className="brand" aria-label="Ares">
          <span className="brand-mark">A</span>
          Ares
        </span>
        <button className="tool-button" title="Import 3MF/STL (Ctrl+I)" onClick={() => input.current?.click()}
          disabled={engine !== "ready" || loadingFile}>
          <ImportIcon /> <span>Import</span>
        </button>
        <button className="tool-button" title="Export G-code" onClick={exportGcode}
          disabled={slice.status !== "done"}>
          <ExportIcon /> <span>Export G-code</span>
        </button>
        <input ref={input} type="file" accept=".3mf,.stl" hidden data-testid="import-input"
          onChange={(event) => {
            const file = event.target.files?.[0];
            if (file) void importFile(file);
            event.target.value = "";
          }} />
      </div>
      <nav className="topbar-tabs">
        <Link to="/prepare" className="tab" activeProps={{ className: "tab active" }}>Prepare</Link>
        <Link to="/preview" className="tab" activeProps={{ className: "tab active" }}>Preview</Link>
      </nav>
      <div className="topbar-right">
        <button className="icon-button" title={theme === "dark" ? "Light theme" : "Dark theme"}
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
          {theme === "dark" ? <SunIcon /> : <MoonIcon />}
        </button>
        <button className="primary-button" onClick={() => void slicePlate()}
          disabled={!scene || slicing || engine !== "ready"}>
          <SliceIcon /> {slicing ? "Slicing…" : "Slice plate"}
        </button>
      </div>
    </header>
  );
}
