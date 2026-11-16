/**
 * The dual-extruder sidebar (Sidebar::priv::layout_printer, Plater.cpp:584-660): per-nozzle
 * diameter and volume type, and the filament-to-nozzle map (FilamentGroupPopup, FilamentMapDialog).
 */
import { filamentNames, firstString, stringList } from "../sidebar/options";
import { setOverride } from "./actions";
import { saveConfig, storedConfig } from "./persistence";
import { saveSelection } from "./presets";
import type { PrinterPanel } from "./printerPanel";
import { invalidatedSlices, type PresetSelection, useApp } from "./store";

/** FilamentMapMode values (PrintConfig.cpp:579-583). */
export const MAP_MODES = ["Auto For Flush", "Auto For Match", "Manual"] as const;
export type MapMode = (typeof MAP_MODES)[number];

/** `isBBL && is_dual_extruder` (Plater.cpp:2645-2648): the Left / Right Nozzle groups replace the nozzle combo. */
export const isDual = (panel: PrinterPanel | null): panel is PrinterPanel => panel !== null && panel.bbl && panel.nozzleDiameters.length === 2;

/**
 * Volume types the extruder offers: those whose get_extruder_variant_string (PrintConfig.cpp:588-606),
 * `extruder_type + " " + nozzle_volume_type`, is in its `extruder_variant_list` entry.
 */
export function volumeTypeOptions(panel: PrinterPanel, extruder: number, volumeTypes: readonly string[]): string[] {
  const variants = (panel.extruderVariants[extruder] ?? "").split(",");
  return volumeTypes.filter((type) => variants.includes(`${panel.extruderTypes[extruder]} ${type}`));
}

export interface ExtruderState {
  volumeTypes: string[];
  mode: MapMode;
  /** `filament_map` by filament: 1 left, 2 right (FilamentMapPanel.cpp:42-50). */
  map: string[];
}

/** The project config values of the selection, or of the project with its edits. */
export function extruderState(): ExtruderState {
  const { presetSelection: selection, scene, overrides } = useApp.getState();
  const slots = selection ? selection.filaments.length : filamentNames(scene?.settings ?? null).length;
  const project = scene?.settings?.project;
  const edited = (key: string) => overrides[key] as string | string[] | undefined;
  const volumeTypes = selection?.nozzleVolumeTypes ?? (edited("nozzle_volume_type") as string[] | undefined) ?? stringList(project, "nozzle_volume_type");
  const mode = (selection?.filamentMapMode ?? edited("filament_map_mode") ?? firstString(project, "filament_map_mode") ?? "Auto For Flush") as MapMode;
  const map = selection?.filamentMap ?? (edited("filament_map") as string[] | undefined) ?? stringList(project, "filament_map");
  // Plater.cpp:17876-17877: a map shorter than the filaments is filled with the left nozzle.
  return { volumeTypes, mode, map: Array.from({ length: slots }, (_, index) => map[index] ?? "1") };
}

/**
 * The extruder fields of a new preset selection: the printer's stored volume types
 * (AppConfig::get_nozzle_volume_types_from_config) or its `default_nozzle_volume_type`
 * (PresetBundle.cpp:2880-2900), and the preferred map mode (FilamentGroupPopup.cpp:25-36, :199).
 */
export function selectionExtruders(printer: string, panel: PrinterPanel | null): Partial<PresetSelection> {
  if (!isDual(panel)) return { nozzleVolumeTypes: undefined, filamentMapMode: undefined, filamentMap: undefined };
  return {
    nozzleVolumeTypes: storedConfig.nozzleVolumeTypes[printer] ?? panel.defaultNozzleVolumeTypes,
    filamentMapMode: storedConfig.preferredFilamentMapMode,
    filamentMap: undefined,
  };
}

function update(patch: Partial<PresetSelection>, edits: Record<string, string | string[]>) {
  const { presetSelection: selection, scene } = useApp.getState();
  if (!selection) {
    const project = scene?.settings?.project;
    for (const [key, value] of Object.entries(edits)) setOverride(key, value, project?.[key]);
    return;
  }
  useApp.setState(({ slice }) => ({
    presetSelection: { ...selection, ...patch },
    ...invalidatedSlices(slice),
  }));
  saveSelection();
}

/** TabPrinter::set_extruder_volume_type (Tab.cpp:7121-7138): stored for the printer. */
export function setVolumeType(extruder: number, type: string) {
  const volumeTypes = [...extruderState().volumeTypes];
  volumeTypes[extruder] = type;
  const printer = useApp.getState().presetSelection?.printer;
  if (printer) saveConfig({ nozzleVolumeTypes: { ...storedConfig.nozzleVolumeTypes, [printer]: volumeTypes } });
  update({ nozzleVolumeTypes: volumeTypes }, { nozzle_volume_type: volumeTypes });
}

/** Plater::set_global_filament_map_mode (Plater.cpp:16949-16957). Auto For Match needs a connected printer. */
export function setFilamentMapMode(mode: MapMode) {
  saveConfig({ preferredFilamentMapMode: mode });
  update({ filamentMapMode: mode }, { filament_map_mode: mode });
}

/** FilamentMapManualPanel: a filament dragged to the other nozzle (FilamentMapPanel.cpp:81-97 swaps all). */
export function moveFilament(filament: number) {
  const map = extruderState().map.map((value, index) => (index === filament ? (value === "1" ? "2" : "1") : value));
  update({ filamentMap: map }, { filament_map: map });
}

/** The preset slice's project config values of the extruders, sized to `filaments`. */
export function extruderOverrides(selection: PresetSelection, filaments: number): Record<string, string | string[]> {
  if (selection.nozzleVolumeTypes === undefined) return {};
  return {
    nozzle_volume_type: selection.nozzleVolumeTypes,
    filament_map_mode: selection.filamentMapMode!,
    filament_map: Array.from({ length: filaments }, (_, index) => selection.filamentMap?.[index] ?? "1"),
  };
}
