import type { ConfigOverrides, CurrentPrinter, OptionMap, OptionValue, ParsedPresetBundle, PresetCatalog, ProfileSliceOptions, Scene, SliceResult } from "@ares-slicer/wasm";
import type { DialogFilament } from "../sidebar/filamentDialog";
import { firstString, presetFilamentSlots } from "../sidebar/options";
import { deleteImport, loadImports, saveConfig, saveImports, storedConfig } from "./persistence";
import { extruderOverrides, selectionExtruders } from "./extruders";
import { printerBedType, updatePrinterPanel } from "./printer";
import { slicer } from "./slicer";
import { type CompatibleState, type ModelFile, type PresetSelection, useApp } from "./store";
import { decodeOptions } from "./typedSettings";

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
export function presetCatalog(): Promise<PresetCatalog> {
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

/** The installed vendors' printers and the printer models of the printer combo's "System presets" group. */
async function updateSystemPrinters() {
  const { vendors } = useApp.getState();
  try {
    const catalog = await presetCatalog();
    const [printers, models] = await Promise.all([catalog.printers(), catalog.printerModels()]);
    if (useApp.getState().vendors === vendors) {
      useApp.setState({
        systemPrinters: printers.filter((printer) => printer.vendor !== "user"),
        printerModels: models.map(({ model }) => model),
      });
    }
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
  }
}

/**
 * GUI_App startup (PresetBundle::load_selections, PresetBundle.cpp:2723): the stored
 * imports join the catalog and the stored printer is selected again.
 */
async function restoreSession() {
  const stored = await loadImports();
  for (const { key, file } of stored) {
    try {
      const parsed = await slicer.parsePresetBundle(file.bytes, file.filename);
      importFiles.push({ input: file.bytes, filename: file.filename });
      useApp.setState(({ presetImports }) => ({ presetImports: [...presetImports, parsed] }));
    } catch {
      await deleteImport(key);
    }
  }
  if (useApp.getState().vendors.length > 0) await updateSystemPrinters();
  await updateInstalledFilaments();
  const { printer } = storedConfig;
  if (printer === null) return;
  // select_preset_by_name_strict: a printer that is no longer loaded is not selected.
  const printers = await (await presetCatalog()).printers();
  if (printers.some(({ name }) => name === printer)) await selectPrinter(printer);
  else saveConfig({ printer: null });
}

void restoreSession();

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
    // PresetComboBoxes.cpp:1216-1226: each filament's type and vendor for the combo's grouping.
    const values = await catalog.presetValues("filament", presets.filaments, ["filament_type", "filament_vendor"]);
    const filamentInfo = new Map(presets.filaments.map((name, index) => [name, {
      type: firstString(values[index], "filament_type") ?? "",
      vendor: firstString(values[index], "filament_vendor") ?? "",
    }]));
    if (useApp.getState().presetSelection === selection) useApp.setState({ compatible: { status: "done", presets, filamentInfo } });
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
    await saveImports(files.map((file, index) => ({ filename: file.name, bytes: inputs[index] })));
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
    return;
  }
  await updateCompatible();
}

/** ConfigWizard: the installed system vendors. */
export async function setVendors(vendors: string[]) {
  saveConfig({ vendors });
  useApp.setState({ vendors });
  await Promise.all([updateSystemPrinters(), updateInstalledFilaments(), updateCompatible(), updatePrinterPanel()]);
}

/** PresetBundle::load_installed_filaments (PresetBundle.cpp:2541-2600) over the stored set. */
async function updateInstalledFilaments() {
  const { vendors } = useApp.getState();
  const installed = await (await presetCatalog()).installedFilaments(storedConfig.installedFilaments);
  if (useApp.getState().vendors !== vendors) return;
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

/** PresetBundle::export_selections (PresetBundle.cpp:2917-2945): the selection, also under its printer. */
export function saveSelection() {
  const selection = useApp.getState().presetSelection;
  if (!selection) {
    saveConfig({ printer: null });
    return;
  }
  const { printer, process, filaments, bedType } = selection;
  const settings = { ...storedConfig.printers[printer], process, filaments: Array.from(filaments, (name) => name ?? ""), bedType };
  saveConfig({ printer, printers: { ...storedConfig.printers, [printer]: settings } });
}

/**
 * Selects a user or system printer; `null` returns to the project's presets. The printer's
 * last process and filaments come back (`remember_printer_config`, Tab.cpp:6229-6233,
 * PresetBundle::update_selections, PresetBundle.cpp:2625-2650), each only while it is still
 * loaded and compatible (select_preset_by_name_strict, then update_compatible).
 */
export async function selectPrinter(printer: string | null) {
  const saved = printer === null ? undefined : storedConfig.printers[printer];
  useApp.setState({ presetSelection: printer === null ? null : { printer, filaments: saved?.filaments ?? [] } });
  await updateCompatible();
  const restored = (selection: PresetSelection | null, compatible: CompatibleState | null) =>
    selection?.printer === printer && compatible?.status === "done" ? { selection, presets: compatible.presets } : null;
  let state = restored(useApp.getState().presetSelection, useApp.getState().compatible);
  if (state && saved?.process && state.presets.processes.includes(saved.process)) {
    useApp.setState({ presetSelection: { ...state.selection, process: saved.process } });
    await updateCompatible();
    state = restored(useApp.getState().presetSelection, useApp.getState().compatible);
  }
  if (state) {
    const filaments = state.selection.filaments.map((name) => (state.presets.filaments.includes(name) ? name : ""));
    useApp.setState({ presetSelection: { ...state.selection, filaments } });
  }
  await updatePrinterPanel();
  const selection = useApp.getState().presetSelection;
  if (selection && selection.printer === printer) {
    useApp.setState({
      presetSelection: { ...selection, bedType: printerBedType(saved?.bedType), ...selectionExtruders(printer, useApp.getState().printerPanel) },
    });
  }
  if ((useApp.getState().presetSelection?.printer ?? null) === printer) saveSelection();
}

/**
 * Selects the printer a printer model item stands for, resolved from the `current` printer
 * (Plater::priv::on_select_preset, Plater.cpp:9565-9572).
 */
export async function selectPrinterModel(model: string, current: CurrentPrinter) {
  try {
    const printer = await (await presetCatalog()).similarPrinterPreset(model, "", current);
    if (printer === null) {
      useApp.setState({ compatible: { status: "error", error: `No printer preset of model "${model}" for "${current.name}"` } });
      return;
    }
    await selectPrinter(printer);
  } catch (error) {
    useApp.setState({ compatible: { status: "error", error: (error as Error).message } });
  }
}

/** Selects a process of the compatible list; filaments are filtered against it. */
export async function selectProcess(process: string) {
  const selection = useApp.getState().presetSelection!;
  useApp.setState({ presetSelection: { ...selection, process } });
  saveSelection();
  await updateCompatible();
}

export function selectFilament(slot: number, filament: string) {
  const selection: PresetSelection = useApp.getState().presetSelection!;
  const filaments = [...selection.filaments];
  filaments[slot] = filament;
  useApp.setState({ presetSelection: { ...selection, filaments } });
  saveSelection();
}

/**
 * Why the plate cannot be sliced with the selected presets, or null. An STL
 * is placed by arrange before a preset slice (the CLI's `--arrange`), which
 * is not ported yet.
 */
export function presetSliceBlocker(selection: PresetSelection, scene: Scene): string | null {
  // An STL is centered like one model loaded on an empty plate (Plater.cpp:7149-7160); with more
  // objects OrcaSlicer looks for empty cells instead, which is not ported yet.
  if (scene.format === "stl" && scene.objects.length !== 1) {
    return "Placing more than one STL object on the plate needs arrange, which is not ported yet";
  }
  if (!selection.process) return "Select a process preset to slice";
  if (Array.from({ length: presetFilamentSlots(scene.settings) }).some((_, slot) => !selection.filaments[slot])) {
    return "Select a filament preset for every filament to slice";
  }
  return null;
}

/** Installed vendors and imported presets of the engine catalog. */
export type CatalogSources = { vendors: readonly string[]; presetImports: readonly ParsedPresetBundle[] };

/** The `sliceWithProfiles` / `profileSettings` arguments of a complete selection. */
export async function profileRequest(scene: Scene, { printer, process, filaments }: PresetSelection, { vendors, presetImports }: CatalogSources, plate = scene.plates[0]?.id) {
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  const catalog = JSON.stringify({ vendors: loaded, imports: presetImports });
  const options: ProfileSliceOptions = {
    plate,
    seed: 0,
    placement: scene.format === "stl" ? "center" : "none",
  };
  return { catalog, selection: { machine: printer, process: process!, filaments }, options };
}

/**
 * Sidebar edits as engine override tokens: text edits already hold the option's
 * `serialize()` text (the Field.cpp commit), typed values are encoded by the engine.
 */
export async function overrideTokens(overrides: ConfigOverrides): Promise<Record<string, string | string[]>> {
  const isText = (value: OptionValue): value is string | string[] =>
    typeof value === "string" || (Array.isArray(value) && value.every((element) => typeof element === "string"));
  const edits = Object.entries(overrides);
  const typed = edits.filter(([, value]) => !isText(value));
  const encoded = await slicer.encodeConfigValues(typed);
  return Object.fromEntries([
    ...edits.filter((edit): edit is [string, string | string[]] => isText(edit[1])),
    ...typed.map(([key], at) => [key, encoded[at]]),
  ]);
}

/**
 * Slices with the selected presets, like the OrcaSlicer CLI's `--load-settings` /
 * `--load-filaments`: plate `plate` of the edited 3MF over its project config, or an STL centered on
 * the bed as Plater::priv::load_model_objects places a model loaded on an empty
 * plate (Plater.cpp:7056-7058, 7111-7112, 7149-7160). Sidebar edits apply after
 * the presets, as an edit of the selected preset in a settings tab does.
 */
export async function sliceWithPresets(file: ModelFile, scene: Scene, selection: PresetSelection, overrides: ConfigOverrides, plate?: number): Promise<SliceResult> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, useApp.getState(), plate);
  // The plate combo's `curr_bed_type` is part of the project config the slice composes.
  // So are the dual-nozzle volume types and filament map.
  const project: ConfigOverrides = {
    ...(selection.bedType === undefined ? {} : { curr_bed_type: selection.bedType }),
    ...extruderOverrides(selection, presetFilamentSlots(scene.settings)),
  };
  const request = { ...options, overrides: await overrideTokens({ ...overrides, ...project }) };
  const { plater } = useApp.getState();
  return plater
    ? plater.sliceWithProfiles(catalog, profiles, request)
    : slicer.sliceWithProfiles(file.bytes, file.name, catalog, profiles, request);
}

/**
 * Values of the selected presets as the preset slice composes them (`PresetBundle::full_config`),
 * typed where decoded, for the process tab to show and edit.
 */
export async function presetValues(file: ModelFile, scene: Scene, selection: PresetSelection, sources: CatalogSources): Promise<OptionMap> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, sources);
  const settings = await slicer.profileSettings(file.bytes, file.name, catalog, profiles, options);
  return { ...settings, ...(await decodeOptions(useApp.getState().configDefs!, settings)) };
}

/**
 * The scene as the preset slice places it on the selected printer's bed: an STL centered like
 * one model loaded on an empty plate (Plater.cpp:7149-7160), with the composed config's bed.
 */
export async function placedScene(file: ModelFile, scene: Scene, selection: PresetSelection, sources: CatalogSources): Promise<Scene> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, sources);
  return slicer.placementScene(file.bytes, file.name, catalog, profiles, options);
}

/** Names of the imported presets of one `type`, for the "User presets" group. */
export function importedNames(imports: readonly ParsedPresetBundle[], type: "machine" | "process" | "filament"): Set<string> {
  return new Set(
    imports.flatMap(({ profiles }) =>
      profiles.filter((profile) => profile.type === type).map((profile) => profile.name as string)),
  );
}
