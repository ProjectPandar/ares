import type { ParsedPresetBundle, PresetCatalog } from "@ares-slicer/wasm";
import { slicer } from "./slicer";
import { type PresetSelection, useApp } from "./store";

/**
 * PresetBundle::ORCA_FILAMENT_LIBRARY: always installed and loaded before the
 * other vendors (PresetBundle.cpp:2216-2234).
 */
export const FILAMENT_LIBRARY = "OrcaFilamentLibrary";

/** Imported preset files, in import order, for the engine catalog. */
const importFiles: { input: Uint8Array; filename: string }[] = [];

let synced: { vendors: readonly string[]; imports: number; catalog: PresetCatalog } | null = null;
let syncing: Promise<unknown> = Promise.resolve();

/**
 * The engine catalog (PresetBundle) of the installed vendors and imported
 * presets: rebuilt when the vendors change, imports added as they arrive.
 * Syncs run one at a time.
 */
function presetCatalog(): Promise<PresetCatalog> {
  const next = syncing.then(syncCatalog);
  syncing = next.catch(() => undefined);
  return next;
}

async function syncCatalog(): Promise<PresetCatalog> {
  const { vendors } = useApp.getState();
  if (synced?.vendors !== vendors) {
    const start = performance.now();
    void synced?.catalog.free();
    synced = null;
    const { loadVendor } = await import("@ares-slicer/profiles");
    const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
    const catalog = await slicer.createPresetCatalog();
    const encoder = new TextEncoder();
    for (const vendor of loaded) await catalog.addVendor(encoder.encode(JSON.stringify(vendor)));
    performance.measure("ares:catalog-build", { start });
    synced = { vendors, imports: 0, catalog };
  }
  for (const { input, filename } of importFiles.slice(synced.imports)) await synced.catalog.addImport(input, filename);
  synced.imports = importFiles.length;
  return synced.catalog;
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
    const catalog = await presetCatalog();
    const start = performance.now();
    const presets = await catalog
      .filterCompatible(selection.printer, selection.process)
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
    const inputs = await Promise.all(files.map(async (file) => new Uint8Array(await file.arrayBuffer())));
    const parsed = await Promise.all(files.map((file, index) => slicer.parsePresetBundle(inputs[index], file.name)));
    importFiles.push(...files.map((file, index) => ({ input: inputs[index], filename: file.name })));
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
