import type { ProjectPresetSelection } from "@ares-slicer/wasm";

/**
 * PresetBundle::ORCA_FILAMENT_LIBRARY: always installed and loaded before the
 * other vendors (PresetBundle.cpp:2216-2234).
 */
export const FILAMENT_LIBRARY = "OrcaFilamentLibrary";

/** The "Project presets" selection: the presets the plater selects when it opens the project. */
export const FROM_PROJECT: ProjectPresetSelection = { from_project: true, load_and_select: "always" };
