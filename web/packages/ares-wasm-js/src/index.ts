import type { PrintConfigDef } from "./configDef.js";
import { PlaterSession } from "./platerSession.js";
import { PresetCatalog } from "./presetCatalog.js";
import type { RequestBody, WorkerRequest, WorkerResponse } from "./protocol.js";
import type { MovesSlider, MovesView, SliceResult, ViewColors, ViewRange } from "./toolpaths.js";
import type { CompatiblePresets, ConfigValue, GuideModel, OptionMap, ParsedPresetBundle, ProfileSelection, ProfileSliceOptions, Scene, SlicingStatus, StlFile, StlImportOptions } from "./types.js";

export * from "./toolpaths.js";
export { PlaterSession } from "./platerSession.js";
export { PresetCatalog, type FilamentSlotState } from "./presetCatalog.js";
export type * from "./configDef.js";
export type * from "./types.js";

type Pending = { resolve: (value: unknown) => void; reject: (error: Error) => void; onStatus?: (status: SlicingStatus) => void };

export interface AresSlicerOptions {
  /** A module worker running `@ares-slicer/wasm/worker`. */
  worker: Worker;
  /** URL of `@ares-slicer/wasm/ares_wasm_bg.wasm`. */
  wasmUrl: string | URL;
}

/**
 * Promise client for the Ares WebAssembly slicer. Every call runs in the
 * worker, so slicing never blocks the calling thread (the role of
 * OrcaSlicer's `BackgroundSlicingProcess`). Input bytes are copied before
 * transfer; callers keep ownership of their buffers.
 */
export class AresSlicer {
  readonly #worker: Worker;
  readonly #pending = new Map<number, Pending>();
  readonly #ready: Promise<void>;
  #nextId = 0;
  #nextCatalogId = 0;
  #nextSessionId = 0;

  constructor({ worker, wasmUrl }: AresSlicerOptions) {
    this.#worker = worker;
    worker.onmessage = ({ data }: MessageEvent<WorkerResponse>) => {
      // Statuses of a running request arrive before its response.
      if ("status" in data) {
        this.#pending.get(data.id)?.onStatus?.(data.status);
        return;
      }
      const pending = this.#pending.get(data.id);
      this.#pending.delete(data.id);
      if (data.ok) pending?.resolve(data.value);
      else pending?.reject(new Error(data.error));
    };
    worker.onerror = (event) => {
      const error = new Error(event.message || "Ares worker failed");
      for (const pending of this.#pending.values()) pending.reject(error);
      this.#pending.clear();
    };
    this.#ready = this.#call({ kind: "init", wasmUrl: String(wasmUrl) }).then(() => undefined);
  }

  /** Resolves once the WebAssembly module is instantiated in the worker. */
  ready(): Promise<void> {
    return this.#ready;
  }

  /** `PrintConfigDef` option metadata (labels, units, enums, ranges, modes) for settings UIs. */
  async printConfigDef(): Promise<PrintConfigDef> {
    await this.#ready;
    return (await this.#call({ kind: "printConfigDef" })) as PrintConfigDef;
  }

  /** Loads a 3MF project or STL model into plater scene data. */
  async loadScene(input: Uint8Array): Promise<Scene> {
    await this.#ready;
    return (await this.#call({ kind: "loadScene", input: input.slice() })) as Scene;
  }

  /**
   * Slices one plate of a 3MF project (first plate when omitted). `overrides`
   * are user edits applied over the project config, by option key: a
   * `ConfigOption::serialize()` string from `encodeConfigValues`, or one string
   * per element of a vector option. Rejects an unknown key or invalid value.
   */
  async sliceProject(input: Uint8Array, plate?: number, overrides: Record<string, string | string[]> = {}): Promise<SliceResult> {
    await this.#ready;
    return (await this.#call({ kind: "sliceProject", input: input.slice(), plate, overrides: JSON.stringify(overrides) })) as SliceResult;
  }

  /**
   * Slices a model with selected OrcaSlicer presets, like the OrcaSlicer CLI's
   * `--load-settings` / `--load-filaments`. `filename` is the model's file name
   * (`File.name`); it names STL objects. `catalogJson` is the preset catalog
   * `{"vendors": [...], "imports": [...]}`; the engine resolves inheritance and
   * composes the config. Serialize a large catalog once and reuse the string.
   */
  async sliceWithProfiles(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<SliceResult> {
    await this.#ready;
    return (await this.#call({
      kind: "sliceWithProfiles",
      input: input.slice(),
      filename,
      catalog: catalogJson,
      selection: JSON.stringify(selection),
      options: JSON.stringify(options),
    })) as SliceResult;
  }

  /**
   * The scene `sliceWithProfiles` slices for the same arguments: the model
   * loaded with the selected presets and placed as `options.placement` asks
   * (an STL centered on the selected printer's bed), with the composed
   * config as `settings`. Rejects as `sliceWithProfiles` does.
   */
  async placementScene(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<Scene> {
    await this.#ready;
    return (await this.#call({
      kind: "placementScene",
      input: input.slice(),
      filename,
      catalog: catalogJson,
      selection: JSON.stringify(selection),
      options: JSON.stringify(options),
    })) as Scene;
  }

  /**
   * The config `sliceWithProfiles` slices with for the same arguments: the
   * selected presets composed over the input (`PresetBundle::full_config`) with
   * `options.overrides` applied, as one option map in the stored form of
   * `Scene.settings`. Rejects as `sliceWithProfiles` does on a bad catalog,
   * selection or override.
   */
  async profileSettings(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<OptionMap> {
    await this.#ready;
    return (await this.#call({
      kind: "profileSettings",
      input: input.slice(),
      filename,
      catalog: catalogJson,
      selection: JSON.stringify(selection),
      options: JSON.stringify(options),
    })) as OptionMap;
  }

  /**
   * Reads the presets of an OrcaSlicer preset JSON or an `.orca_printer` /
   * `.orca_filament` / `.orca_bundle` archive (`PresetBundle::import_presets`).
   * Add the result to the catalog's `imports`.
   */
  async parsePresetBundle(input: Uint8Array, filename: string): Promise<ParsedPresetBundle> {
    await this.#ready;
    return (await this.#call({ kind: "parsePresetBundle", input: input.slice(), filename })) as ParsedPresetBundle;
  }

  /**
   * The setup guide's printer models (`VendorProfile::PrinterModel`) of vendor envelopes
   * `[{id, index, files}]` as JSON bytes, such as `loadPrinterModels()` of `@ares-slicer/profiles`.
   */
  async guideModels(vendorsJson: Uint8Array): Promise<GuideModel[]> {
    await this.#ready;
    return (await this.#call({ kind: "guideModels", input: vendorsJson.slice() })) as GuideModel[];
  }

  /** An empty preset catalog in the worker; add vendors and imports, then filter. */
  async createPresetCatalog(): Promise<PresetCatalog> {
    await this.#ready;
    const catalogId = this.#nextCatalogId++;
    await this.#call({ kind: "catalogCreate", catalogId });
    return new PresetCatalog((body) => this.#call(body), catalogId);
  }

  /**
   * Loads a 3MF project into an editable plater session in the worker
   * (`PartPlateList::load_from_3mf_structure`). `filename` names the file for preset slicing.
   * Rejects as `loadScene` does on a bad project.
   */
  async createPlaterSession(input: Uint8Array, filename: string): Promise<PlaterSession> {
    await this.#ready;
    const sessionId = this.#nextSessionId++;
    await this.#call({ kind: "platerCreate", sessionId, input: input.slice(), filename });
    return new PlaterSession((body, onStatus) => this.#call(body, onStatus), sessionId);
  }

  /**
   * Loads a model with selected presets into an editable plater session, the bed and plates from
   * the composed config: an STL lands on the bed centre as on an empty plate when
   * `options.placement` is "center" (`Plater::priv::load_model_objects`). Arguments and
   * rejections are those of `sliceWithProfiles`.
   */
  async createPlaterSessionWithProfiles(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<PlaterSession> {
    await this.#ready;
    const sessionId = this.#nextSessionId++;
    await this.#call({
      kind: "platerCreateWithProfiles",
      sessionId,
      input: input.slice(),
      filename,
      catalog: catalogJson,
      selection: JSON.stringify(selection),
      options: JSON.stringify(options),
    });
    return new PlaterSession((body, onStatus) => this.#call(body, onStatus), sessionId);
  }

  /** Loads every named STL through the core's multipart load policy. */
  async createPlaterSessionFromStlFiles(files: readonly StlFile[], choices: StlImportOptions): Promise<PlaterSession> {
    await this.#ready;
    const sessionId = this.#nextSessionId++;
    await this.#call({
      kind: "platerCreateFromStlFiles", sessionId,
      inputs: files.map(({ bytes }) => bytes.slice()),
      batch: JSON.stringify({ names: files.map(({ name }) => name), ...choices }),
    });
    return new PlaterSession((body, onStatus) => this.#call(body, onStatus), sessionId);
  }

  /** Loads the complete STL batch onto the selected printer's bed. */
  async createPlaterSessionFromStlFilesWithProfiles(
    files: readonly StlFile[], choices: StlImportOptions, catalogJson: string,
    selection: ProfileSelection, options: ProfileSliceOptions,
  ): Promise<PlaterSession> {
    await this.#ready;
    const sessionId = this.#nextSessionId++;
    await this.#call({
      kind: "platerCreateFromStlFilesWithProfiles", sessionId,
      inputs: files.map(({ bytes }) => bytes.slice()),
      batch: JSON.stringify({ names: files.map(({ name }) => name), ...choices }),
      catalog: catalogJson, selection: JSON.stringify(selection), options: JSON.stringify(options),
    });
    return new PlaterSession((body, onStatus) => this.#call(body, onStatus), sessionId);
  }

  /**
   * Selectable processes and filaments of the catalog compatible with the
   * `printer` preset, and filaments also with the `process` preset when given
   * (`Preset::is_compatible_with_printer` / `is_compatible_with_print`).
   * `catalogJson` is the same catalog string `sliceWithProfiles` takes.
   * Rejects when a selected preset or one of its parents is not in the catalog.
   */
  async filterCompatiblePresets(catalogJson: string, printer: string, process?: string): Promise<CompatiblePresets> {
    await this.#ready;
    return (await this.#call({ kind: "filterCompatiblePresets", catalog: catalogJson, printer, process })) as CompatiblePresets;
  }

  /**
   * Decodes `[key, serialized]` entries, each a value in `ConfigOption::serialize()` form as
   * 3MF projects and presets store it (e.g. `"1"` for a bool, `"0x0"` for a point), into typed
   * values in one round trip. A vector key decodes its comma-joined elements to an array.
   * Rejects on an unknown key or a malformed value, naming the entry.
   */
  async decodeConfigValues(entries: readonly (readonly [key: string, serialized: string])[]): Promise<ConfigValue[]> {
    await this.#ready;
    return (await this.#call({ kind: "decodeConfigValues", entries: entries.map(([key, value]) => [key, value]) })) as ConfigValue[];
  }

  /**
   * The inverse of {@link decodeConfigValues}: encodes `[key, value]` entries, typed values in the
   * shape `decodeConfigValues` returns, into `ConfigOption::serialize()` strings in one round trip.
   * Rejects on an unknown key or a value of the wrong shape, naming the entry.
   */
  async encodeConfigValues(entries: readonly (readonly [key: string, value: ConfigValue])[]): Promise<string[]> {
    await this.#ready;
    return (await this.#call({ kind: "encodeConfigValues", entries: entries.map(([key, value]) => [key, value]) })) as string[];
  }

  /** Slices an STL model with Ares slice options. */
  async sliceStl(input: Uint8Array, options: Record<string, unknown> = {}): Promise<SliceResult> {
    await this.#ready;
    return (await this.#call({
      kind: "sliceStl",
      input: input.slice(),
      options: JSON.stringify(options),
    })) as SliceResult;
  }

  /**
   * libvgcode `ViewerImpl` view range of the latest `sliceProject` toolpaths,
   * as GCodeViewer computes it for the legend and layer slider: shows the
   * extrusion roles with bit `i` of `visibleRoles` set (`EXTRUSION_ROLES`
   * order) and the options with bit `i` of `visibleOptions` set
   * (`Toolpaths.optionTypes` order), then sets the inclusive 0-based `layers` range.
   * Returns vertex indices `[first, last, current]`; `current` is the vertex
   * whose G-code line the G-code window marks.
   */
  async viewRange(layers: [number, number], visibleRoles: number, visibleOptions: number): Promise<ViewRange> {
    await this.#ready;
    return (await this.#call({ kind: "viewRange", layers, visibleRoles, visibleOptions })) as ViewRange;
  }

  /**
   * libvgcode `ViewerImpl` toolpath colors of the latest `sliceProject`
   * toolpaths in the `viewType` view (one of `Toolpaths.viewTypes`), as
   * GCodeViewer colors them after the legend's view type combo: with the
   * extrusion roles and options of `visibleRoles` / `visibleOptions` shown
   * (see `viewRange`), which the color ranges cover. Also returns the view's
   * color range for the legend.
   */
  async viewColors(viewType: string, visibleRoles: number, visibleOptions: number): Promise<ViewColors> {
    await this.#ready;
    return (await this.#call({ kind: "viewColors", viewType, visibleRoles, visibleOptions })) as ViewColors;
  }

  /**
   * Moves slider of the latest `sliceProject` toolpaths (GCodeViewer
   * `update_moves_slider`): sets the view as `viewRange` does, then returns
   * one stop per G-code line of the top layer's moves and the thumbs. With
   * `higher`, the higher thumb moves there first (`movesHigher`). A changed
   * layer range (`set_layers_z_range`) puts the higher thumb at the first
   * stop when it was there, else at the last; otherwise both thumbs span the
   * visible range.
   */
  async movesSlider(layers: [number, number], visibleRoles: number, visibleOptions: number, higher?: number): Promise<MovesSlider> {
    await this.#ready;
    return (await this.#call({ kind: "movesSlider", layers, visibleRoles, visibleOptions, higher })) as MovesSlider;
  }

  /**
   * Moves the higher thumb of the latest `movesSlider` to stop `higher`,
   * clamped to the stops (`IMSlider::SetHigherValue`), and shows the moves
   * between the thumbs (libvgcode `set_view_visible_range`). Returns the view
   * range, the thumbs, the enabled segments and option markers, and the
   * `viewType` colors with the layers below the top one grayed while the
   * higher thumb is not at the last stop, and the tool marker once the view
   * ended before the last move since the slice.
   */
  async movesHigher(higher: number, viewType: string, visibleRoles: number, visibleOptions: number): Promise<MovesView> {
    await this.#ready;
    return (await this.#call({ kind: "movesHigher", higher, viewType, visibleRoles, visibleOptions })) as MovesView;
  }

  terminate(): void {
    this.#worker.terminate();
  }

  #call(body: RequestBody, onStatus?: (status: SlicingStatus) => void): Promise<unknown> {
    const id = this.#nextId++;
    const request = { ...body, id } as WorkerRequest;
    const transfer = "inputs" in request ? request.inputs.map((input) => input.buffer) : "input" in request ? [request.input.buffer] : [];
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve, reject, onStatus });
      this.#worker.postMessage(request, transfer);
    });
  }
}
