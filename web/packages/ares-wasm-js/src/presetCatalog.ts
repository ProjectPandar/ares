import type { RequestBody } from "./protocol.js";
import type { CompatiblePresets, CurrentPrinter, OptionMap, PresetKind, PrinterModel, PrinterModelInfo, SelectablePrinter } from "./types.js";

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

  /**
   * Adds one vendor `{index, files}` as JSON bytes, e.g. the UTF-8 of
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

  /** Releases the worker's catalog. */
  async free(): Promise<void> {
    await this.#call({ kind: "catalogFree", catalogId: this.#catalogId });
  }
}
