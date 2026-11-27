import { Link, useNavigate } from "@tanstack/react-router";
import { type RefObject, useEffect, useRef, useState } from "react";
import {
  exportAllGcode,
  exportGcode,
  importFile,
  importFiles,
  setTheme,
  sliceAllPlates,
  slicePlate,
} from "../state/actions";
import { importPresetFiles, presetSliceBlocker } from "../state/presets";
import { useApp } from "../state/store";
import { CloudMenu } from "./CloudMenu";
import { FileMenu } from "./FileMenu";
import { MoonIcon, OrcaIcon, SunIcon } from "./icons";
import { SplitAction, type ToolbarAction } from "./SplitAction";

/** BBLTopbar: the File and Cloud menus and the project title. */
export function TitleBar({
  importInput: input,
  projectInput,
  onShortcuts,
  onAbout,
  onSignIn,
}: {
  importInput: RefObject<HTMLInputElement | null>;
  projectInput: RefObject<HTMLInputElement | null>;
  onShortcuts: () => void;
  onAbout: () => void;
  onSignIn: () => void;
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
        <CloudMenu onSignIn={onSignIn} />
        <input
          ref={input}
          type="file"
          accept=".3mf,.stl"
          multiple
          hidden
          data-testid="import-input"
          onChange={(event) => {
            importFiles([...(event.target.files ?? [])]);
            event.target.value = "";
          }}
        />
        {/* Plater::load_project (File > Open Project). */}
        <input
          ref={projectInput}
          type="file"
          accept=".3mf"
          hidden
          data-testid="project-input"
          onChange={(event) => {
            const picked = event.target.files?.[0];
            if (picked) void importFile(picked);
            event.target.value = "";
          }}
        />
        {/* MainFrame::load_config_file (File > Import > Import Configs). */}
        <input
          ref={presetInput}
          type="file"
          accept=".json,.zip,.orca_printer,.orca_bundle,.orca_filament"
          multiple
          hidden
          data-testid="preset-import-input"
          onChange={(event) => {
            const picked = [...(event.target.files ?? [])];
            if (picked.length) void importPresetFiles(picked);
            event.target.value = "";
          }}
        />
      </div>
      <span className="titlebar-title">{file ? `${file.name} - Ares` : "Ares"}</span>
      <div className="titlebar-right">
        <HelpMenu onShortcuts={onShortcuts} onAbout={onAbout} />
        <button
          className="bar-button"
          title={theme === "dark" ? "Light theme" : "Dark theme"}
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
        >
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
      if (event instanceof KeyboardEvent ? event.key === "Escape" : !root.current?.contains(event.target as Node))
        setOpen(false);
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
      <button className="bar-button" aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen(!open)}>
        Help
      </button>
      {open && (
        <div className="context-menu" role="menu" aria-label="Help">
          <button type="button" role="menuitem" onClick={pick(onShortcuts)}>
            Keyboard Shortcuts
          </button>
          <button type="button" role="menuitem" onClick={pick(onAbout)}>
            About Ares
          </button>
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
export function TabBar({ onPrint }: { onPrint: () => void }) {
  const { engine, scene, slice, presetSelection, plateSlices, cloud } = useApp();
  const slicing = slice.status === "slicing";
  const blocker = scene && presetSelection ? presetSliceBlocker(presetSelection, scene) : null;
  const multiPlate = (scene?.plates.length ?? 0) > 1;
  const navigate = useNavigate();
  // MainFrame.cpp:2411-2415: the button posts the slice event, then selects Preview at once.
  const sliceAndPreview = (start: () => Promise<void>) => {
    void start();
    void navigate({ to: "/preview" });
  };
  const sliceDisabled = !scene || slicing || engine !== "ready" || blocker !== null;
  const outputActions: [ToolbarAction, ...ToolbarAction[]] = [
    {
      label: "Export G-code file",
      run: exportGcode,
      disabled: slice.status !== "done",
    },
    { label: "Print plate", run: onPrint, disabled: slice.status !== "done" },
  ];
  if (multiPlate) {
    outputActions.push({
      label: "Export all",
      run: exportAllGcode,
      disabled: slicing || Object.keys(plateSlices).length === 0,
    });
  }
  return (
    <nav className="tabbar">
      <div className="tabbar-tabs">
        <Link to="/prepare" className="tab" activeProps={{ className: "tab active" }}>
          <OrcaIcon name="tab_3d_active" size={20} /> Prepare
        </Link>
        <Link to="/preview" className="tab" activeProps={{ className: "tab active" }}>
          <OrcaIcon name="tab_preview_active" size={20} /> Preview
        </Link>
        {/* MainFrame::show_device (MainFrame.cpp:1661-1745): the Device tab after Preview, while signed in. */}
        {cloud && (
          <Link to="/devices" className="tab" activeProps={{ className: "tab active" }}>
            <OrcaIcon name="tab_monitor_active" size={20} /> Devices
          </Link>
        )}
      </div>
      <div className="tabbar-actions">
        {multiPlate ? (
          <SplitAction
            label="Slicing"
            primary
            busy={slicing}
            title={blocker ?? undefined}
            actions={[
              {
                label: "Slice plate",
                run: () => sliceAndPreview(slicePlate),
                disabled: sliceDisabled,
              },
              {
                label: "Slice all",
                run: () => sliceAndPreview(sliceAllPlates),
                disabled: sliceDisabled,
              },
            ]}
          />
        ) : (
          <button
            type="button"
            className="side-button slice-action"
            onClick={() => sliceAndPreview(slicePlate)}
            disabled={sliceDisabled}
            title={blocker ?? undefined}
          >
            {slicing ? "Slicing…" : "Slice plate"}
          </button>
        )}
        <SplitAction label="Output" actions={outputActions} />
      </div>
    </nav>
  );
}
