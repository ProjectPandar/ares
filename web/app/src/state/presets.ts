import type { CurrentPrinter, FilamentSlotState } from "@ares-slicer/wasm";
import { firstString } from "../sidebar/options";
import { selectionExtruders } from "./extruders";
import { deleteImport, loadImports, saveConfig, saveImports, storedConfig, storedConfigExists } from "./persistence";
import {
  addImportFiles,
  importedNames,
  presetCatalog,
  updateInstalledFilaments,
  updateSystemPrinters,
} from "./presetCatalog";
import { projectFilamentState, rememberedSelection, saveSelection } from "./presetPersistence";
import { printerBedType, updatePrinterPanel } from "./printer";
import { slicer } from "./slicer";
import { type PresetSelection, useApp } from "./store";

export { FILAMENT_LIBRARY } from "./projectPresets";
export {
  dialogFilaments,
  importedNames,
  installVendors,
  presetCatalog,
  setInstalledFilaments,
  updateInstalledFilaments,
  updateSystemPrinters,
} from "./presetCatalog";
export { exportedProject, presetSliceBlocker, projectFilamentState, saveSelection } from "./presetPersistence";

/**
 * GUI_App startup (PresetBundle::load_selections, PresetBundle.cpp:2723): the stored
 * imports join the catalog and the stored printer is selected again.
 */
async function restoreSession() {
  const stored = await loadImports();
  for (const { key, file } of stored) {
    try {
      const parsed = await slicer.parsePresetBundle(file.bytes, file.filename);
      addImportFiles([{ input: file.bytes, filename: file.filename }]);
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
    const filamentInfo = new Map(
      presets.filaments.map((name, index) => [
        name,
        {
          type: firstString(values[index], "filament_type") ?? "",
          vendor: firstString(values[index], "filament_vendor") ?? "",
        },
      ]),
    );
    const { scene, installedFilaments } = useApp.getState();
    const selected = await catalog.selectCompatibleFilaments(
      selection.printer,
      selection.process,
      selection.filaments,
      [...installedFilaments],
    );
    const { filaments, ...filamentProject } = await catalog.normalizeFilamentSlots(selection.printer, {
      ...(project ?? selection.filamentProject ?? projectFilamentState(scene?.settings ?? null)),
      filaments: selected,
    });
    // No project open: load_selections copies the filament colours into filament_multi_colour (PresetBundle.cpp:4116-4117).
    if (!project && !selection.filamentProject && !scene?.settings)
      filamentProject.filament_multi_colour = filamentProject.filament_colour;
    if (filtered(selection)) {
      const current = useApp.getState().presetSelection!;
      useApp.setState({
        presetSelection: { ...current, filaments, filamentProject },
        compatible: { status: "done", presets, filamentInfo },
      });
    }
  } catch (error) {
    if (filtered(selection)) {
      useApp.setState({ compatible: { status: "error", error: (error as Error).message } });
    }
  }
}

/** MainFrame::load_config_file: adds the files' presets as user presets. */
export async function importPresetFiles(files: File[]) {
  try {
    const inputs = await Promise.all(files.map(async (file) => new Uint8Array(await file.arrayBuffer())));
    const parsed = await Promise.all(files.map((file, index) => slicer.parsePresetBundle(inputs[index], file.name)));
    addImportFiles(files.map((file, index) => ({ input: inputs[index], filename: file.name })));
    useApp.setState(({ presetImports }) => ({ presetImports: [...presetImports, ...parsed], loadError: undefined }));
    await saveImports(files.map((file, index) => ({ filename: file.name, bytes: inputs[index] })));
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
    return;
  }
  await updateCompatible();
}

/**
 * Selects a user or system printer with the selection `remembered` for it (`rememberedSelection`), by default its
 * own; `null` returns to the project's presets. update_compatible(Always) keeps the remembered print while compatible, else picks the one
 * PreferedPrintProfileMatch prefers against the printer's default_print_profile (PresetBundle.cpp:8837-8838).
 */
export async function selectPrinter(
  printer: string | null,
  stored = printer === null ? undefined : storedConfig.printers[printer],
) {
  const saved = printer === null ? undefined : storedConfig.printers[printer];
  const remembered = printer === null ? null : rememberedSelection(stored);
  if (printer === null || remembered === null) useApp.setState({ presetSelection: null, compatible: null });
  else {
    // The combo shows the printer and the filtering state while the print is chosen.
    useApp.setState({
      presetSelection: { printer, filaments: remembered.filaments },
      compatible: { status: "filtering" },
    });
    // A printer the catalog cannot resolve has no print; updateCompatible shows the engine's error.
    const process = await (await presetCatalog()).compatiblePrint(printer, remembered.process).catch(() => null);
    const chosen = useApp.getState().presetSelection;
    if (chosen?.printer !== printer) return;
    if (process) useApp.setState({ presetSelection: { ...chosen, process } });
  }
  // Keep filtering visible until the selected printer has its effective plate type.
  await updatePrinterPanel();
  const { presetSelection: selection, printerPanel } = useApp.getState();
  if ((selection?.printer ?? null) !== printer) return;
  if (selection) {
    useApp.setState({
      presetSelection: {
        ...selection,
        bedType: printerBedType(saved?.bedType),
        ...selectionExtruders(selection.printer, printerPanel),
      },
    });
    await updateCompatible(remembered?.project);
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
      useApp.setState({
        compatible: { status: "error", error: `No printer preset of model "${model}" for "${current.name}"` },
      });
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
