import type { GuideFilament, GuideModel, InstalledPrinters } from "@ares-slicer/wasm";
import { saveConfig, storedConfig } from "./persistence";
import { presetCatalog } from "./catalog";
import { FILAMENT_LIBRARY, selectFilament, selectPrinter, updateInstalledFilaments, updateSystemPrinters } from "./presets";
import { slicer } from "./slicer";
import { installedVendors, useApp } from "./store";

/** GuideFrame::BuildProfileJson's models (WebGuideDialog.cpp:1389-1413): every vendor's printer models. */
export async function guideModels(): Promise<GuideModel[]> {
  const { loadPrinterModels } = await import("@ares-slicer/profiles");
  return slicer.guideModels(new TextEncoder().encode(JSON.stringify(await loadPrinterModels())));
}

/**
 * BuildProfileJson's filaments (WebGuideDialog.cpp:1416-1457) of `vendors` and the filament library,
 * from a catalog of the wizard's own: nothing is applied before the wizard finishes.
 */
export async function guideFilaments(vendors: readonly string[]): Promise<GuideFilament[]> {
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  const catalog = await slicer.createPresetCatalog();
  try {
    const encoder = new TextEncoder();
    for (const vendor of loaded) await catalog.addVendor(encoder.encode(JSON.stringify(vendor)));
    return await catalog.guideFilaments();
  } finally {
    void catalog.free();
  }
}

/**
 * save_userguide_models then SaveProfile (WebGuideDialog.cpp:607-622, 940-968): every guide model
 * whose id is checked is installed with all of its nozzle variants, a sorted set as AppConfig keeps it.
 */
export function installedFromChecked(models: readonly GuideModel[], checked: ReadonlySet<string>): InstalledPrinters {
  const installed: InstalledPrinters = {};
  for (const { vendor, model, variants } of models) {
    if (checked.has(model)) (installed[vendor] ??= {})[model] = [...variants].sort();
  }
  return installed;
}

/** save_userguide_models (WebGuideDialog.cpp:624-655): the checked models' default materials that are listed filaments. */
export function defaultMaterials(models: readonly GuideModel[], checked: ReadonlySet<string>, filaments: readonly GuideFilament[]): string[] {
  const listed = new Set(filaments.map(({ name }) => name));
  return models.filter(({ model }) => checked.has(model)).flatMap((model) => model.default_materials).filter((name) => listed.has(name));
}

/**
 * GuideFrame::apply_config's preferred printer (WebGuideDialog.cpp:1066-1124): the first model, in
 * map order, of the default bundle "Custom" and then the other vendors whose installed variants are
 * new; its first variant in the model's nozzle order, or the first new variant of a model that was
 * installed before.
 */
export function preferredPrinter(installed: InstalledPrinters, previous: InstalledPrinters, models: readonly GuideModel[]): [string, string] | null {
  const preferred = (vendor: string): [string, string] | null => {
    const enabled = installed[vendor];
    if (!enabled) return null;
    for (const model of Object.keys(enabled).sort()) {
      const variants = [...enabled[model]].sort();
      if (variants.length === 0) continue;
      const nozzles = models.find((guide) => guide.vendor === vendor && guide.model === model)!.variants;
      const variant = variants.length > 1 ? nozzles.find((nozzle) => variants.includes(nozzle))! : variants[0];
      const old = previous[vendor]?.[model];
      if (old === undefined) return [model, variant];
      const added = variants.find((nozzle) => !old.includes(nozzle));
      if (added !== undefined) return [model, added];
    }
    return null;
  };
  const DEFAULT_BUNDLE = "Custom";
  for (const vendor of [DEFAULT_BUNDLE, ...Object.keys(installed).sort().filter((id) => id !== DEFAULT_BUNDLE)]) {
    const printer = preferred(vendor);
    if (printer) return printer;
  }
  return null;
}

/**
 * GuideFrame::apply_config into PresetBundle::apply_vendor_config with `overwrite`
 * (PresetBundle.cpp:2050-2095): the installed printers and filaments are stored, the catalog reloads
 * them, load_selections selects the preferred printer, else the stored one, else the first visible
 * printer by name (select_preset_by_name, Preset.cpp:4240-4249), and an active filament that is
 * not one of `filaments` becomes the first of them that is visible and compatible.
 */
export async function finishWizard(installed: InstalledPrinters, filaments: readonly string[], models: readonly GuideModel[]) {
  const preferred = preferredPrinter(installed, useApp.getState().installedPrinters, models);
  saveConfig({ installedPrinters: installed, installedFilaments: [...filaments].sort() });
  const vendors = installedVendors(installed);
  // The catalog is rebuilt only when the installed vendors change.
  if (vendors.join("\0") === useApp.getState().vendors.join("\0")) useApp.setState({ installedPrinters: installed });
  else useApp.setState({ installedPrinters: installed, vendors });
  await Promise.all([updateSystemPrinters(), updateInstalledFilaments()]);
  const catalog = await presetCatalog();
  const visible = (await catalog.printers()).map(({ name }) => name).sort();
  const name = (preferred && await catalog.systemPrinter(...preferred)) ?? storedConfig.printer;
  await selectPrinter(name !== null && visible.includes(name) ? name : visible[0] ?? null);
  const { presetSelection, compatible, installedFilaments } = useApp.getState();
  if (filaments.length > 0 && presetSelection && compatible?.status === "done" && !filaments.includes(presetSelection.filaments[0] ?? "")) {
    const first = [...filaments].sort().find((filament) => installedFilaments.has(filament) && compatible.presets.filaments.includes(filament));
    if (first !== undefined) selectFilament(0, first);
  }
  useApp.setState({ wizard: null });
}
