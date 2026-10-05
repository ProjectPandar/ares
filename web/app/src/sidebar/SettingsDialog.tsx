import type { OptionMap, ProjectSettings } from "@ares-slicer/wasm";
import { useEffect, useRef } from "react";
import type { LayoutGroup, LayoutPage } from "./layoutTypes";
import { ModeSwitch } from "./ModeSwitch";
import { SettingsPages } from "./SettingsPages";
import { FILAMENT_PAGES, PRINTER_PAGES } from "./presetLayout";

export type SettingsTab = { kind: "printer" } | { kind: "filament"; slot: number };

// Tab.cpp:4930-4931 `is_marlin_flavor`, by `GCodeFlavor` serialization (PrintConfig.cpp:164-168).
const MARLIN_FLAVORS = new Set(["marlin", "marlin2", "klipper", "reprapfirmware", "repetier"]);

const atExtruder = (groups: LayoutGroup[], index: number): LayoutGroup[] =>
  groups.map((group) => ({ ...group, rows: group.rows.map((row) => (row.kind === "option" ? { ...row, index } : row)) }));

/**
 * TabPrinter page order after build_unregular_pages (Tab.cpp:4927-5080): "Motion ability"
 * only for Marlin-like flavours, "Multimaterial" in front of one page per extruder.
 */
function printerPages(values: OptionMap): LayoutPage[] {
  const [basic, gcode, multimaterial, extruder, motion, notes] = PRINTER_PAGES;
  const nozzles = values.nozzle_diameter;
  const count = Array.isArray(nozzles) ? nozzles.length : 1;
  const extruders = Array.from({ length: count }, (_, index) => ({
    title: count > 1 ? `Extruder ${index + 1}` : extruder.title,
    groups: atExtruder(extruder.groups, index),
  }));
  return MARLIN_FLAVORS.has(String(values.gcode_flavor))
    ? [basic, gcode, multimaterial, ...extruders, motion, notes]
    : [basic, multimaterial, ...extruders, gcode, notes];
}

/** Read-only TabPrinter / TabFilament of the project's presets (Plater edit buttons). */
export function SettingsDialog({ tab, settings, onClose }: { tab: SettingsTab; settings: ProjectSettings; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => dialog.current!.showModal(), []);
  const printer = tab.kind === "printer";
  const values = { ...settings.project, ...(printer ? settings.printer : settings.filament) };
  const title = printer ? "Printer settings" : "Filament settings";
  const presetKey = printer ? "printer_settings_id" : "filament_settings_id";
  const preset = values[presetKey];
  const name = Array.isArray(preset) ? preset[printer ? 0 : tab.slot] : preset;
  return (
    <dialog ref={dialog} className="settings-dialog" aria-labelledby="settings-title" onClose={onClose}>
      <header className="shortcuts-header">
        <h2 id="settings-title">{title}</h2>
        <span className="settings-preset">{name ? String(name) : ""}</span>
        <ModeSwitch />
        <button className="bar-button" aria-label="Close" onClick={() => dialog.current!.close()}>{"\u2715"}</button>
      </header>
      <div className="settings-body process">
        <SettingsPages pages={printer ? printerPages(values) : FILAMENT_PAGES}
          context={{ values, slot: printer ? undefined : tab.slot }} />
      </div>
    </dialog>
  );
}
