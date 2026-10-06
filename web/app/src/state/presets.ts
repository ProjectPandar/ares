import type { ConfigOverrides, CurrentPrinter, OptionMap, OptionValue, ParsedPresetBundle, PresetCatalog, ProfileSliceOptions, Scene, SliceResult } from "@ares-slicer/wasm";
import { presetFilamentSlots } from "../sidebar/options";
import { slicer } from "./slicer";
import { type ModelFile, type PresetSelection, useApp } from "./store";
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

if (useApp.getState().vendors.length > 0) void updateSystemPrinters();

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
  await Promise.all([updateSystemPrinters(), updateCompatible()]);
}

/** Selects a user or system printer; `null` returns to the project's presets. */
export async function selectPrinter(printer: string | null) {
  useApp.setState({ presetSelection: printer === null ? null : { printer, filaments: [] } });
  await updateCompatible();
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
  await updateCompatible();
}

export function selectFilament(slot: number, filament: string) {
  const selection: PresetSelection = useApp.getState().presetSelection!;
  const filaments = [...selection.filaments];
  filaments[slot] = filament;
  useApp.setState({ presetSelection: { ...selection, filaments } });
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
async function profileRequest(scene: Scene, { printer, process, filaments }: PresetSelection, { vendors, presetImports }: CatalogSources) {
  const { loadVendor } = await import("@ares-slicer/profiles");
  const loaded = await Promise.all([FILAMENT_LIBRARY, ...vendors].map(loadVendor));
  const catalog = JSON.stringify({ vendors: loaded, imports: presetImports });
  const options: ProfileSliceOptions = {
    plate: scene.plates[0]?.id,
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
 * `--load-filaments`: a 3MF plate over its project config, or an STL centered on
 * the bed as Plater::priv::load_model_objects places a model loaded on an empty
 * plate (Plater.cpp:7056-7058, 7111-7112, 7149-7160). Sidebar edits apply after
 * the presets, as an edit of the selected preset in a settings tab does.
 */
export async function sliceWithPresets(file: ModelFile, scene: Scene, selection: PresetSelection, overrides: ConfigOverrides): Promise<SliceResult> {
  const { catalog, selection: profiles, options } = await profileRequest(scene, selection, useApp.getState());
  return slicer.sliceWithProfiles(file.bytes, file.name, catalog, profiles, { ...options, overrides: await overrideTokens(overrides) });
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
