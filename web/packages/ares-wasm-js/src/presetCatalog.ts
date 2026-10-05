import type { RequestBody } from "./protocol.js";
import type { CompatiblePresets } from "./types.js";

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

  /** Releases the worker's catalog. */
  async free(): Promise<void> {
    await this.#call({ kind: "catalogFree", catalogId: this.#catalogId });
  }
}
