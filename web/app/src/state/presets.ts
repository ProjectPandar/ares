import type { CurrentPrinter, FilamentSlotState, InstalledPrinters, ParsedPresetBundle, PlaterSession, PresetCatalog, ProjectSettings, Scene } from "@ares-slicer/wasm";
import type { DialogFilament } from "../sidebar/filamentDialog";
import { firstString } from "../sidebar/options";
import { deleteImport, loadImports, type PrinterSettings, saveConfig, saveImports, storedConfig, storedConfigExists } from "./persistence";
import { selectionExtruders } from "./extruders";
import { printerBedType, updatePrinterPanel } from "./printer";
import { catalogJson } from "./projectPresets";
import { slicer } from "./slicer";
import { type PresetSelection, useApp } from "./store";

/**
 * PresetBundle::ORCA_FILAMENT_LIBRARY: always installed and loaded before the
 * other vendors (PresetBundle.cpp:2216-2234).
 */
export const FILAMENT_LIBRARY = "OrcaFilamentLibrary";

/** Imported preset files, in import order, for the engine catalog. */
const importFiles: { input: Uint8Array; filename: string }[] = [];

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
  await updateSystemPrinters();
  await updateInstalledFilaments();
  // GUI_App::config_wizard_startup (GUI_App.cpp:10129-10135): no app config, or only default
  // printers (Preset.cpp:4599-4606), where imported printers count as visible.
  const { systemPrinters, presetImports } = useApp.getState();
  if (!storedConfigExists || (systemPrinters.length === 0 && importedNames(presetImports, "machine").size === 0)) {
    useApp.setState({ wizard: "first-run" });
  }
  const { printer } = storedConfig;
  if (printer === null) return;
  // select_preset_by_name_strict: a printer that is no longer loaded is not selected.
  const printers = await (await presetCatalog()).printers();
  if (printers.some(({ name }) => name === printer)) await selectPrinter(printer);
  else saveConfig({ printer: null });
}

/** The startup selection; a file load waits for it, as the plater only takes files after GUI_App init. */
export const sessionRestored = restoreSession();

/** Whether the selection still has the printer and process a filter ran for; filaments do not change it. */
function filtered({ printer, process }: PresetSelection) {
  const selection = useApp.getState().presetSelection;
  return selection?.printer === printer && selection.process === process;
}

/**
 * PresetBundle::update_compatible(Always) for the selected printer and process: the compatible
 * lists, and each missing or incompatible filament slot replaced by the best compatible visible
 * preset (PresetBundle.cpp:8650-8673), then the slots padded to the printer's minimum
 * (update_multi_material_filament_presets, 8611-8617). `project` replaces the selection's palette and flush data.
 */
export async function updateCompatible(project?: Partial<FilamentSlotState>) {
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
    const { scene, installedFilaments } = useApp.getState();
    const selected = await catalog.selectCompatibleFilaments(selection.printer, selection.process, selection.filaments, [...installedFilaments]);
    const { filaments, ...filamentProject } = await catalog.normalizeFilamentSlots(selection.printer,
      { ...(project ?? selection.filamentProject ?? projectFilamentState(scene?.settings ?? null)), filaments: selected });
    // No project open: load_selections copies the filament colours into filament_multi_colour (PresetBundle.cpp:4116-4117).
    if (!project && !selection.filamentProject && !scene?.settings) filamentProject.filament_multi_colour = filamentProject.filament_colour;
    if (filtered(selection)) {
      const current = useApp.getState().presetSelection!;
      useApp.setState({ presetSelection: { ...current, filaments, filamentProject }, compatible: { status: "done", presets, filamentInfo } });
    }
  } catch (error) {
    if (filtered(selection)) {
      useApp.setState({ compatible: { status: "error", error: (error as Error).message } });
    }
  }
}

/** The loaded project's arrays; omitted fields use the core's PrintConfig defaults. */
export function projectFilamentState(settings: ProjectSettings | null): Partial<FilamentSlotState> {
  const values = { ...settings?.project, ...settings?.filament };
  const state: Partial<FilamentSlotState> = {};
  const entries = (key: string) => {
    const raw = values[key];
    return Array.isArray(raw) ? raw : String(raw).split(/[;,]/);
  };
  for (const key of ["flush_multiplier", "flush_volumes_vector", "flush_volumes_matrix", "filament_map", "filament_nozzle_map", "filament_volume_map"] as const) {
    if (key in values) state[key] = entries(key).map(Number);
  }
  for (const key of ["filament_colour", "filament_multi_colour", "filament_colour_type"] as const) {
    if (key in values) state[key] = entries(key).map(String);
  }
  return state;
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

/**
 * PresetBundle::export_selections (PresetBundle.cpp:4232-4292): the selection, also under its printer with the
 * project palette and flush data.
 */
export function saveSelection() {
  const { presetSelection: selection, scene } = useApp.getState();
  if (!selection) {
    saveConfig({ printer: null });
    return;
  }
  const { printer, process, filaments, bedType } = selection;
  const project = selection.filamentProject ?? projectFilamentState(scene?.settings ?? null);
  const settings = {
    ...storedConfig.printers[printer], process, filaments: Array.from(filaments, (name) => name ?? ""), bedType, ...exportedProject(project),
  };
  saveConfig({ printer, printers: { ...storedConfig.printers, [printer]: settings } });
}

/** The palette and flush data export_selections stores (PresetBundle.cpp:4264-4292). */
export function exportedProject(project: Partial<FilamentSlotState>) {
  // std::to_string writes six decimals (4279-4292).
  const toString = (values?: number[]) => values?.map((value) => Number(value.toFixed(6)));
  return {
    filamentColours: project.filament_colour,
    filamentMultiColours: project.filament_multi_colour,
    filamentColourTypes: project.filament_colour_type,
    flushVolumesMatrix: toString(project.flush_volumes_matrix),
    flushVolumesVector: toString(project.flush_volumes_vector),
    flushMultiplier: toString(project.flush_multiplier),
  };
}

/**
 * PresetBundle::update_selections from the selection `saved` stored for a printer (PresetBundle.cpp:3914-4003), which Tab::select_preset runs with
 * the default `remember_printer_config` (Tab.cpp:7740-7742) and load_selections repeats at startup (4082-4164):
 * the print, slots and palette export_selections stored for the printer (4251-4292). A printer without them gets
 * an empty print and slot 1, which update_compatible fills with the printer's preferred presets, and one "#26A69A"
 * colour; the flush data stays the current project's.
 */
function rememberedSelection(saved: PrinterSettings | undefined) {
  const { presetSelection: current, scene } = useApp.getState();
  const project = current?.filamentProject ?? projectFilamentState(scene?.settings ?? null);
  // 3929-3937: slot 1, then filament_01, filament_02, ...
  const filaments = [saved?.filaments[0] ?? "", ...(saved?.filaments.slice(1) ?? [])];
  const resized = (values: string[], fill: string) => Array.from({ length: filaments.length }, (_, index) => values[index] ?? fill);
  const colours = resized(saved?.filamentColours ?? [], "#26A69A");
  return {
    // 3918-3923: the stored print, strictly selected; "" leaves none (Preset.cpp:4335-4357).
    process: saved?.process ?? "",
    filaments,
    project: {
      flush_volumes_matrix: saved?.flushVolumesMatrix ?? project.flush_volumes_matrix,
      flush_volumes_vector: saved?.flushVolumesVector ?? project.flush_volumes_vector,
      flush_multiplier: saved?.flushMultiplier ?? project.flush_multiplier,
      filament_colour: colours,
      filament_multi_colour: saved?.filamentMultiColours?.length ? saved.filamentMultiColours : colours,
      filament_colour_type: resized(saved?.filamentColourTypes ?? [], "1"),
      // 3963-3970: one filament_map, filament_nozzle_map and filament_volume_map entry per colour.
      filament_map: colours.map(() => 1),
      filament_nozzle_map: colours.map(() => 0),
      filament_volume_map: colours.map(() => 0),
    } satisfies Partial<FilamentSlotState>,
  };
}

/**
 * Selects a user or system printer with the selection `remembered` for it (`rememberedSelection`), by default its
 * own; `null` returns to the project's presets. update_compatible(Always) keeps the remembered print while compatible, else picks the one
 * PreferedPrintProfileMatch prefers against the printer's default_print_profile (PresetBundle.cpp:8837-8838).
 */
export async function selectPrinter(printer: string | null, stored = printer === null ? undefined : storedConfig.printers[printer]) {
  const saved = printer === null ? undefined : storedConfig.printers[printer];
  const remembered = printer === null ? null : rememberedSelection(stored);
  if (printer === null || remembered === null) useApp.setState({ presetSelection: null });
  else {
    // The combo shows the printer and the filtering state while the print is chosen.
    useApp.setState({ presetSelection: { printer, filaments: remembered.filaments }, compatible: { status: "filtering" } });
    // A printer the catalog cannot resolve has no print; updateCompatible shows the engine's error.
    const process = await (await presetCatalog()).compatiblePrint(printer, remembered.process).catch(() => null);
    const chosen = useApp.getState().presetSelection;
    if (process && chosen?.printer === printer) useApp.setState({ presetSelection: { ...chosen, process } });
  }
  await updateCompatible(remembered?.project);
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
  // Plater::priv::on_select_preset resolves the model and selects its preset in the selection event (Plater.cpp:13016-13031).
  useApp.setState({ compatible: { status: "filtering" } });
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
  await updateCompatible();
  saveSelection();
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
  if (scene.format === "stl" && scene.objects.length !== 1 && useApp.getState().file?.kind !== "stl-batch") {
    return "Placing more than one STL object on the plate needs arrange, which is not ported yet";
  }
  if (!selection.process) return "Select a process preset to slice";
  if (selection.filaments.some((name) => !name)) {
    return "Select a filament preset for every filament to slice";
  }
  return null;
}

/** Names of the imported presets of one `type`, for the "User presets" group. */
export function importedNames(imports: readonly ParsedPresetBundle[], type: "machine" | "process" | "filament"): Set<string> {
  return new Set(
    imports.flatMap(({ profiles }) =>
      profiles.filter((profile) => profile.type === type).map((profile) => profile.name as string)),
  );
}
