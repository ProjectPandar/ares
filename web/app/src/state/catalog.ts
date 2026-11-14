import type { InstalledPrinters, PlaterSession, PresetCatalog } from "@ares-slicer/wasm";
import { FILAMENT_LIBRARY } from "./presets";
import { catalogJson } from "./projectPresets";
import { slicer } from "./slicer";
import { useApp } from "./store";

/** Imported preset files, in import order, for the engine catalog. */
export const importFiles: { input: Uint8Array; filename: string }[] = [];

let synced: { vendors: readonly string[]; installed: InstalledPrinters | null; imports: number; catalog: PresetCatalog } | null = null;
let syncing: Promise<unknown> = Promise.resolve();

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
