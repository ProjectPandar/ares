import type { RequestBody } from "./protocol.js";
import type { CompatiblePresets, CurrentPrinter, GuideFilament, InstalledPrinters, OptionMap, PresetKind, PrinterModel, PrinterModelInfo, SelectablePrinter } from "./types.js";

export interface FilamentSlotState {
  filaments: string[];
  flush_multiplier: number[];
  flush_volumes_vector: number[];
  flush_volumes_matrix: number[];
  filament_colour: string[];
  filament_multi_colour: string[];
  filament_colour_type: string[];
  filament_map: number[];
  filament_nozzle_map: number[];
  filament_volume_map: number[];
}

type Call = (body: RequestBody) => Promise<unknown>;

/**
 * Preset catalog held in the slicer worker (`PresetBundle` holding semantics):
 * each vendor and import is parsed once when added, and filtering reuses the
 * loaded presets. Create with `AresSlicer.createPresetCatalog`; `free` it when
 * replaced.
 */
export class PresetCatalog {
  readonly #call: Call;
  readonly #catalogId: number;

  /** @internal */
  constructor(call: Call, catalogId: number) {
    this.#call = call;
    this.#catalogId = catalogId;
  }

  /** @internal The worker's id of this catalog. */
  get workerId(): number {
    return this.#catalogId;
  }

  /**
   * Adds one vendor `{id, index, files}` as JSON bytes, e.g. the UTF-8 of
   * `JSON.stringify(await loadVendor(id))`. Add OrcaFilamentLibrary first,
   * as `PresetBundle::load_system_presets_from_json` loads it.
   */
  async addVendor(vendorJson: Uint8Array): Promise<void> {
    await this.#call({ kind: "catalogAddVendor", catalogId: this.#catalogId, input: vendorJson.slice() });
  }

  /** Adds the user presets of an OrcaSlicer preset JSON or `.orca_*` archive (`PresetBundle::import_presets`). */
  async addImport(input: Uint8Array, filename: string): Promise<void> {
    await this.#call({ kind: "catalogAddImport", catalogId: this.#catalogId, input: input.slice(), filename });
  }

  /**
   * Selectable processes and filaments compatible with the `printer` preset,
   * and filaments also with the `process` preset when given
   * (`Preset::is_compatible_with_printer` / `is_compatible_with_print`).
   * Presets whose parent is not loaded are left out and listed in
   * `missing_parents`. Rejects when the selected printer or process is not
   * in the catalog or is missing a parent.
   */
  async filterCompatible(printer: string, process?: string): Promise<CompatiblePresets> {
    return (await this.#call({ kind: "catalogFilter", catalogId: this.#catalogId, printer, process })) as CompatiblePresets;
  }

  /**
   * Printers a printer combo lists, in collection order with the vendors'
   * printers first and imported ones (`vendor` "user") last. Printers with
   * `instantiation` "false" or an unloaded parent are left out
   * (`PresetComboBoxes.cpp:1170-1186`).
   */
  async printers(): Promise<SelectablePrinter[]> {
    return (await this.#call({ kind: "catalogPrinters", catalogId: this.#catalogId })) as SelectablePrinter[];
  }

  /**
   * The system printer models the plater printer combo lists instead of their
   * presets, each once (`PresetComboBoxes.cpp:1238-1251`). Imported printers
   * are not models; `printers()` lists them by name.
   */
  async printerModels(): Promise<PrinterModel[]> {
    return (await this.#call({ kind: "catalogPrinterModels", catalogId: this.#catalogId })) as PrinterModel[];
  }

  /**
   * The printer preset a selected printer `model` stands for
   * (`PresetBundle::get_similar_printer_preset`), preferring the preset like
   * `current` with `variant` (empty: `current`'s variant). `null` when no
   * printer has the model, or when `model` is empty and `current` is not in
   * the catalog.
   */
  async similarPrinterPreset(model: string, variant: string, current: CurrentPrinter): Promise<string | null> {
    return (await this.#call({ kind: "catalogSimilarPrinter", catalogId: this.#catalogId, model, variant, current })) as string | null;
  }

  /**
   * The `keys` of each named preset of `kind` as loaded (`inherits` applied,
   * `PresetBundle.cpp:4885-4927`), in `names` order: the stored value of the
   * nearest preset in the chain that sets a key, else the option's default
   * text. Rejects an unknown name or key.
   */
  async presetValues(kind: PresetKind, names: string[], keys: string[]): Promise<OptionMap[]> {
    return (await this.#call({ kind: "catalogPresetValues", catalogId: this.#catalogId, presetKind: kind, names, keys })) as OptionMap[];
  }

  /** Vendor, unsupported bed types, default materials and nozzle diameters of a `printer_model`. */
  async printerModelInfo(model: string): Promise<PrinterModelInfo> {
    return (await this.#call({ kind: "catalogPrinterModelInfo", catalogId: this.#catalogId, model })) as PrinterModelInfo;
  }

  /**
   * The installed filament set (AppConfig `filaments` section) after
   * `PresetBundle::load_installed_filaments` seeds it with the default
   * materials of printers no installed filament is compatible with, sorted.
   */
  async installedFilaments(installed: string[]): Promise<string[]> {
    return (await this.#call({ kind: "catalogInstalledFilaments", catalogId: this.#catalogId, installed })) as string[];
  }

  /**
   * The filament of each slot after `PresetBundle::update_compatible(Always)`: a compatible
   * filament stays, a missing or incompatible one takes the best compatible visible preset, or
   * "" when there is none. System filaments are visible when `installed` names them.
   */
  async selectCompatibleFilaments(printer: string, process: string | undefined, filaments: string[], installed: string[]): Promise<string[]> {
    return (await this.#call({ kind: "catalogSelectCompatibleFilaments", catalogId: this.#catalogId, printer, process, filaments, installed })) as string[];
  }

  /** PresetBundle hardware minimum, last-preset padding, and per-nozzle flush layout. */
  async normalizeFilamentSlots(printer: string, state: Partial<FilamentSlotState> & { filaments: string[] }): Promise<FilamentSlotState> {
    return (await this.#call({ kind: "catalogNormalizeFilamentSlots", catalogId: this.#catalogId, printer, state })) as FilamentSlotState;
  }

  /**
   * Sets the `AppConfig` vendors section. `printers`, `printerModels` and `installedFilaments`
   * then hide the system printers `Preset::set_visible_from_appconfig` hides; without it every
   * printer is listed.
   */
  async setInstalledPrinters(installed: InstalledPrinters): Promise<void> {
    await this.#call({ kind: "catalogSetInstalledPrinters", catalogId: this.#catalogId, installed });
  }

  /**
   * The first system printer of `model` with `variant` (any variant when empty), visible or not
   * (`PrinterPresetCollection::find_system_preset_by_model_and_variant`).
   */
  async systemPrinter(model: string, variant: string): Promise<string | null> {
    return (await this.#call({ kind: "catalogSystemPrinter", catalogId: this.#catalogId, model, variant })) as string | null;
  }

  /**
   * The process a selection of `printer` holds (`PresetBundle::update_compatible` with `Always`):
   * `current` while it is compatible, else the compatible process that best matches its alias,
   * the printer's `default_print_profile` and its layer height; null when none is compatible.
   */
  async compatiblePrint(printer: string, current?: string): Promise<string | null> {
    return (await this.#call({ kind: "catalogCompatiblePrint", catalogId: this.#catalogId, printer, current })) as string | null;
  }

  /** The loaded system filaments in name order with the printers they are compatible with, for the setup guide. */
  async guideFilaments(): Promise<GuideFilament[]> {
    return (await this.#call({ kind: "catalogGuideFilaments", catalogId: this.#catalogId })) as GuideFilament[];
  }

  /** Releases the worker's catalog. */
  async free(): Promise<void> {
    await this.#call({ kind: "catalogFree", catalogId: this.#catalogId });
  }
}
