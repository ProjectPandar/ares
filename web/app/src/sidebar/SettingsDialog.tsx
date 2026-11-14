import type { OptionMap, ProjectSettings } from "@ares-slicer/wasm";
import { useEffect, useRef, useState } from "react";
import { presetCatalog } from "../state/catalog";
import type { LayoutGroup, LayoutPage } from "./layoutTypes";
import { useApp } from "../state/store";
import { decodeOptions, tabValues } from "../state/typedSettings";
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

/** Plater.cpp:3752-3758 opens the toolchanger's selected filament tab directly;
 * Tab.cpp:7233-7267 loads its edited preset, independently of a project file. */
export function SettingsDialog({ tab, settings, onClose }: { tab: SettingsTab; settings: ProjectSettings | null; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => dialog.current!.showModal(), []);
  const printer = tab.kind === "printer";
  const typed = useApp((state) => state.typedSettings);
  const selection = useApp((state) => printer ? null : state.presetSelection);
  const defs = useApp((state) => state.configDefs)!;
  const [fetched, setFetched] = useState<{ selection: typeof selection; values: OptionMap } | null>(null);
  useEffect(() => {
    if (!selection) return;
    let stale = false;
    const pages = FILAMENT_PAGES;
    const keys = [...new Set(pages.flatMap((page) => page.groups.flatMap((group) => group.rows.flatMap((row) =>
      row.kind === "line" ? row.options.map((option) => option.key) : row.kind === "separator" ? [] : [row.key]))))];
    void presetCatalog().then(async (catalog) => {
      const presets = await catalog.presetValues("filament", selection.filaments, keys);
      const values = { ...presets[0] };
      for (const key of keys) {
        if (/s(?:_nullable)?$/.test(defs.get(key)!.type)) values[key] = presets.map((preset) => {
          const value = preset[key];
          return Array.isArray(value) ? value[0] : value;
        });
      }
      const decoded = await decodeOptions(defs, values);
      if (!stale) setFetched({ selection, values: { ...values, ...decoded } });
    });
    return () => { stale = true; };
  }, [selection, defs]);
  const values = selection ? (fetched?.selection === selection ? fetched.values : null)
    : settings && typed && tabValues(settings, typed, printer ? "printer" : "filament");
  const title = printer ? "Printer settings" : "Filament settings";
  const presetKey = printer ? "printer_settings_id" : "filament_settings_id";
  const preset = selection ? (printer ? selection.printer : selection.filaments) : values?.[presetKey];
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
        <SettingsPages pages={printer && values ? printerPages(values) : printer ? PRINTER_PAGES : FILAMENT_PAGES}
          context={values && { values, slot: printer ? undefined : tab.slot }} />
      </div>
    </dialog>
  );
}
