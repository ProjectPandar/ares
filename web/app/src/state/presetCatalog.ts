import type { InstalledPrinters, ParsedPresetBundle, PlaterSession, PresetCatalog } from "@ares-slicer/wasm";
import type { DialogFilament } from "../sidebar/filamentDialog";
import { firstString } from "../sidebar/options";
import { saveConfig, storedConfig } from "./persistence";
import { catalogJson, FILAMENT_LIBRARY } from "./projectPresets";
import { slicer } from "./slicer";
import { useApp } from "./store";

/** Imported preset files, in import order, for the engine catalog. */
const importFiles: { input: Uint8Array; filename: string }[] = [];

let synced: {
  vendors: readonly string[];
  installed: InstalledPrinters | null;
  imports: number;
  catalog: PresetCatalog;
} | null = null;
let syncing: Promise<unknown> = Promise.resolve();

export function addImportFiles(files: readonly { input: Uint8Array; filename: string }[]) {
  importFiles.push(...files);
}

/**
 * The engine catalog (PresetBundle) of the installed vendors and imported
 * presets: rebuilt when the vendors change, imports added as they arrive, and
 * its printers' visibility set from the installed printers
 * (PresetBundle::load_installed_printers). Syncs run one at a time.
 */
export function presetCatalog(): Promise<PresetCatalog> {
  const next = syncing.then(syncCatalog);
  syncing = next.catch(() => undefined);
  return next;
}

async function syncCatalog(): Promise<PresetCatalog> {
  const { vendors, installedPrinters } = useApp.getState();
  const changed = synced?.vendors !== vendors || synced.imports !== importFiles.length;
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
    synced = { vendors, installed: null, imports: 0, catalog };
    // The installed vendors changed: the plater's vendor test reads the new set.
    await useApp.getState().plater?.setInstalledVendors(catalog);
  }
  if (synced.installed !== installedPrinters) {
    await synced.catalog.setInstalledPrinters(installedPrinters);
    synced.installed = installedPrinters;
  }
  for (const { input, filename } of importFiles.slice(synced.imports)) await synced.catalog.addImport(input, filename);
  synced.imports = importFiles.length;
  // PresetBundle::load_presets at startup (GUI_App.cpp:3723) and after the wizard (ConfigWizard.cpp:2597):
  // the project load that follows selects from these presets instead of loading them again.
  if (changed) await slicer.loadPresetBundle(await catalogJson(useApp.getState()));
  return synced.catalog;
}

/**
 * Hands a new plater the installed system vendors (`PresetBundle::vendors`), which the arrange's
 * vendor test reads; `syncCatalog` hands them again when the installed vendors change.
 */
export async function installVendors(plater: PlaterSession) {
  await plater.setInstalledVendors(await presetCatalog());
}

/** The installed printers and the printer models of the printer combo's "System presets" group. */
export async function updateSystemPrinters() {
  const { installedPrinters } = useApp.getState();
  try {
    const catalog = await presetCatalog();
    const [printers, models] = await Promise.all([catalog.printers(), catalog.printerModels()]);
    if (useApp.getState().installedPrinters === installedPrinters) {
      useApp.setState({
        systemPrinters: printers.filter((printer) => printer.vendor !== "user"),
        printerModels: models.map(({ model }) => model),
      });
    }
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
  }
}

/** PresetBundle::load_installed_filaments (PresetBundle.cpp:3757-3816) over the stored set. */
export async function updateInstalledFilaments() {
  const { installedPrinters } = useApp.getState();
  const installed = await (await presetCatalog()).installedFilaments(storedConfig.installedFilaments);
  if (useApp.getState().installedPrinters !== installedPrinters) return;
  saveConfig({ installedFilaments: installed });
  useApp.setState({ installedFilaments: new Set(installed) });
}

/**
 * The system filaments of the filament dialog for the selected printer, with their type and vendor
 * (GuideFrame::GetFilamentInfo, WebGuideDialog.cpp:1035): the compatible ones (common.js:86-117).
 */
export async function dialogFilaments(printer: string): Promise<DialogFilament[]> {
  const catalog = await presetCatalog();
  const user = importedNames(useApp.getState().presetImports, "filament");
  const names = (await catalog.filterCompatible(printer)).filaments.filter((name) => !user.has(name));
  const values = await catalog.presetValues("filament", names, ["filament_type", "filament_vendor"]);
  return names.map((name, index) => ({
    name,
    type: firstString(values[index], "filament_type") ?? "",
    vendor: firstString(values[index], "filament_vendor") ?? "",
  }));
}

/** The filament dialog's result (GuideFrame::SaveProfile): the new installed set. */
export function setInstalledFilaments(installed: string[]) {
  saveConfig({ installedFilaments: [...installed].sort() });
  useApp.setState({ installedFilaments: new Set(installed) });
}

/** Names of the imported presets of one `type`, for the "User presets" group. */
export function importedNames(
  imports: readonly ParsedPresetBundle[],
  type: "machine" | "process" | "filament",
): Set<string> {
  return new Set(
    imports.flatMap(({ profiles }) =>
      profiles.filter((profile) => profile.type === type).map((profile) => profile.name as string),
    ),
  );
}
