import type { WorkerRequest, WorkerResponse } from "./protocol";
import type { ConfigOverrides, SliceResult } from "./toolpaths";
import type { Scene } from "./types";

export * from "./toolpaths";
export type * from "./types";

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

  /** Slices an STL model with Ares slice options. */
  async sliceStl(input: Uint8Array, options: Record<string, unknown> = {}): Promise<SliceResult> {
    await this.#ready;
    return (await this.#call({
      kind: "sliceStl",
      input: input.slice(),
      options: JSON.stringify(options),
    })) as SliceResult;
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
