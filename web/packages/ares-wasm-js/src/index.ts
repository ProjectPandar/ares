import type { PrintConfigDef } from "./configDef.js";
import type { WorkerRequest, WorkerResponse } from "./protocol.js";
import type { ConfigOverrides, SliceResult, ViewRange } from "./toolpaths.js";
import type { CompatiblePresets, ParsedPresetBundle, ProfileSelection, ProfileSliceOptions, Scene } from "./types.js";

export * from "./toolpaths.js";
export type * from "./configDef.js";
export type * from "./types.js";

type Pending = { resolve: (value: unknown) => void; reject: (error: Error) => void };
type RequestBody = WorkerRequest extends infer R ? (R extends unknown ? Omit<R, "id"> : never) : never;

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

  constructor({ worker, wasmUrl }: AresSlicerOptions) {
    this.#worker = worker;
    worker.onmessage = ({ data }: MessageEvent<WorkerResponse>) => {
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
   * edit the project config before slicing; they are rejected until ares-core
   * ships its option override API (M854).
   */
  async sliceProject(input: Uint8Array, plate?: number, overrides: ConfigOverrides = {}): Promise<SliceResult> {
    const edited = Object.keys(overrides);
    if (edited.length > 0) {
      throw new Error(`Option overrides are not supported by this Ares build yet (edited: ${edited.join(", ")})`);
    }
    await this.#ready;
    return (await this.#call({ kind: "sliceProject", input: input.slice(), plate })) as SliceResult;
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
   * Reads the presets of an OrcaSlicer preset JSON or an `.orca_printer` /
   * `.orca_filament` / `.orca_bundle` archive (`PresetBundle::import_presets`).
   * Add the result to the catalog's `imports`.
   */
  async parsePresetBundle(input: Uint8Array, filename: string): Promise<ParsedPresetBundle> {
    await this.#ready;
    return (await this.#call({ kind: "parsePresetBundle", input: input.slice(), filename })) as ParsedPresetBundle;
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
   * (`OPTION_TYPES` order), then sets the inclusive 0-based `layers` range.
   * Returns vertex indices `[first, last, current]`; `current` is the vertex
   * whose G-code line the G-code window marks.
   */
  async viewRange(layers: [number, number], visibleRoles: number, visibleOptions: number): Promise<ViewRange> {
    await this.#ready;
    return (await this.#call({ kind: "viewRange", layers, visibleRoles, visibleOptions })) as ViewRange;
  }

  terminate(): void {
    this.#worker.terminate();
  }

  #call(body: RequestBody): Promise<unknown> {
    const id = this.#nextId++;
    const request = { ...body, id } as WorkerRequest;
    const transfer = "input" in request ? [request.input.buffer] : [];
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve, reject });
      this.#worker.postMessage(request, transfer);
    });
  }
}
