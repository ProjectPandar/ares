import type { ParsedPresetBundle } from "@ares-slicer/wasm";
import { slicer } from "./slicer";
import { type PresetSelection, useApp } from "./store";

/**
 * PresetBundle::ORCA_FILAMENT_LIBRARY: always installed and loaded before the
 * other vendors (PresetBundle.cpp:2216-2234).
 */
export const FILAMENT_LIBRARY = "OrcaFilamentLibrary";

let catalog: { vendors: readonly string[]; imports: readonly ParsedPresetBundle[]; json: string } | null = null;

/**
 * The engine catalog of the installed vendors and imported presets, rebuilt
 * only when either changes: stringifying a large vendor costs more than the
 * filter itself.
 */
async function catalogJson(): Promise<string> {
  const { vendors, presetImports } = useApp.getState();
  if (catalog?.vendors === vendors && catalog.imports === presetImports) return catalog.json;
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  const start = performance.now();
  const json = JSON.stringify({ vendors: loaded, imports: presetImports });
  performance.measure("ares:catalog-stringify", { start });
  catalog = { vendors, imports: presetImports, json };
  return json;
}

/** PresetBundle::update_compatible for the selected printer and process. */
async function updateCompatible() {
  const selection = useApp.getState().presetSelection;
  if (!selection) {
    useApp.setState({ compatible: null });
    return;
  }
  useApp.setState({ compatible: { status: "filtering" } });
  try {
    const json = await catalogJson();
    const start = performance.now();
    const presets = await slicer
      .filterCompatiblePresets(json, selection.printer, selection.process)
      .finally(() => performance.measure("ares:filter-compatible-presets", { start }));
    if (useApp.getState().presetSelection === selection) useApp.setState({ compatible: { status: "done", presets } });
  } catch (error) {
    if (useApp.getState().presetSelection === selection) {
      useApp.setState({ compatible: { status: "error", error: (error as Error).message } });
    }
  }
}

/** MainFrame::load_config_file: adds the files' presets as user presets. */
export async function importPresetFiles(files: File[]) {
  try {
    const parsed = await Promise.all(
      files.map(async (file) => slicer.parsePresetBundle(new Uint8Array(await file.arrayBuffer()), file.name)),
    );
    useApp.setState(({ presetImports }) => ({ presetImports: [...presetImports, ...parsed], loadError: undefined }));
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
    return;
  }
  await updateCompatible();
}

/** ConfigWizard: the installed system vendors. */
export async function setVendors(vendors: string[]) {
  localStorage.setItem("ares.vendors", JSON.stringify(vendors));
  useApp.setState({ vendors });
  await updateCompatible();
}

/** Selects a user or system printer; `null` returns to the project's presets. */
export async function selectPrinter(printer: string | null) {
  useApp.setState({ presetSelection: printer === null ? null : { printer, filaments: [] } });
  await updateCompatible();
}

/** Selects a process of the compatible list; filaments are filtered against it. */
export async function selectProcess(process: string) {
  const selection = useApp.getState().presetSelection!;
  useApp.setState({ presetSelection: { ...selection, process } });
  await updateCompatible();
}

export function selectFilament(slot: number, filament: string) {
  const selection: PresetSelection = useApp.getState().presetSelection!;
  const filaments = [...selection.filaments];
  filaments[slot] = filament;
  useApp.setState({ presetSelection: { ...selection, filaments } });
}

/** Names of the imported presets of one `type`, for the "User presets" group. */
export function importedNames(imports: readonly ParsedPresetBundle[], type: "machine" | "process" | "filament"): Set<string> {
  return new Set(
    imports.flatMap(({ profiles }) =>
      profiles.filter((profile) => profile.type === type).map((profile) => profile.name as string)),
  );
}
