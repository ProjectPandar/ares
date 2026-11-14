import type { ProjectPresetSelection } from "@ares-slicer/wasm";
import { type CatalogSources, FILAMENT_LIBRARY } from "./presets";

/** The catalog JSON `sliceWithProfiles` takes: the installed vendors and the imported presets. */
export async function catalogJson({ vendors, presetImports }: CatalogSources): Promise<string> {
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  return JSON.stringify({ vendors: loaded, imports: presetImports });
}

/** The "Project presets" selection: the presets the plater selects when it opens the project. */
export const FROM_PROJECT: ProjectPresetSelection = { from_project: true, load_and_select: "always" };
