import type { PrintConfigDef } from "./configDef.js";
import { PlaterSession } from "./platerSession.js";
import { PresetCatalog } from "./presetCatalog.js";
import type { RequestBody } from "./protocol.js";
import type { MovesSlider, MovesView, SliceResult, ViewColors, ViewRange, ViewerTextureOptions } from "./toolpaths.js";
import type {
  AresSlicerOptions,
  CompatiblePresets,
  ConfigValue,
  DirectProjectSliceOptions,
  GuideModel,
  OptionMap,
  ParsedPresetBundle,
  ProfileSelection,
  ProfileSliceOptions,
  ProjectPresetSelection,
  Scene,
  StlFile,
  StlImportOptions,
} from "./types.js";
import { profilePayload, stlBatchPayload, WorkerClient } from "./workerClient.js";

export * from "./toolpaths.js";
export { PlaterSession } from "./platerSession.js";
export { PresetCatalog, type FilamentSlotState } from "./presetCatalog.js";
export type * from "./configDef.js";
export type * from "./types.js";

/**
 * Promise client for the Ares WebAssembly slicer. Every call runs in the
 * worker, so slicing never blocks the calling thread (the role of
 * OrcaSlicer's `BackgroundSlicingProcess`). Input bytes are copied before
 * transfer; callers keep ownership of their buffers.
 */
export class AresSlicer {
  readonly #client: WorkerClient;
  #nextCatalogId = 0;
  #nextSessionId = 0;

  constructor(options: AresSlicerOptions) {
    this.#client = new WorkerClient(options);
  }

  /** Resolves once the WebAssembly module is instantiated in the worker. */
  ready(): Promise<void> {
    return this.#client.ready();
  }

  /** `PrintConfigDef` option metadata (labels, units, enums, ranges, modes) for settings UIs. */
  printConfigDef(): Promise<PrintConfigDef> {
    return this.#client.call({ kind: "printConfigDef" });
  }

  /** `Preset::get_default_bed_type`, with an inherited hint and resolved catalog model ID. */
  resolveDefaultBedType(hint: string, modelId: string): Promise<string> {
    return this.#client.call({ kind: "resolveDefaultBedType", hint, modelId });
  }

  /** The OrcaSlicer release whose behaviour the engine ports, such as `"2.5.0"`. */
  orcaSlicerVersion(): Promise<string> {
    return this.#client.call({ kind: "orcaSlicerVersion" });
  }

  /** Loads a 3MF project or STL model into plater scene data. */
  loadScene(input: Uint8Array): Promise<Scene> {
    return this.#client.call({ kind: "loadScene", input: input.slice() });
  }

  /**
   * Slices one plate of a 3MF project (first plate when omitted). `overrides`
   * are user edits applied over the project config, by option key: a
   * `ConfigOption::serialize()` string from `encodeConfigValues`, or one string
   * per element of a vector option. Rejects an unknown key or invalid value.
   */
  sliceProject(
    input: Uint8Array,
    plate?: number,
    overrides: Record<string, string | string[]> = {},
    options?: DirectProjectSliceOptions,
  ): Promise<SliceResult> {
    return this.#client.call({
      kind: "sliceProject",
      input: input.slice(),
      plate,
      overrides: JSON.stringify(overrides),
      catalog: options?.catalogJson,
    });
  }

  /**
   * Slices a model with selected OrcaSlicer presets, like the OrcaSlicer CLI's
   * `--load-settings` / `--load-filaments`. `filename` is the model's file name
   * (`File.name`); it names STL objects. `catalogJson` is the preset catalog
   * `{"vendors": [...], "imports": [...]}`; the engine resolves inheritance and
   * composes the config. Serialize a large catalog once and reuse the string.
   */
  sliceWithProfiles(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<SliceResult> {
    return this.#client.call({
      kind: "sliceWithProfiles",
      ...profilePayload(input, filename, catalogJson, selection, options),
    });
  }

  /**
   * The scene `sliceWithProfiles` slices for the same arguments: the model
   * loaded with the selected presets and placed as `options.placement` asks
   * (an STL centered on the selected printer's bed), with the composed
   * config as `settings`. Rejects as `sliceWithProfiles` does.
   */
  placementScene(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<Scene> {
    return this.#client.call({
      kind: "placementScene",
      ...profilePayload(input, filename, catalogJson, selection, options),
    });
  }

  /**
   * The config `sliceWithProfiles` slices with for the same arguments: the
   * selected presets composed over the input (`PresetBundle::full_config`) with
   * `options.overrides` applied, as one option map in the stored form of
   * `Scene.settings`. Rejects as `sliceWithProfiles` does on a bad catalog,
   * selection or override.
   */
  profileSettings(
    input: Uint8Array,
    filename: string,
    catalogJson: string,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<OptionMap> {
    return this.#client.call({
      kind: "profileSettings",
      ...profilePayload(input, filename, catalogJson, selection, options),
    });
  }

  /**
   * Reads the presets of an OrcaSlicer preset JSON or an `.orca_printer` /
   * `.orca_filament` / `.orca_bundle` archive (`PresetBundle::import_presets`).
   * Add the result to the catalog's `imports`.
   */
  parsePresetBundle(input: Uint8Array, filename: string): Promise<ParsedPresetBundle> {
    return this.#client.call({ kind: "parsePresetBundle", input: input.slice(), filename });
  }

  /**
   * The setup guide's printer models (`VendorProfile::PrinterModel`) of vendor envelopes
   * `[{id, index, files}]` as JSON bytes, such as `loadPrinterModels()` of `@ares-slicer/profiles`.
   */
  guideModels(vendorsJson: Uint8Array): Promise<GuideModel[]> {
    return this.#client.call({ kind: "guideModels", input: vendorsJson.slice() });
  }

  /** An empty preset catalog in the worker; add vendors and imports, then filter. */
  async createPresetCatalog(): Promise<PresetCatalog> {
    await this.#client.ready();
    const catalogId = this.#nextCatalogId++;
    await this.#client.send({ kind: "catalogCreate", catalogId });
    return new PresetCatalog((body) => this.#client.send(body), catalogId);
  }

  /**
   * Loads a 3MF project into an editable plater session in the worker
   * (`PartPlateList::load_from_3mf_structure`). `filename` names the file for preset slicing.
   * Rejects as `loadScene` does on a bad project.
   */
  createPlaterSession(input: Uint8Array, filename: string): Promise<PlaterSession> {
    return this.#openSession((sessionId) => ({ kind: "platerCreate", sessionId, input: input.slice(), filename }));
  }

  /**
   * Loads a model with selected presets into an editable plater session, the bed and plates from
   * the composed config: an STL lands on the bed centre as on an empty plate when
   * `options.placement` is "center" (`Plater::priv::load_model_objects`). Arguments and
   * rejections are those of `sliceWithProfiles`.
   */
  createPlaterSessionWithProfiles(
    input: Uint8Array,
    filename: string,
    catalog: string | PresetCatalog,
    selection: ProfileSelection | ProjectPresetSelection,
    options: ProfileSliceOptions,
  ): Promise<PlaterSession> {
    const ref = typeof catalog === "string" ? catalog : catalog.workerId;
    return this.#openSession((sessionId) => ({
      kind: "platerCreateWithProfiles",
      sessionId,
      ...profilePayload(input, filename, ref, selection, options),
    }));
  }

  /** Loads every named STL through the core's multipart load policy. */
  createPlaterSessionFromStlFiles(files: readonly StlFile[], choices: StlImportOptions): Promise<PlaterSession> {
    return this.#openSession((sessionId) => ({
      kind: "platerCreateFromStlFiles",
      sessionId,
      ...stlBatchPayload(files, choices),
    }));
  }

  /** Loads the complete STL batch onto the selected printer's bed. */
  createPlaterSessionFromStlFilesWithProfiles(
    files: readonly StlFile[],
    choices: StlImportOptions,
    catalog: string | PresetCatalog,
    selection: ProfileSelection,
    options: ProfileSliceOptions,
  ): Promise<PlaterSession> {
    const ref = typeof catalog === "string" ? catalog : catalog.workerId;
    return this.#openSession((sessionId) => ({
      kind: "platerCreateFromStlFilesWithProfiles",
      sessionId,
      ...stlBatchPayload(files, choices),
      catalog: ref,
      selection: JSON.stringify(selection),
      options: JSON.stringify(options),
    }));
  }

  /**
   * Selectable processes and filaments of the catalog compatible with the
   * `printer` preset, and filaments also with the `process` preset when given
   * (`Preset::is_compatible_with_printer` / `is_compatible_with_print`).
   * `catalogJson` is the same catalog string `sliceWithProfiles` takes.
   * Rejects when a selected preset or one of its parents is not in the catalog.
   */
  filterCompatiblePresets(catalogJson: string, printer: string, process?: string): Promise<CompatiblePresets> {
    return this.#client.call({ kind: "filterCompatiblePresets", catalog: catalogJson, printer, process });
  }

  /**
   * Loads the presets of `catalog` (the `sliceWithProfiles` catalog string or a worker
   * `PresetCatalog` handle) once, as OrcaSlicer loads its `PresetBundle` at startup and after the
   * wizard; a later project load with the same catalog reuses them.
   */
  async loadPresetBundle(catalog: string | PresetCatalog): Promise<void> {
    await this.#client.call({
      kind: "loadPresetBundle",
      catalog: typeof catalog === "string" ? catalog : catalog.workerId,
    });
  }

  /**
   * Decodes `[key, serialized]` entries, each a value in `ConfigOption::serialize()` form as
   * 3MF projects and presets store it (e.g. `"1"` for a bool, `"0x0"` for a point), into typed
   * values in one round trip. A vector key decodes its comma-joined elements to an array.
   * Rejects on an unknown key or a malformed value, naming the entry.
   */
  decodeConfigValues(entries: readonly (readonly [key: string, serialized: string])[]): Promise<ConfigValue[]> {
    return this.#client.call({ kind: "decodeConfigValues", entries: entries.map(([key, value]) => [key, value]) });
  }

  /**
   * The inverse of {@link decodeConfigValues}: encodes `[key, value]` entries, typed values in the
   * shape `decodeConfigValues` returns, into `ConfigOption::serialize()` strings in one round trip.
   * Rejects on an unknown key or a value of the wrong shape, naming the entry.
   */
  encodeConfigValues(entries: readonly (readonly [key: string, value: ConfigValue])[]): Promise<string[]> {
    return this.#client.call({ kind: "encodeConfigValues", entries: entries.map(([key, value]) => [key, value]) });
  }

  /** Slices an STL model with Ares slice options. */
  sliceStl(input: Uint8Array, options: Record<string, unknown> = {}): Promise<SliceResult> {
    return this.#client.call({ kind: "sliceStl", input: input.slice(), options: JSON.stringify(options) });
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
  viewRange(layers: [number, number], visibleRoles: number, visibleOptions: number): Promise<ViewRange> {
    return this.#client.call({ kind: "viewRange", layers, visibleRoles, visibleOptions });
  }

  /**
   * libvgcode `ViewerImpl` toolpath colors of the latest `sliceProject`
   * toolpaths in the `viewType` view (one of `Toolpaths.viewTypes`), as
   * GCodeViewer colors them after the legend's view type combo: with the
   * extrusion roles and options of `visibleRoles` / `visibleOptions` shown
   * (see `viewRange`), which the color ranges cover. Also returns the view's
   * color range for the legend.
   */
  viewColors(viewType: string, visibleRoles: number, visibleOptions: number): Promise<ViewColors> {
    return this.#client.call({ kind: "viewColors", viewType, visibleRoles, visibleOptions });
  }

  /** From now on the worker packs `Toolpaths.textures` of each slice result, in rows of up to `maxTextureSize` texels. */
  async prepareViewerTextures(maxTextureSize: number, options?: ViewerTextureOptions): Promise<void> {
    await this.#client.send({
      kind: "prepareViewerTextures",
      maxTextureSize,
      probes: options?.probes,
      includeRawArrays: options?.includeRawArrays,
    });
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
  movesSlider(
    layers: [number, number],
    visibleRoles: number,
    visibleOptions: number,
    higher?: number,
  ): Promise<MovesSlider> {
    return this.#client.call({ kind: "movesSlider", layers, visibleRoles, visibleOptions, higher });
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
  movesHigher(higher: number, viewType: string, visibleRoles: number, visibleOptions: number): Promise<MovesView> {
    return this.#client.call({ kind: "movesHigher", higher, viewType, visibleRoles, visibleOptions });
  }

  terminate(): void {
    this.#client.terminate();
  }

  async #openSession(create: (sessionId: number) => RequestBody): Promise<PlaterSession> {
    await this.#client.ready();
    const sessionId = this.#nextSessionId++;
    await this.#client.send(create(sessionId));
    return new PlaterSession((body, onStatus) => this.#client.send(body, onStatus), sessionId);
  }
}
