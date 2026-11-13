import type { ProjectPresetSelection } from "@ares-slicer/wasm";
import { type CatalogSources, FILAMENT_LIBRARY } from "./presets";

/** The catalog JSON `sliceWithProfiles` takes: the installed vendors and the imported presets. */
export async function catalogJson({ vendors, presetImports }: CatalogSources): Promise<string> {
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  return JSON.stringify({ vendors: loaded, imports: presetImports });
}

/**
 * The "Project presets" selection: the project's own presets, each key it lacks taken from its
 * system preset in the catalog (OrcaSlicer.cpp:3167-3200, PresetBundle::load_config_file_config).
 * A system preset of a vendor that is not installed keeps the project's values.
 */
export const FROM_PROJECT: ProjectPresetSelection = { from_project: true };
