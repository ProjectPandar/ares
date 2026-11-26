import type { FilamentSlotState, ProjectSettings, Scene } from "@ares-slicer/wasm";
import { type PrinterSettings, saveConfig, storedConfig } from "./persistence";
import { type PresetSelection, useApp } from "./store";

/** The loaded project's arrays; omitted fields use the core's PrintConfig defaults. */
export function projectFilamentState(settings: ProjectSettings | null): Partial<FilamentSlotState> {
  const values = { ...settings?.project, ...settings?.filament };
  const state: Partial<FilamentSlotState> = {};
  const entries = (key: string) => {
    const raw = values[key];
    return Array.isArray(raw) ? raw : String(raw).split(/[;,]/);
  };
  for (const key of [
    "flush_multiplier",
    "flush_volumes_vector",
    "flush_volumes_matrix",
    "filament_map",
    "filament_nozzle_map",
    "filament_volume_map",
  ] as const) {
    if (key in values) state[key] = entries(key).map(Number);
  }
  for (const key of ["filament_colour", "filament_multi_colour", "filament_colour_type"] as const) {
    if (key in values) state[key] = entries(key).map(String);
  }
  return state;
}

/**
 * PresetBundle::export_selections (PresetBundle.cpp:4232-4292): the selection, also under its printer with the
 * project palette and flush data.
 */
export function saveSelection() {
  const { presetSelection: selection, scene } = useApp.getState();
  if (!selection) {
    saveConfig({ printer: null });
    return;
  }
  const { printer, process, filaments, bedType } = selection;
  const project = selection.filamentProject ?? projectFilamentState(scene?.settings ?? null);
  const settings = {
    ...storedConfig.printers[printer],
    process,
    filaments: Array.from(filaments, (name) => name ?? ""),
    bedType,
    ...exportedProject(project),
  };
  saveConfig({ printer, printers: { ...storedConfig.printers, [printer]: settings } });
}

/** The palette and flush data export_selections stores (PresetBundle.cpp:4264-4292). */
export function exportedProject(project: Partial<FilamentSlotState>) {
  // std::to_string writes six decimals (4279-4292).
  const toString = (values?: number[]) => values?.map((value) => Number(value.toFixed(6)));
  return {
    filamentColours: project.filament_colour,
    filamentMultiColours: project.filament_multi_colour,
    filamentColourTypes: project.filament_colour_type,
    flushVolumesMatrix: toString(project.flush_volumes_matrix),
    flushVolumesVector: toString(project.flush_volumes_vector),
    flushMultiplier: toString(project.flush_multiplier),
  };
}

/**
 * PresetBundle::update_selections from the selection `saved` stored for a printer (PresetBundle.cpp:3914-4003), which Tab::select_preset runs with
 * the default `remember_printer_config` (Tab.cpp:7740-7742) and load_selections repeats at startup (4082-4164):
 * the print, slots and palette export_selections stored for the printer (4251-4292). A printer without them gets
 * an empty print and slot 1, which update_compatible fills with the printer's preferred presets, and one "#26A69A"
 * colour; the flush data stays the current project's.
 */
export function rememberedSelection(saved: PrinterSettings | undefined) {
  const { presetSelection: current, scene } = useApp.getState();
  const project = current?.filamentProject ?? projectFilamentState(scene?.settings ?? null);
  // 3929-3937: slot 1, then filament_01, filament_02, ...
  const filaments = [saved?.filaments[0] ?? "", ...(saved?.filaments.slice(1) ?? [])];
  const resized = (values: string[], fill: string) =>
    Array.from({ length: filaments.length }, (_, index) => values[index] ?? fill);
  const colours = resized(saved?.filamentColours ?? [], "#26A69A");
  return {
    // 3918-3923: the stored print, strictly selected; "" leaves none (Preset.cpp:4335-4357).
    process: saved?.process ?? "",
    filaments,
    project: {
      flush_volumes_matrix: saved?.flushVolumesMatrix ?? project.flush_volumes_matrix,
      flush_volumes_vector: saved?.flushVolumesVector ?? project.flush_volumes_vector,
      flush_multiplier: saved?.flushMultiplier ?? project.flush_multiplier,
      filament_colour: colours,
      filament_multi_colour: saved?.filamentMultiColours?.length ? saved.filamentMultiColours : colours,
      filament_colour_type: resized(saved?.filamentColourTypes ?? [], "1"),
      // 3963-3970: one filament_map, filament_nozzle_map and filament_volume_map entry per colour.
      filament_map: colours.map(() => 1),
      filament_nozzle_map: colours.map(() => 0),
      filament_volume_map: colours.map(() => 0),
    } satisfies Partial<FilamentSlotState>,
  };
}

/**
 * Why the plate cannot be sliced with the selected presets, or null. An STL
 * is placed by arrange before a preset slice (the CLI's `--arrange`), which
 * is not ported yet.
 */
export function presetSliceBlocker(selection: PresetSelection, scene: Scene): string | null {
  // An STL is centered like one model loaded on an empty plate (Plater.cpp:7149-7160); with more
  // objects OrcaSlicer looks for empty cells instead, which is not ported yet.
  if (scene.format === "stl" && scene.objects.length !== 1 && useApp.getState().file?.kind !== "stl-batch") {
    return "Placing more than one STL object on the plate needs arrange, which is not ported yet";
  }
  if (!selection.process) return "Select a process preset to slice";
  if (selection.filaments.some((name) => !name)) {
    return "Select a filament preset for every filament to slice";
  }
  return null;
}
