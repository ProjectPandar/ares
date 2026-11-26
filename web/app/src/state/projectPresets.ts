import type { ProjectPresetSelection } from "@ares-slicer/wasm";
import type { CatalogSources } from "./presetSlice";

/**
 * PresetBundle::ORCA_FILAMENT_LIBRARY: always installed and loaded before the
 * other vendors (PresetBundle.cpp:2216-2234).
 */
export const FILAMENT_LIBRARY = "OrcaFilamentLibrary";

let cachedVendors: readonly string[] | undefined;
let cachedImports: CatalogSources["presetImports"] | undefined;
let cachedPromise: Promise<string> | undefined;

function sameList<T>(a: readonly T[] | undefined, b: readonly T[]): boolean {
  return a !== undefined && a.length === b.length && a.every((item, index) => item === b[index]);
}

/** The catalog JSON `sliceWithProfiles` takes: the installed vendors and the imported presets. */
export function catalogJson({ vendors, presetImports }: CatalogSources): Promise<string> {
  if (cachedPromise && sameList(cachedVendors, vendors) && sameList(cachedImports, presetImports)) {
    return cachedPromise;
  }
  const vendorsSnapshot = [...vendors];
  const importsSnapshot = [...presetImports];
  cachedVendors = vendorsSnapshot;
  cachedImports = importsSnapshot;
  const pending = (async () => {
    const { loadVendor } = await import("@ares-slicer/profiles");
    const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendorsSnapshot].map(loadVendor));
    return JSON.stringify({ vendors: loaded, imports: importsSnapshot });
  })();
  cachedPromise = pending;
  void pending.catch(() => {
    if (cachedPromise === pending) cachedPromise = undefined;
  });
  return pending;
}

/** The "Project presets" selection: the presets the plater selects when it opens the project. */
export const FROM_PROJECT: ProjectPresetSelection = { from_project: true, load_and_select: "always" };
